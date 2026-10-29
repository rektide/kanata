#![cfg_attr(
    feature = "simulated_output",
    allow(dead_code, unused_imports, unused_variables, unused_mut)
)]

use anyhow::{Result, anyhow, bail};
use evdev::{EventSummary, EventType, InputEvent, RelativeAxisCode};
use log::info;
use parking_lot::Mutex;
use std::convert::TryFrom;
use std::sync::Arc;
use std::sync::mpsc::SyncSender as Sender;

use super::mouse_axis::{AXIS_COUNT, AxisState, AxisValues, MouseAxis};
use super::*;

impl Kanata {
    /// Enter an infinite loop that listens for OS key events and sends them to the processing
    /// thread.
    pub fn event_loop(kanata: Arc<Mutex<Self>>, tx: Sender<KeyEvent>) -> Result<()> {
        info!("entering the event loop");

        let k = kanata.lock();
        let allow_hardware_repeat = k.allow_hardware_repeat;
        let mouse_movement_key = k.mouse_movement_key.clone();
        let mouse_axis_map = k.mouse_axis_map.clone();
        let mut kbd_in = match KbdIn::new(
            &k.kbd_in_paths,
            k.continue_if_no_devices,
            k.include_names.clone(),
            k.exclude_names.clone(),
            k.device_detect_mode,
        ) {
            Ok(kbd_in) => kbd_in,
            Err(e) => {
                bail!("failed to open keyboard device(s): {}", e)
            }
        };

        // In some environments, this needs to be done after the input device grab otherwise it
        // does not work on kanata startup.
        Kanata::set_repeat_rate(k.x11_repeat_rate)?;
        drop(k);
        let mut axis_state = AxisState::default();

        loop {
            let events = kbd_in.read().map_err(|e| anyhow!("failed read: {}", e))?;
            log::trace!("event count: {}\nevents:\n{events:?}", events.len());

            for report in events.split_inclusive(|ev| {
                ev.event_type() == EventType::SYNCHRONIZATION && ev.code() == 0
            }) {
                let map = *mouse_axis_map.lock();
                // Keep identity passthrough native. Reconfigure even when
                // disabled, so old fractions cannot leak through a later map.
                if map.is_identity() {
                    axis_state.apply(map, [0; AXIS_COUNT]);
                }
                let mut transformed = if !map.is_identity() {
                    Some(map_axis_report(
                        &mut axis_state,
                        map,
                        report,
                        |code| MAPPED_KEYS.lock().contains(&code),
                        |code| {
                            tx.try_send(KeyEvent::new(code, KeyValue::Tap))?;
                            Ok(())
                        },
                    )?)
                } else {
                    None
                };
                for in_event in report.iter().copied() {
                    if let Some(ms_mvmt_key) = *mouse_movement_key.lock()
                        && let EventSummary::RelativeAxis(
                            _,
                            RelativeAxisCode::REL_X | RelativeAxisCode::REL_Y,
                            _,
                        ) = in_event.destructure()
                    {
                        let fake_event = KeyEvent::new(ms_mvmt_key, KeyValue::Tap);
                        if let Err(e) = tx.try_send(fake_event) {
                            bail!("failed to send on channel: {}", e)
                        }
                    }

                    if !map.is_identity() && relative_axis(in_event).is_some() {
                        if let Some(output) = transformed.take() {
                            let mut kanata = kanata.lock();
                            #[cfg(not(feature = "simulated_output"))]
                            for event in output {
                                kanata.kbd_out.write_raw(event)?;
                            }
                        }
                        continue;
                    }

                    let key_event = match KeyEvent::try_from(in_event) {
                        Ok(ev) => ev,
                        _ => {
                            // Pass-through non-key and non-scroll events
                            let mut kanata = kanata.lock();
                            #[cfg(not(feature = "simulated_output"))]
                            kanata
                                .kbd_out
                                .write_raw(in_event)
                                .map_err(|e| anyhow!("failed write: {}", e))?;
                            continue;
                        }
                    };

                    check_for_exit(&key_event);

                    if key_event.value == KeyValue::Repeat && !allow_hardware_repeat {
                        continue;
                    }

                    if key_event.value == KeyValue::Tap {
                        // Scroll event for sure. Only scroll events produce Tap.
                        if !handle_scroll(&kanata, in_event, key_event.code, report)? {
                            continue;
                        }
                    }

                    match key_event.value {
                        KeyValue::Release => {
                            PRESSED_KEYS.lock().remove(&key_event.code);
                        }
                        KeyValue::Press => {
                            PRESSED_KEYS.lock().insert(key_event.code);
                        }
                        _ => {}
                    }

                    // Handle normal keypresses.
                    // Check if this keycode is mapped in the configuration.
                    // If it hasn't been mapped, send it immediately.
                    if !MAPPED_KEYS.lock().contains(&key_event.code) {
                        let mut kanata = kanata.lock();
                        #[cfg(not(feature = "simulated_output"))]
                        kanata
                            .kbd_out
                            .write_raw(in_event)
                            .map_err(|e| anyhow!("failed write: {}", e))?;
                        continue;
                    };

                    // Send key events to the processing loop
                    if let Err(e) = tx.try_send(key_event) {
                        bail!("failed to send on channel: {}", e)
                    }
                }
            }
        }
    }

