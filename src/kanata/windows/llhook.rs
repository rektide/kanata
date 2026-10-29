use kanata_parser::cfg::WinSyncKeystateBehaviour;
use parking_lot::Mutex;
use std::convert::TryFrom;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, RecvError, SyncSender as Sender, TryRecvError, sync_channel};
use std::time;

use super::PRESSED_KEYS;
use crate::kanata::mouse_axis::{AXIS_COUNT, AxisState, MouseAxis};
use crate::kanata::*;

impl Kanata {
    /// Initialize the callback that is passed to the Windows low level hook to receive key events
    /// and run the native_windows_gui event loop.
    pub fn event_loop(
        _cfg: Arc<Mutex<Self>>,
        tx: Sender<KeyEvent>,
        #[cfg(all(target_os = "windows", feature = "gui"))]
        ui: crate::gui::system_tray_ui::SystemTrayUi,
    ) -> Result<()> {
        // Display debug and panic output when launched from a terminal.
        #[cfg(not(feature = "gui"))]
        unsafe {
            use winapi::um::wincon::*;
            if AttachConsole(ATTACH_PARENT_PROCESS) != 0 {
                panic!("Could not attach to console");
            }
        };

        let (preprocess_tx, preprocess_rx) = sync_channel(100);
        start_event_preprocessor(preprocess_rx, tx);
        let kb_preprocess_tx = preprocess_tx.clone();

        // This callback should return `false` if the input event is **not** handled by the
        // callback and `true` if the input event **is** handled by the callback. Returning false
        // informs the callback caller that the input event should be handed back to the OS for
        // normal processing.
        let _kbhook = KeyboardHook::set_input_cb(move |input_event| {
            let mut key_event = match KeyEvent::try_from(input_event) {
                Ok(ev) => ev,
                _ => return false,
            };

            check_for_exit(&key_event);
            let oscode = key_event.code;
            if !MAPPED_KEYS.lock().contains(&oscode) {
                return false;
            }

            // Unlike Linux, Windows does not use a separate value for repeat. However, our code
            // needs to differentiate between initial press and repeat press.
            log::debug!("event loop: {:?}", key_event);
            match key_event.value {
                KeyValue::Release if PRESSED_KEYS.lock().remove(&key_event.code).is_none() => {
                    log::debug!("forwarding release of never-seen press {:?}", key_event);
                    // If we never saw the initial press for this release,
                    // we should not pass it along to the processing loop.
                    // There is likely a pressed key in Windows that Kanata never knew about.
                    // Consuming the release here will mean Windows gets a stuck press.
                    return false;
                }
                KeyValue::Press => {
                    let mut pressed_keys = PRESSED_KEYS.lock();
                    if let std::collections::hash_map::Entry::Vacant(e) =
                        pressed_keys.entry(key_event.code)
                    {
                        e.insert(web_time::Instant::now());
                    } else {
                        key_event.value = KeyValue::Repeat;
                    }
                }
                _ => {}
            }

            // Send input_events to the preprocessing loop. Panic if channel somehow gets full or if
            // channel disconnects. Typing input should never trigger a panic based on the channel
            // getting full, assuming regular operation of the program and some other bug isn't the
            // problem. I've tried to crash the program by pressing as many keys on my keyboard at
            // the same time as I could, but was unable to.
            try_send_panic(&kb_preprocess_tx, key_event);
            true
        });

        use OsCode::*;
        let oscodes_for_mhook_active = &[
            BTN_LEFT,
            BTN_RIGHT,
            BTN_MIDDLE,
            BTN_SIDE,
            BTN_EXTRA,
            MouseWheelUp,
            MouseWheelDown,
            MouseWheelLeft,
            MouseWheelRight,
        ];
        let motion_snapshot = _cfg.lock().motion_snapshot.clone();
        let maps_movement_at_startup = motion_snapshot.lock().map.maps_movement();
        let _handle = if oscodes_for_mhook_active
            .iter()
            .any(|osc| MAPPED_KEYS.lock().contains(osc))
            || motion_snapshot.lock().map.maps_wheel()
            || maps_movement_at_startup
        {
            log::info!("Installing mouse hook callback.");
            if maps_movement_at_startup {
                log::info!(
                    "Mouse movement axis mapping is active: physical mouse moves are blocked \
                     from the OS pipeline and re-injected with the mapped position. This adds \
                     slight movement latency, and Raw Input consumers (e.g. some games) still \
                     observe the physical events."
                );
            }
            // Mouse outputs for the axis remapping path. SendInput runs on
            // this dedicated thread; see `AxisInjection` for why it must not
            // run on the hook thread or the processing thread.
            let (axis_injection_tx, axis_injection_rx) =
                std::sync::mpsc::channel::<crate::oskbd::AxisInjection>();
            std::thread::spawn(move || {
                while let Ok(injection) = axis_injection_rx.recv() {
                    injection
                        .emit()
                        .expect("failed to inject mapped mouse axes");
                }
            });
            let mut axis_state = AxisState::default();
            let mousehook = MouseHook::set_input_cb(move |mouse_event| {
                log::debug!("llhook mouse event: {mouse_event:?}");
                let snapshot = motion_snapshot.lock().clone();
                axis_state.observe_snapshot(&snapshot);
                let map = snapshot.map;
                let mut input = [0; AXIS_COUNT];
                let is_axis_event = match mouse_event {
                    MouseEventType::Move(MouseMoveEvent { point: Some(point) })
                        if map.maps_movement() =>
                    {
                        let Some(current) = current_mouse_point() else {
                            return false;
                        };
                        input[MouseAxis::X as usize] = point.x.saturating_sub(current.x);
                        input[MouseAxis::Y as usize] = point.y.saturating_sub(current.y);
                        true
                    }
                    MouseEventType::Wheel(wheel) if !map.is_identity() => {
                        let axis = match wheel.wheel {
                            MouseWheel::Vertical => MouseAxis::V,
                            MouseWheel::Horizontal => MouseAxis::H,
                            _ => return false,
                        };
                        input[axis as usize] = wheel.delta.into();
                        true
                    }
                    _ => false,
                };
                let mut output = axis_state.apply(input);
                if is_axis_event {
                    axis_state
                        .route_wheels(&mut output, snapshot.wheels, |code| {
                            try_send_panic(&preprocess_tx, KeyEvent::new(code, KeyValue::Tap));
                            Ok(())
                        })
                        .expect("route wheel taps");
                    if output.iter().any(|value| *value != 0) {
                        axis_injection_tx
                            .send(crate::oskbd::AxisInjection { deltas: output })
                            .expect("mouse axis output thread disconnected");
                    }
                    return true;
                }
                let key_event = match KeyEvent::try_from(mouse_event) {
                    Ok(ev) => ev,
                    _ => return false,
                };
                if !(if WheelMask::is_wheel(key_event.code) {
                    snapshot.wheels.contains(key_event.code)
                } else {
                    MAPPED_KEYS.lock().contains(&key_event.code)
                }) {
                    return false;
                }
                try_send_panic(&preprocess_tx, key_event);
                true
            });
            log::info!("Installed mouse hook callback successfully.");
            Some(mousehook)
        } else {
            log::info!("No mouse inputs were in defsrc on startup. Not activating mouse hook.");
            None
        };

        #[cfg(all(target_os = "windows", feature = "gui"))]
        let _ui = ui; // prevents thread from panicking on exiting via a GUI
        // The event loop is also required for the low-level keyboard hook to work.
        native_windows_gui::dispatch_thread_events();
        Ok(())
    }

