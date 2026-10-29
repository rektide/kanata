use anyhow::{Result, anyhow};
use kanata_interception as ic;
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::mpsc::SyncSender as Sender;

use super::PRESSED_KEYS;
use crate::kanata::mouse_axis::{AXIS_COUNT, AxisState, MotionSnapshot, MouseAxis, WheelMask};
use crate::kanata::*;
use crate::oskbd::KeyValue;
use kanata_parser::keys::OsCode;

impl Kanata {
    pub fn event_loop_inner(kanata: Arc<Mutex<Self>>, tx: Sender<KeyEvent>) -> Result<()> {
        let intrcptn = ic::Interception::new().ok_or_else(|| anyhow!("interception driver should init: have you completed the interception driver installation?"))?;
        intrcptn.set_filter(ic::is_keyboard, ic::Filter::KeyFilter(ic::KeyFilter::all()));
        let mut strokes = [ic::Stroke::Keyboard {
            code: ic::ScanCode::Esc,
            state: ic::KeyState::empty(),
            information: 0,
        }; 32];

        let keyboards_to_intercept_hwids = kanata.lock().intercept_kb_hwids.clone();
        let keyboards_to_intercept_hwids_exclude = kanata.lock().intercept_kb_hwids_exclude.clone();
        let mouse_to_intercept_hwids: Option<Vec<[u8; HWID_ARR_SZ]>> =
            kanata.lock().intercept_mouse_hwids.clone();
        let mouse_to_intercept_excluded_hwids: Option<Vec<[u8; HWID_ARR_SZ]>> =
            kanata.lock().intercept_mouse_hwids_exclude.clone();
        let mouse_movement_key = kanata.lock().mouse_movement_key.clone();
        let motion_snapshot = kanata.lock().motion_snapshot.clone();
        if mouse_to_intercept_hwids.is_some()
            || mouse_to_intercept_excluded_hwids.is_some()
            || !motion_snapshot.lock().map.is_identity()
        {
            // Movement remapping needs MOVE strokes as well, just like
            // mouse-movement-key.
            let intercept_move_strokes =
                mouse_movement_key.lock().is_some() || motion_snapshot.lock().map.maps_movement();
            if intercept_move_strokes {
                intrcptn.set_filter(ic::is_mouse, ic::Filter::MouseFilter(ic::MouseState::all()));
            } else {
                intrcptn.set_filter(
                    ic::is_mouse,
                    ic::Filter::MouseFilter(ic::MouseState::all() & (!ic::MouseState::MOVE)),
                );
            }
        }
        let mut is_dev_interceptable: HashMap<ic::Device, bool> = HashMap::default();
        let mut axis_states: HashMap<ic::Device, AxisState> = HashMap::default();
        loop {
            let dev = intrcptn.wait();
            if dev > 0 {
                let num_strokes = intrcptn.receive(dev, &mut strokes) as usize;
                for i in 0..num_strokes {
                    let key_event = match strokes[i] {
                        ic::Stroke::Keyboard { state, .. } => {
                            if !is_device_interceptable(
                                dev,
                                &intrcptn,
                                &keyboards_to_intercept_hwids,
                                &keyboards_to_intercept_hwids_exclude,
                                &mut is_dev_interceptable,
                            ) {
                                log::debug!("stroke {:?} is from undesired device", strokes[i]);
                                intrcptn.send(dev, &strokes[i..i + 1]);
                                continue;
                            }
                            log::debug!("got stroke {:?}", strokes[i]);
                            let code = match OsCodeWrapper::try_from(strokes[i]) {
                                Ok(c) => c.0,
                                _ => {
                                    log::debug!("could not map code to oscode");
                                    intrcptn.send(dev, &strokes[i..i + 1]);
                                    continue;
                                }
                            };
                            let value = match state.contains(ic::KeyState::UP) {
                                false => KeyValue::Press,
                                true => KeyValue::Release,
                            };
                            KeyEvent::new(code, value)
                        }
                        ic::Stroke::Mouse {
                            state,
                            rolling,
                            flags,
                            x,
                            y,
                            ..
                        } => {
                            let allow_this_dev = is_device_interceptable(
                                dev,
                                &intrcptn,
                                &mouse_to_intercept_hwids,
                                &mouse_to_intercept_excluded_hwids,
                                &mut is_dev_interceptable,
                            );

                            if !allow_this_dev {
                                intrcptn.send(dev, &strokes[i..i + 1]);
                                continue;
                            }
                            if let Some(ms_mvmt_key) = *mouse_movement_key.lock()
                                && !flags.contains(ic::MouseFlags::MOVE_ABSOLUTE)
                                && (x != 0 || y != 0)
                            {
                                tx.try_send(KeyEvent::new(ms_mvmt_key, KeyValue::Tap))?;
                            }
                            let snapshot = motion_snapshot.lock().clone();
                            let axis_state = axis_states.entry(dev).or_default();
                            axis_state.observe_snapshot(&snapshot);
                            if !snapshot.map.is_identity() {
                                let output = map_mouse_stroke_axes(
                                    axis_state,
                                    &snapshot,
                                    strokes[i],
                                    |code| MAPPED_KEYS.lock().contains(&code),
                                    |event| send_key_event(&tx, event),
                                )?;
                                intrcptn.send(dev, &output);
                                continue;
                            }
                            if let Some(event) =
                                mouse_state_to_event(state, rolling, snapshot.wheels)
                            {
                                event
                            } else {
                                intrcptn.send(dev, &strokes[i..i + 1]);
                                continue;
                            }
                        }
                    };
                    check_for_exit(&key_event);
                    if !WheelMask::is_wheel(key_event.code)
                        && !MAPPED_KEYS.lock().contains(&key_event.code)
                    {
                        log::debug!("{key_event:?} is not mapped");
                        intrcptn.send(dev, &strokes[i..i + 1]);
                        continue;
                    }
                    send_key_event(&tx, key_event)?;
                }
            }
        }
    }
    pub fn event_loop(
        kanata: Arc<Mutex<Self>>,
        tx: Sender<KeyEvent>,
        #[cfg(feature = "gui")] ui: crate::gui::system_tray_ui::SystemTrayUi,
    ) -> Result<()> {
        #[cfg(not(feature = "gui"))]
        {
            Self::event_loop_inner(kanata, tx)
        }
        #[cfg(feature = "gui")]
        {
            std::thread::spawn(move || -> Result<()> { Self::event_loop_inner(kanata, tx) });
            let _ui = ui; // prevents thread from panicking on exiting via a GUI
            native_windows_gui::dispatch_thread_events();
            Ok(())
        }
    }
}