    pub fn check_release_non_physical_shift(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn set_repeat_rate(s: Option<KeyRepeatSettings>) -> Result<()> {
        if let Some(s) = s {
            log::info!(
                "Using xset to set X11 repeat delay to {} and repeat rate to {}",
                s.delay,
                s.rate,
            );
            let cmd_output = std::process::Command::new("xset")
                .args([
                    "r",
                    "rate",
                    s.delay.to_string().as_str(),
                    s.rate.to_string().as_str(),
                ])
                .output()
                .map_err(|e| {
                    log::error!("failed to run xset: {e:?}");
                    e
                })?;
            log::info!(
                "xset stdout: {}",
                String::from_utf8_lossy(&cmd_output.stdout)
            );
            log::info!(
                "xset stderr: {}",
                String::from_utf8_lossy(&cmd_output.stderr)
            );
        }
        Ok(())
    }
}

fn relative_axis(event: InputEvent) -> Option<(MouseAxis, bool)> {
    let EventSummary::RelativeAxis(_, code, _) = event.destructure() else {
        return None;
    };
    use MouseAxis::*;
    Some(match code {
        RelativeAxisCode::REL_X => (X, false),
        RelativeAxisCode::REL_Y => (Y, false),
        RelativeAxisCode::REL_Z => (Z, false),
        RelativeAxisCode::REL_RX => (Rx, false),
        RelativeAxisCode::REL_RY => (Ry, false),
        RelativeAxisCode::REL_RZ => (Rz, false),
        RelativeAxisCode::REL_DIAL => (Dial, false),
        RelativeAxisCode::REL_MISC => (Misc, false),
        RelativeAxisCode::REL_WHEEL => (V, false),
        RelativeAxisCode::REL_HWHEEL => (H, false),
        RelativeAxisCode::REL_WHEEL_HI_RES => (V, true),
        RelativeAxisCode::REL_HWHEEL_HI_RES => (H, true),
        _ => return None,
    })
}

fn axis_code(axis: MouseAxis) -> RelativeAxisCode {
    use MouseAxis::*;
    match axis {
        X => RelativeAxisCode::REL_X,
        Y => RelativeAxisCode::REL_Y,
        Z => RelativeAxisCode::REL_Z,
        Rx => RelativeAxisCode::REL_RX,
        Ry => RelativeAxisCode::REL_RY,
        Rz => RelativeAxisCode::REL_RZ,
        Dial => RelativeAxisCode::REL_DIAL,
        Misc => RelativeAxisCode::REL_MISC,
        V => RelativeAxisCode::REL_WHEEL_HI_RES,
        H => RelativeAxisCode::REL_HWHEEL_HI_RES,
    }
}

fn map_axis_report(
    state: &mut AxisState,
    map: MouseAxisMap,
    report: &[InputEvent],
    is_mapped: impl Fn(OsCode) -> bool,
    tap: impl FnMut(OsCode) -> Result<()>,
) -> Result<Vec<InputEvent>> {
    let mut input: AxisValues = [0; AXIS_COUNT];
    let mut high_resolution = [false; AXIS_COUNT];
    for event in report {
        if let Some((axis, true)) = relative_axis(*event) {
            high_resolution[axis as usize] = true;
        }
    }
    for event in report {
        let Some((axis, high_res)) = relative_axis(*event) else {
            continue;
        };
        let mut delta = event.value();
        if axis.is_wheel() && !high_res {
            // Deduplicate each wheel separately, within this report. A hi-res
            // vertical event must not hide a legacy horizontal wheel event.
            if high_resolution[axis as usize] {
                continue;
            }
            delta = delta.saturating_mul(120);
        }
        input[axis as usize] = input[axis as usize].saturating_add(delta);
    }
    let mut output = state.apply(map, input);
    state.route_wheels(&mut output, is_mapped, tap)?;
    let mut events = Vec::new();
    for axis in MouseAxis::ALL {
        let value = output[axis as usize];
        if value != 0 {
            events.push(InputEvent::new(
                EventType::RELATIVE.0,
                axis_code(axis).0,
                value,
            ));
        }
    }
    for (index, axis) in MouseAxis::WHEELS.into_iter().enumerate() {
        let notches = state.wheel_notches(index, output[axis as usize]);
        if notches != 0 {
            let code = if axis == MouseAxis::V {
                RelativeAxisCode::REL_WHEEL
            } else {
                RelativeAxisCode::REL_HWHEEL
            };
            events.push(InputEvent::new(EventType::RELATIVE.0, code.0, notches));
        }
    }
    Ok(events)
}

/// Returns true if the scroll event should be sent to the processing loop, otherwise returns
/// false.
fn handle_scroll(
    kanata: &Mutex<Kanata>,
    in_event: InputEvent,
    code: OsCode,
    all_events: &[InputEvent],
) -> Result<bool> {
    let direction: MWheelDirection = code.try_into().unwrap();
    let scroll_distance = in_event.value().unsigned_abs() as u16;
    match in_event.destructure() {
        EventSummary::RelativeAxis(_, axis_type, _) => {
            match axis_type {
                RelativeAxisCode::REL_WHEEL | RelativeAxisCode::REL_HWHEEL => {
                    if MAPPED_KEYS.lock().contains(&code) {
                        return Ok(true);
                    }
                    // If we just used `write_raw` here, some of the scrolls issued by kanata would be
                    // REL_WHEEL_HI_RES + REL_WHEEL and some just REL_WHEEL and an issue like this one
                    // would happen: https://github.com/jtroo/kanata/issues/395
                    //
                    // So to fix this case, we need to use `scroll` which will also send hi-res scrolls
                    // along normal scrolls.
                    //
                    // However, if this is a normal scroll event, it may be sent alongside a hi-res
                    // scroll event. In this scenario, the hi-res event should be used to call
                    // scroll, and not the normal event. Otherwise, too much scrolling will happen.
                    let mut kanata = kanata.lock();
                    if !all_events.iter().any(|ev| {
                        matches!(
                            ev.destructure(),
                            EventSummary::RelativeAxis(
                                _,
                                RelativeAxisCode::REL_WHEEL_HI_RES
                                    | RelativeAxisCode::REL_HWHEEL_HI_RES,
                                _
                            )
                        )
                    }) {
                        kanata
                            .kbd_out
                            .scroll(direction, scroll_distance * HI_RES_SCROLL_UNITS_IN_LO_RES)
                            .map_err(|e| anyhow!("failed write: {}", e))?;
                    }
                    Ok(false)
                }
                RelativeAxisCode::REL_WHEEL_HI_RES | RelativeAxisCode::REL_HWHEEL_HI_RES => {
                    if !MAPPED_KEYS.lock().contains(&code) {
                        // Passthrough if the scroll wheel event is not mapped
                        // in the configuration.
                        let mut kanata = kanata.lock();
                        kanata
                            .kbd_out
                            .scroll(direction, scroll_distance)
                            .map_err(|e| anyhow!("failed write: {}", e))?;
                    }
                    // Kanata will not handle high resolution scroll events for now.
                    // Full notch scrolling only.
                    Ok(false)
                }
                _ => unreachable!("expect to be handling a wheel event"),
            }
        }
        _ => unreachable!("expect to be handling a wheel event"),
    }
}

#[cfg(test)]
mod axis_tests {
    use super::*;