    /// # Note
    ///
    /// This is disabled by default due to known issues.
    /// Under some use cases this works just fine and can be enabled.
    ///
    /// ## Known issues
    ///
    /// - Ralt/lctl may have some issues with AltGr layouts
    /// - Some software may have a later-stage remapping that changes which VK is active for a
    ///   given VK that Kanata outputs. E.g. changing to roya/loya modifiers.
    ///
    /// # Description
    ///
    /// On Windows with LLHOOK/SendInput APIs,
    /// Kanata does not have as much control
    /// over the full system's keystates as one would want;
    /// unlike in Linux or with the Interception driver.
    /// Sometimes Kanata can miss events; e.g. a release is
    /// missed and a keystate remains pressed within Kanata (1),
    /// or a press is missed in Kanata but the release is caught,
    /// and thus the keystate remains pressed within the Windows system
    /// because Kanata consumed the release and didn't know what to do about it (2).
    ///
    /// For (1), `release_normalkey_states` theoretically fixes the issue
    /// after 60s of Kanata being idle,
    /// but that is a long time and doesn't seem to work consistently.
    /// Unfortunately this does not seem to be easily fixable in all cases.
    /// For example, a press consumed by Kanata could result in
    /// **only** a `(layer-while-held ...)` action as the output;
    /// if the corresponding release were missed,
    /// Kanata has no information available from the larger Windows system
    /// to confirm that the physical key is actually released
    /// but that the process didn't see the event.
    /// E.g. there is the `GetAsyncKeyState` API
    /// and this will be useful when the missed release has a key output,
    /// but not with the layer example.
    /// There does not appear to be any "raw input" mechanism
    /// to see the snapshot of the current state of physical keyboard keys.
    ///
    /// For (2), consider that this might be fixed purely within Kanata's
    /// event handling and processing, by checking Kanata's active action states,
    /// and if there are no active states corresponding to a released event,
    /// to send a release of the original input.
    /// This would result in extra release events though;
    /// for example if the `A` key action is `(macro a)`,
    /// the above logic will result in a second SendInput release event of `A`.
    /// Instead, this function checks against the outside Windows state.
    ///
    /// The solution makes use of the following states:
    /// - `MAPPED_KEYS` (MK)
    /// - `GetAsyncKeyState` WinAPI (GKS)
    /// - `PRESSED_KEYS` (PK)
    /// - `self.prev_keys` (SPV)
    ///
    /// If a discrepancy is detected,
    /// this procedure releases Windows keys via SendInput
    /// and/or clears internal Kanata states.
    ///
    /// The checks are:
    /// 1. For all of SPV, check that it is pressed in GKS.
    ///    If a key is not pressed, find the coordinate of this state.
    ///    Clear in PK and clear all states with the same coordinate as key output.
    /// 2. For all keys in MK and active in GKS, check it is in SPV.
    ///    If not in SPV, call SendInput to release in Windows.
    #[cfg(not(feature = "simulated_input"))]
    pub(crate) fn win_synchronize_keystates(&mut self) {
        use kanata_keyberon::layout::*;
        use winapi::um::winuser::*;

        if matches!(
            self.windows_sync_keystates,
            WinSyncKeystateBehaviour::WinSyncDoNothing
        ) {
            return;
        }

        // Note: this procedure (1) used to be also gated behind the config flag.
        // Revisiting it later, it should be reasonably safe to always run this,
        // because releasing a Kanata state should not interfere with other programs
        // in contrast to releasing Windows VK codes which could.
        log::debug!("synchronizing kanata keystates with windows");
        for pvk in self.prev_keys.iter() {
            // Check 1 : each pvk is expected to be pressed.
            let osc: OsCode = pvk.into();

            let vk = match osc {
                OsCode::KEY_KPENTER => {
                    // Special case:
                    // From scancode info we can differentiate KP Enter vs. Enter
                    // but we cannot from Windows VK states.
                    // Check against VK_RETURN
                    VK_RETURN
                }
                _ => i32::from(osc),
            };

            if vk > 254 {
                // 254 should be highest valid VK number in Windows OS.
                continue;
            }

            let vk_state = unsafe { GetAsyncKeyState(vk) } as u32;
            let is_pressed_in_windows = vk_state >= 0b1000000;
            if is_pressed_in_windows {
                continue;
            }

            log::error!(
                "Unexpected keycode is pressed in kanata but not in Windows. Clearing kanata states: {pvk:?}"
            );
            // Need to clear internal state about this key.
            // find coordinate(s) in keyberon associated with pvk
            let mut coords_to_clear = Vec::<KCoord>::new();
            let layout = self.layout.bm();
            layout.states.retain(|s| {
                let retain = match s.keycode() {
                    Some(k) => k != *pvk,
                    _ => true,
                };
                if !retain && let Some(coord) = s.coord() {
                    coords_to_clear.push(coord);
                }
                retain
            });

            // Clear other states other than keycode associated with a keycode that needs to be
            // cleaned up.
            layout.states.retain(|s| match s.coord() {
                Some(c) => !coords_to_clear.contains(&c),
                None => true,
            });

            // Clear PRESSED_KEYS for coordinates associated with real and not virtual keys
            let mut pressed_keys = PRESSED_KEYS.lock();
            for osc in coords_to_clear.iter().copied().filter_map(|c| match c {
                (FAKE_KEY_ROW, _) => None,
                (_, kc) => Some(OsCode::from(kc)),
            }) {
                pressed_keys.remove(&osc);
            }
            drop(pressed_keys);
        }

        if matches!(
            self.windows_sync_keystates,
            WinSyncKeystateBehaviour::WinSyncClearKanataStates
        ) {
            return;
        }

        log::debug!("releasing windows keystates with kanata");
        let mapped_keys = MAPPED_KEYS.lock();
        for mapped_osc in mapped_keys.iter().copied() {
            // Check 2: each active win vk mapped in Kanata should have a value in pvk
            if matches!(
                mapped_osc,
                OsCode::BTN_LEFT
                    | OsCode::BTN_RIGHT
                    | OsCode::BTN_MIDDLE
                    | OsCode::BTN_SIDE
                    | OsCode::BTN_EXTRA
            ) {
                // Skip mouse. Probably not under primary control of Kanata.
                continue;
            }
            let vk = i32::from(mapped_osc);
            if vk >= 256 {
                continue;
            }
            let vk_state = unsafe { GetAsyncKeyState(vk) } as u32;
            let is_pressed_in_windows = vk_state >= 0b1000000;
            if !is_pressed_in_windows {
                continue;
            }
            let vk = vk as u16;
            let Some(osc) = OsCode::from_u16(vk) else {
                continue;
            };
            if self.prev_keys.contains(&osc.into()) {
                continue;
            }
            log::error!(
                "Unexpected keycode is pressed in Windows but not Kanata. Releasing in Windows: {osc}"
            );
            let _ = release_key(&mut self.kbd_out, osc);
        }
        drop(mapped_keys);
    }
}