fn send_key_event(tx: &Sender<KeyEvent>, mut event: KeyEvent) -> Result<()> {
    match event.value {
        KeyValue::Release => {
            PRESSED_KEYS.lock().remove(&event.code);
        }
        KeyValue::Press => {
            let mut pressed = PRESSED_KEYS.lock();
            if pressed.contains(&event.code) {
                event.value = KeyValue::Repeat;
            } else {
                pressed.insert(event.code);
            }
        }
        _ => {}
    }
    tx.try_send(event)?;
    Ok(())
}

fn is_device_interceptable(
    input_dev: ic::Device,
    intrcptn: &ic::Interception,
    allowed_hwids: &Option<Vec<[u8; HWID_ARR_SZ]>>,
    excluded_hwids: &Option<Vec<[u8; HWID_ARR_SZ]>>,
    cache: &mut HashMap<ic::Device, bool>,
) -> bool {
    match (allowed_hwids, excluded_hwids) {
        (None, None) => true,
        (Some(allowed), None) => match cache.get(&input_dev) {
            Some(v) => *v,
            None => {
                let mut hwid = [0u8; HWID_ARR_SZ];
                log::trace!("getting hardware id for input dev: {input_dev}");
                let res = intrcptn.get_hardware_id(input_dev, &mut hwid);
                let dev_is_interceptable = allowed.contains(&hwid);
                log::info!(
                    "include check - res {res}; device #{input_dev} is intercepted: {dev_is_interceptable}; hwid {hwid:?} "
                );
                cache.insert(input_dev, dev_is_interceptable);
                dev_is_interceptable
            }
        },
        (None, Some(excluded)) => match cache.get(&input_dev) {
            Some(v) => *v,
            None => {
                let mut hwid = [0u8; HWID_ARR_SZ];
                log::trace!("getting hardware id for input dev: {input_dev}");
                let res = intrcptn.get_hardware_id(input_dev, &mut hwid);
                let dev_is_interceptable = !excluded.contains(&hwid);
                log::info!(
                    "exclude check - res {res}; device #{input_dev} is intercepted: {dev_is_interceptable}; hwid {hwid:?} "
                );
                cache.insert(input_dev, dev_is_interceptable);
                dev_is_interceptable
            }
        },
        _ => unreachable!("excluded and allowed should be mutually exclusive"),
    }
}
fn mouse_state_to_event(
    state: ic::MouseState,
    rolling: i16,
    wheels: WheelMask,
) -> Option<KeyEvent> {
    if state.contains(ic::MouseState::RIGHT_BUTTON_DOWN) {
        Some(KeyEvent::new(OsCode::BTN_RIGHT, KeyValue::Press))
    } else if state.contains(ic::MouseState::RIGHT_BUTTON_UP) {
        Some(KeyEvent::new(OsCode::BTN_RIGHT, KeyValue::Release))
    } else if state.contains(ic::MouseState::LEFT_BUTTON_DOWN) {
        Some(KeyEvent::new(OsCode::BTN_LEFT, KeyValue::Press))
    } else if state.contains(ic::MouseState::LEFT_BUTTON_UP) {
        Some(KeyEvent::new(OsCode::BTN_LEFT, KeyValue::Release))
    } else if state.contains(ic::MouseState::MIDDLE_BUTTON_DOWN) {
        Some(KeyEvent::new(OsCode::BTN_MIDDLE, KeyValue::Press))
    } else if state.contains(ic::MouseState::MIDDLE_BUTTON_UP) {
        Some(KeyEvent::new(OsCode::BTN_MIDDLE, KeyValue::Release))
    } else if state.contains(ic::MouseState::BUTTON_4_DOWN) {
        Some(KeyEvent::new(OsCode::BTN_SIDE, KeyValue::Press))
    } else if state.contains(ic::MouseState::BUTTON_4_UP) {
        Some(KeyEvent::new(OsCode::BTN_SIDE, KeyValue::Release))
    } else if state.contains(ic::MouseState::BUTTON_5_DOWN) {
        Some(KeyEvent::new(OsCode::BTN_EXTRA, KeyValue::Press))
    } else if state.contains(ic::MouseState::BUTTON_5_UP) {
        Some(KeyEvent::new(OsCode::BTN_EXTRA, KeyValue::Release))
    } else if state.contains(ic::MouseState::WHEEL) {
        let osc = if rolling >= 0 {
            OsCode::MouseWheelUp
        } else {
            OsCode::MouseWheelDown
        };
        if wheels.contains(osc) {
            Some(KeyEvent::new(osc, KeyValue::Tap))
        } else {
            None
        }
    } else if state.contains(ic::MouseState::HWHEEL) {
        let osc = if rolling >= 0 {
            OsCode::MouseWheelRight
        } else {
            OsCode::MouseWheelLeft
        };
        if wheels.contains(osc) {
            Some(KeyEvent::new(osc, KeyValue::Tap))
        } else {
            None
        }
    } else {
        None
    }
}