    fn rel(code: RelativeAxisCode, value: i32) -> InputEvent {
        InputEvent::new(EventType::RELATIVE.0, code.0, value)
    }

    fn value(events: &[InputEvent], code: RelativeAxisCode) -> i32 {
        events
            .iter()
            .filter(|ev| ev.code() == code.0)
            .map(InputEvent::value)
            .sum()
    }

    #[test]
    fn two_wheels_use_separate_resolution_and_preserve_magnitude() {
        let mut map = MouseAxisMap::default();
        map.rows[MouseAxis::X as usize][MouseAxis::H as usize] = 0.25;
        map.rows[MouseAxis::Y as usize][MouseAxis::V as usize] = -0.25;
        let events = map_axis_report(
            &mut AxisState::default(),
            map,
            &[
                rel(RelativeAxisCode::REL_WHEEL, 1),
                rel(RelativeAxisCode::REL_WHEEL_HI_RES, 120),
                rel(RelativeAxisCode::REL_HWHEEL, 2),
            ],
            |_| false,
            |_| unreachable!(),
        )
        .unwrap();
        assert_eq!(value(&events, RelativeAxisCode::REL_X), 60);
        assert_eq!(value(&events, RelativeAxisCode::REL_Y), -30);
        assert_eq!(value(&events, RelativeAxisCode::REL_WHEEL_HI_RES), 120);
        assert_eq!(value(&events, RelativeAxisCode::REL_HWHEEL_HI_RES), 240);
    }