fn try_send_panic(tx: &Sender<KeyEvent>, kev: KeyEvent) {
    if let Err(e) = tx.try_send(kev) {
        panic!("failed to send on channel: {e:?}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum LctlState {
    Pressed,
    Released,
    Pending,
    PendingReleased,
    None,
}

#[derive(Debug, Clone, Copy)]
enum RecvValue {
    Ok(KeyEvent),
    Empty,
    Disconnected,
}

enum CanBlock {
    YesCanBlock,
    NoMustPeriodicPoll,
}

fn preprocessor_recv(preprocess_rx: &Receiver<KeyEvent>, can_block: CanBlock) -> RecvValue {
    match can_block {
        CanBlock::YesCanBlock => match preprocess_rx.recv() {
            Ok(kev) => RecvValue::Ok(kev),
            Err(RecvError) => RecvValue::Disconnected,
        },
        CanBlock::NoMustPeriodicPoll => match preprocess_rx.try_recv() {
            Ok(kev) => RecvValue::Ok(kev),
            Err(TryRecvError::Empty) => RecvValue::Empty,
            Err(TryRecvError::Disconnected) => RecvValue::Disconnected,
        },
    }
}

fn start_event_preprocessor(preprocess_rx: Receiver<KeyEvent>, process_tx: Sender<KeyEvent>) {
    use CanBlock::*;
    use RecvValue::*;
    std::thread::spawn(move || {
        let mut lctl_state = LctlState::None;
        let mut can_block = NoMustPeriodicPoll;
        loop {
            can_block = match preprocessor_recv(&preprocess_rx, can_block) {
                Ok(kev) => {
                    match (*ALTGR_BEHAVIOUR.lock(), kev) {
                        (AltGrBehaviour::DoNothing, _) => {
                            try_send_panic(&process_tx, kev);
                        }
                        (
                            AltGrBehaviour::AddLctlRelease,
                            KeyEvent {
                                value: KeyValue::Release,
                                code: OsCode::KEY_RIGHTALT,
                                ..
                            },
                        ) => {
                            log::debug!("altgr add: adding lctl release");
                            try_send_panic(&process_tx, kev);
                            try_send_panic(
                                &process_tx,
                                KeyEvent::new(OsCode::KEY_LEFTCTRL, KeyValue::Release),
                            );
                            PRESSED_KEYS.lock().remove(&OsCode::KEY_LEFTCTRL);
                        }
                        (
                            AltGrBehaviour::CancelLctlPress,
                            KeyEvent {
                                value: KeyValue::Press,
                                code: OsCode::KEY_LEFTCTRL,
                                ..
                            },
                        ) => {
                            log::debug!("altgr cancel: lctl state->pressed");
                            lctl_state = LctlState::Pressed;
                        }
                        (
                            AltGrBehaviour::CancelLctlPress,
                            KeyEvent {
                                value: KeyValue::Release,
                                code: OsCode::KEY_LEFTCTRL,
                                ..
                            },
                        ) => match lctl_state {
                            LctlState::Pressed => {
                                log::debug!("altgr cancel: lctl state->released");
                                lctl_state = LctlState::Released;
                            }
                            LctlState::Pending => {
                                log::debug!("altgr cancel: lctl state->pending-released");
                                lctl_state = LctlState::PendingReleased;
                            }
                            LctlState::None => {
                                try_send_panic(&process_tx, kev);
                            }
                            _ => {}
                        },
                        (
                            AltGrBehaviour::CancelLctlPress,
                            KeyEvent {
                                value: KeyValue::Press,
                                code: OsCode::KEY_RIGHTALT,
                                ..
                            },
                        ) => {
                            log::debug!("altgr cancel: lctl state->none");
                            lctl_state = LctlState::None;
                            try_send_panic(&process_tx, kev);
                        }
                        (_, _) => {
                            try_send_panic(&process_tx, kev);
                        }
                    }
                    NoMustPeriodicPoll
                }
                Empty => {
                    can_block = if *ALTGR_BEHAVIOUR.lock() == AltGrBehaviour::CancelLctlPress {
                        match lctl_state {
                            LctlState::Pressed => {
                                log::debug!("altgr cancel: lctl state->pending");
                                lctl_state = LctlState::Pending;
                                NoMustPeriodicPoll
                            }
                            LctlState::Released => {
                                log::debug!("altgr cancel: lctl state->pending-released");
                                lctl_state = LctlState::PendingReleased;
                                NoMustPeriodicPoll
                            }
                            LctlState::Pending => {
                                log::debug!("altgr cancel: lctl state->send");
                                try_send_panic(
                                    &process_tx,
                                    KeyEvent::new(OsCode::KEY_LEFTCTRL, KeyValue::Press),
                                );
                                lctl_state = LctlState::None;
                                NoMustPeriodicPoll
                            }
                            LctlState::PendingReleased => {
                                log::debug!("altgr cancel: lctl state->send+release");
                                try_send_panic(
                                    &process_tx,
                                    KeyEvent::new(OsCode::KEY_LEFTCTRL, KeyValue::Press),
                                );
                                try_send_panic(
                                    &process_tx,
                                    KeyEvent::new(OsCode::KEY_LEFTCTRL, KeyValue::Release),
                                );
                                lctl_state = LctlState::None;
                                NoMustPeriodicPoll
                            }
                            _ => YesCanBlock,
                        }
                    } else {
                        YesCanBlock
                    };
                    std::thread::sleep(time::Duration::from_millis(1));
                    can_block
                }
                Disconnected => {
                    panic!("channel disconnected")
                }
            }
        }
    });
}