/// A stroke can carry motion, buttons and wheel data together. Separate the
/// mapped components without dropping the other parts of the original stroke.
fn map_mouse_stroke_axes(
    axis_state: &mut AxisState,
    snapshot: &MotionSnapshot,
    stroke: ic::Stroke,
    is_mapped: impl Fn(OsCode) -> bool,
    mut send_key: impl FnMut(KeyEvent) -> Result<()>,
) -> Result<Vec<ic::Stroke>> {
    axis_state.observe_snapshot(snapshot);
    let ic::Stroke::Mouse {
        mut state,
        flags,
        rolling,
        x,
        y,
        information,
    } = stroke
    else {
        return Ok(vec![stroke]);
    };
    let wheel_bits = ic::MouseState::WHEEL | ic::MouseState::HWHEEL;
    let absolute = flags.contains(ic::MouseFlags::MOVE_ABSOLUTE);
    let mut input = [0; AXIS_COUNT];
    if !absolute {
        input[MouseAxis::X as usize] = x;
        input[MouseAxis::Y as usize] = y;
    }
    if state.contains(ic::MouseState::WHEEL) {
        input[MouseAxis::V as usize] = rolling.into();
    }
    if state.contains(ic::MouseState::HWHEEL) {
        input[MouseAxis::H as usize] = rolling.into();
    }
    let mut output = axis_state.apply(input);
    axis_state.route_wheels(&mut output, snapshot.wheels, |code| {
        send_key(KeyEvent::new(code, KeyValue::Tap))
    })?;
    state.remove(wheel_bits);

    use ic::MouseState as MS;
    for (bit, code, value) in [
        (MS::LEFT_BUTTON_DOWN, OsCode::BTN_LEFT, KeyValue::Press),
        (MS::LEFT_BUTTON_UP, OsCode::BTN_LEFT, KeyValue::Release),
        (MS::RIGHT_BUTTON_DOWN, OsCode::BTN_RIGHT, KeyValue::Press),
        (MS::RIGHT_BUTTON_UP, OsCode::BTN_RIGHT, KeyValue::Release),
        (MS::MIDDLE_BUTTON_DOWN, OsCode::BTN_MIDDLE, KeyValue::Press),
        (MS::MIDDLE_BUTTON_UP, OsCode::BTN_MIDDLE, KeyValue::Release),
        (MS::BUTTON_4_DOWN, OsCode::BTN_SIDE, KeyValue::Press),
        (MS::BUTTON_4_UP, OsCode::BTN_SIDE, KeyValue::Release),
        (MS::BUTTON_5_DOWN, OsCode::BTN_EXTRA, KeyValue::Press),
        (MS::BUTTON_5_UP, OsCode::BTN_EXTRA, KeyValue::Release),
    ] {
        if state.contains(bit) && is_mapped(code) {
            send_key(KeyEvent::new(code, value))?;
            state.remove(bit);
        }
    }
    let mut strokes = Vec::new();
    if !state.is_empty() || absolute || flags.contains(ic::MouseFlags::ATTRIBUTES_CHANGED) {
        strokes.push(ic::Stroke::Mouse {
            state,
            flags,
            rolling: 0,
            x: if absolute { x } else { 0 },
            y: if absolute { y } else { 0 },
            information,
        });
    }
    let (x, y) = (output[MouseAxis::X as usize], output[MouseAxis::Y as usize]);
    if x != 0 || y != 0 {
        strokes.push(ic::Stroke::Mouse {
            state: MS::empty(),
            flags: flags & ic::MouseFlags::MOVE_NO_COALESCE,
            rolling: 0,
            x,
            y,
            information,
        });
    }
    for (axis, state) in [(MouseAxis::V, MS::WHEEL), (MouseAxis::H, MS::HWHEEL)] {
        let mut delta = i64::from(output[axis as usize]);
        while delta != 0 {
            let rolling = delta.clamp(i16::MIN as i64, i16::MAX as i64) as i16;
            strokes.push(ic::Stroke::Mouse {
                state,
                flags: ic::MouseFlags::MOVE_RELATIVE,
                rolling,
                x: 0,
                y: 0,
                information,
            });
            delta -= i64::from(rolling);
        }
    }
    Ok(strokes)
}