    #[test]
    fn extra_inputs_and_outputs_survive_the_linux_adapter() {
        let mut map = MouseAxisMap::default();
        map.rows[MouseAxis::X as usize][MouseAxis::Z as usize] = 2.0;
        map.rows[MouseAxis::V as usize][MouseAxis::Dial as usize] = 120.0;
        map.rows[MouseAxis::Rz as usize][MouseAxis::H as usize] = -1.0;
        let events = map_axis_report(
            &mut AxisState::default(),
            map,
            &[
                rel(RelativeAxisCode::REL_Z, 3),
                rel(RelativeAxisCode::REL_DIAL, 1),
                rel(RelativeAxisCode::REL_HWHEEL_HI_RES, 60),
            ],
            |_| false,
            |_| unreachable!(),
        )
        .unwrap();
        assert_eq!(value(&events, RelativeAxisCode::REL_X), 6);
        assert_eq!(value(&events, RelativeAxisCode::REL_Z), 3);
        assert_eq!(value(&events, RelativeAxisCode::REL_DIAL), 1);
        assert_eq!(value(&events, RelativeAxisCode::REL_WHEEL), 1);
        assert_eq!(value(&events, RelativeAxisCode::REL_RZ), -60);
    }

    #[test]
    fn report_mixing_cancels_before_wheel_keys_fire() {
        let mut map = MouseAxisMap::default();
        map.rows[MouseAxis::V as usize][MouseAxis::X as usize] = 120.0;
        map.rows[MouseAxis::V as usize][MouseAxis::Y as usize] = -120.0;
        map_axis_report(
            &mut AxisState::default(),
            map,
            &[
                rel(RelativeAxisCode::REL_X, 1),
                rel(RelativeAxisCode::REL_Y, 1),
            ],
            |_| true,
            |_| panic!("equal contributions must cancel"),
        )
        .unwrap();
    }
}