#[cfg(test)]
mod axis_tests {
    use super::*;
    use crate::kanata::mouse_axis::MouseAxisMap;

    #[test]
    fn relative_zero_flag_and_combined_buttons_do_not_hide_motion() {
        let mut map = MouseAxisMap::default();
        map.rows[MouseAxis::X as usize][MouseAxis::X as usize] = -1.0;
        map.rows[MouseAxis::H as usize][MouseAxis::V as usize] = 1.0;
        let input = ic::Stroke::Mouse {
            state: ic::MouseState::LEFT_BUTTON_DOWN | ic::MouseState::WHEEL,
            flags: ic::MouseFlags::MOVE_RELATIVE,
            x: 4,
            y: 7,
            rolling: 60,
            information: 123,
        };
        let mut keys = vec![];
        let output = map_mouse_stroke_axes(
            &mut AxisState::default(),
            &MotionSnapshot::initial(map, WheelMask::default()),
            input,
            |code| code == OsCode::BTN_LEFT,
            |key| {
                keys.push(key);
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].code, OsCode::BTN_LEFT);
        assert!(matches!(
            output[0],
            ic::Stroke::Mouse {
                x: -4,
                y: 7,
                information: 123,
                ..
            }
        ));
        assert!(
            matches!(output[1], ic::Stroke::Mouse { state, rolling: 60, .. } if state == ic::MouseState::WHEEL)
        );
        assert!(
            matches!(output[2], ic::Stroke::Mouse { state, rolling: 60, .. } if state == ic::MouseState::HWHEEL)
        );
    }

    #[test]
    fn absolute_coordinates_are_preserved_but_wheels_can_still_map() {
        let mut map = MouseAxisMap::default();
        map.rows[MouseAxis::X as usize][MouseAxis::X as usize] = -1.0;
        map.rows[MouseAxis::V as usize][MouseAxis::V as usize] = -1.0;
        let output = map_mouse_stroke_axes(
            &mut AxisState::default(),
            &MotionSnapshot::initial(map, WheelMask::default()),
            ic::Stroke::Mouse {
                state: ic::MouseState::WHEEL,
                flags: ic::MouseFlags::MOVE_ABSOLUTE,
                x: 12000,
                y: 24000,
                rolling: i16::MIN,
                information: 0,
            },
            |_| false,
            |_| unreachable!(),
        )
        .unwrap();
        assert!(
            matches!(output[0], ic::Stroke::Mouse { x: 12000, y: 24000, flags, .. } if flags.contains(ic::MouseFlags::MOVE_ABSOLUTE))
        );
        // Inverting i16::MIN must split, not overflow or change sign.
        let total: i32 = output
            .iter()
            .map(|stroke| match stroke {
                ic::Stroke::Mouse { rolling, .. } => i32::from(*rolling),
                _ => 0,
            })
            .sum();
        assert_eq!(total, 32768);
    }
}
