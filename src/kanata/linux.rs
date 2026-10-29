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

use super::mouse_axis::{AXIS_COUNT, AxisState, AxisValues, MotionSnapshot, MouseAxis, WheelMask};
use super::*;

impl Kanata {
    /// Enter an infinite loop that listens for OS key events and sends them to the processing
    /// thread.
    pub fn event_loop(kanata: Arc<Mutex<Self>>, tx: Sender<KeyEvent>) -> Result<()> {
        info!("entering the event loop");

        let mut k = kanata.lock();
        let allow_hardware_repeat = k.allow_hardware_repeat;
        let mouse_movement_key = k.mouse_movement_key.clone();
        let motion_snapshot = k.motion_snapshot.clone();
        let plan = k.motion_capture.plan.clone();
        let CapturePlan::Linux(ref linux_plan) = plan else {
            unreachable!("Linux startup capture plan")
        };
        let mut kbd_in = match KbdIn::new(
            &linux_plan.paths,
            linux_plan.continue_if_no_devices,
            linux_plan.include.clone(),
            linux_plan.exclude.clone(),
            linux_plan.mode,
        ) {
            Ok(kbd_in) => kbd_in,
            Err(e) => {
                bail!("failed to open keyboard device(s): {}", e)
            }
        };

        // In some environments, this needs to be done after the input device grab otherwise it
        // does not work on kanata startup.
        Kanata::set_repeat_rate(k.x11_repeat_rate)?;
        k.motion_capture.installed(plan.capabilities());
        drop(k);
        let mut axis_state = AxisState::default();

        loop {
            let events = kbd_in.read().map_err(|e| anyhow!("failed read: {}", e))?;
            log::trace!("event count: {}\nevents:\n{events:?}", events.len());

            for report in events.split_inclusive(|ev| {
                ev.event_type() == EventType::SYNCHRONIZATION && ev.code() == 0
            }) {
                let snapshot = motion_snapshot.lock().clone();
                let mut axes = route_axis_report(&mut axis_state, &snapshot, report, |code| {
                    tx.try_send(KeyEvent::new(code, KeyValue::Tap))?;
                    Ok(())
                })?;
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

                    if let Some(output) = axes.take_axis_output(in_event) {
                        let mut kanata = kanata.lock();
                        #[cfg(not(feature = "simulated_output"))]
                        for event in output {
                            kanata.kbd_out.write_raw(event)?;
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
                        if !handle_scroll(
                            &kanata,
                            in_event,
                            key_event.code,
                            report,
                            snapshot.wheels,
                        )? {
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
                    if !WheelMask::is_wheel(key_event.code)
                        && !MAPPED_KEYS.lock().contains(&key_event.code)
                    {
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

enum AxisReportRoute {
    Legacy,
    Analog(Vec<InputEvent>),
}

impl AxisReportRoute {
    /// Replace known relative axes at their first position in the report. Other
    /// events (including unknown relative codes) retain their native routing.
    fn take_axis_output(&mut self, event: InputEvent) -> Option<Vec<InputEvent>> {
        match self {
            Self::Analog(output) if relative_axis(event).is_some() => Some(std::mem::take(output)),
            _ => None,
        }
    }
}

/// The production decision between analog accounting and the legacy event path.
fn route_axis_report(
    state: &mut AxisState,
    snapshot: &MotionSnapshot,
    report: &[InputEvent],
    tap: impl FnMut(OsCode) -> Result<()>,
) -> Result<AxisReportRoute> {
    state.observe_snapshot(snapshot);
    if !snapshot.pipeline_enabled {
        Ok(AxisReportRoute::Legacy)
    } else {
        map_axis_report(state, snapshot, report, tap).map(AxisReportRoute::Analog)
    }
}

fn map_axis_report(
    state: &mut AxisState,
    snapshot: &MotionSnapshot,
    report: &[InputEvent],
    tap: impl FnMut(OsCode) -> Result<()>,
) -> Result<Vec<InputEvent>> {
    state.observe_snapshot(snapshot);
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
    let mut output = state.apply(input);
    state.route_wheels(&mut output, snapshot.wheels, tap)?;
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
    wheels: WheelMask,
) -> Result<bool> {
    let direction: MWheelDirection = code.try_into().unwrap();
    let scroll_distance = in_event.value().unsigned_abs() as u16;
    match in_event.destructure() {
        EventSummary::RelativeAxis(_, axis_type, _) => {
            match axis_type {
                RelativeAxisCode::REL_WHEEL | RelativeAxisCode::REL_HWHEEL => {
                    if wheels.contains(code) {
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
                    if !wheels.contains(code) {
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
    use super::super::mouse_axis::MouseAxisMap;
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

    fn routed_report(
        state: &mut AxisState,
        snapshot: &MotionSnapshot,
        report: &[InputEvent],
        taps: &mut Vec<OsCode>,
    ) -> Vec<InputEvent> {
        let mut route = route_axis_report(state, snapshot, report, |code| {
            taps.push(code);
            Ok(())
        })
        .unwrap();
        report
            .iter()
            .flat_map(|&event| route.take_axis_output(event).unwrap_or_else(|| vec![event]))
            .collect()
    }

    #[cfg(feature = "simulated_output")]
    #[test]
    fn native_wheel_credit_crosses_identity_precision_in_both_directions() {
        let _lock = crate::tests::CFG_PARSE_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        for layers in [[0, 1], [1, 0]] {
            let mut k = Kanata::new_from_str(
                "(defsrc mwu mwr) (deflayer base a b) (deflayer (precision mouse-axis ((mul-elements (scalar 0.25 (outputs x y))))) _ _)",
                Default::default(),
            )
            .unwrap();
            let mut state = AxisState::default();
            let mut taps = vec![];
            for layer in layers {
                k.layout.bm().set_default_layer(layer);
                k.tick_ms(0, &None).unwrap();
                let snapshot = k.motion_snapshot.lock().clone();
                // Exercise the same routing decision as event_loop, not the
                // unconditional mapper. Legacy mapped hi-res events emit no tap.
                route_axis_report(
                    &mut state,
                    &snapshot,
                    &[
                        rel(RelativeAxisCode::REL_WHEEL_HI_RES, 60),
                        rel(RelativeAxisCode::REL_HWHEEL_HI_RES, 60),
                    ],
                    |code| {
                        taps.push(code);
                        Ok(())
                    },
                )
                .unwrap();
            }
            assert_eq!(
                taps,
                [OsCode::MouseWheelUp, OsCode::MouseWheelRight],
                "half-notches across layers {layers:?} must make one notch per wheel"
            );
        }
    }

    #[test]
    fn native_wheel_toggles_cannot_revive_key_credit_or_lose_multiple_notches() {
        for sign in [-1, 1] {
            let mut snapshot = Arc::new(MotionSnapshot::initial(
                MouseAxisMap::default(),
                WheelMask::new(|_| true),
                true,
            ));
            let mut state = AxisState::default();
            // End each pair at zero credit, then revisit both maps without new
            // wheel input: no dormant identity-path credit may reappear.
            for (index, (delta, notches)) in [(60, 0), (60, 1), (300, 2), (60, 1), (0, 0), (0, 0)]
                .into_iter()
                .enumerate()
            {
                let mut map = MouseAxisMap::default();
                map.rows[0][0] = if index % 2 == 0 { 0.25 } else { 1.0 };
                snapshot = snapshot.next(map, snapshot.wheels, true, false);
                let mut taps = vec![];
                let output = routed_report(
                    &mut state,
                    &snapshot,
                    &[
                        rel(RelativeAxisCode::REL_WHEEL_HI_RES, sign * delta),
                        rel(RelativeAxisCode::REL_HWHEEL_HI_RES, sign * delta),
                    ],
                    &mut taps,
                );
                assert!(output.is_empty(), "mapped wheels must not leak raw output");
                let expected: Vec<_> = MouseAxis::WHEELS
                    .into_iter()
                    .flat_map(|axis| vec![axis.wheel_code(sign > 0).unwrap(); notches])
                    .collect();
                assert_eq!(taps, expected, "report {index}, sign {sign}");
            }
            assert_eq!(state.wheel_key_remainder, [0; 2]);
        }
    }

    #[test]
    fn native_identity_precision_raw_companions_and_metadata_stay_consistent() {
        let mut snapshot = Arc::new(MotionSnapshot::initial(
            MouseAxisMap::default(),
            WheelMask::default(),
            true,
        ));
        let mut state = AxisState::default();
        let metadata = InputEvent::new(EventType::MISC.0, 4, 101);
        let unknown_relative = InputEvent::new(EventType::RELATIVE.0, 0x0f, 9);
        let syn = InputEvent::new(EventType::SYNCHRONIZATION.0, 0, 0);
        for (index, (delta, notches)) in [(60, 0), (60, 1), (300, 2), (60, 1)]
            .into_iter()
            .enumerate()
        {
            let mut map = MouseAxisMap::default();
            map.rows[0][0] = if index % 2 == 0 { 1.0 } else { 0.25 };
            snapshot = snapshot.next(map, snapshot.wheels, true, false);
            let mut report = vec![
                metadata,
                rel(RelativeAxisCode::REL_X, 8),
                rel(RelativeAxisCode::REL_Y, 4),
                rel(RelativeAxisCode::REL_WHEEL_HI_RES, delta),
                rel(RelativeAxisCode::REL_HWHEEL_HI_RES, -delta),
                unknown_relative,
                syn,
            ];
            if notches != 0 {
                report.insert(3, rel(RelativeAxisCode::REL_WHEEL, notches));
                report.insert(4, rel(RelativeAxisCode::REL_HWHEEL, -notches));
            }
            let output = routed_report(&mut state, &snapshot, &report, &mut vec![]);
            assert_eq!(value(&output, RelativeAxisCode::REL_WHEEL_HI_RES), delta);
            assert_eq!(value(&output, RelativeAxisCode::REL_HWHEEL_HI_RES), -delta);
            assert_eq!(value(&output, RelativeAxisCode::REL_WHEEL), notches);
            assert_eq!(value(&output, RelativeAxisCode::REL_HWHEEL), -notches);
            assert_eq!(
                value(&output, RelativeAxisCode::REL_X),
                if index % 2 == 0 { 8 } else { 2 }
            );
            assert_eq!(value(&output, RelativeAxisCode::REL_Y), 4);
            for untouched in [metadata, unknown_relative, syn] {
                assert_eq!(output.iter().filter(|&&ev| ev == untouched).count(), 1);
            }
        }
        assert_eq!(state.wheel_raw_remainder, [0; 2]);
        // Without hi-res companions, legacy magnitudes are still normalized once.
        let output = routed_report(
            &mut state,
            &snapshot,
            &[
                rel(RelativeAxisCode::REL_WHEEL, 2),
                rel(RelativeAxisCode::REL_HWHEEL, -3),
            ],
            &mut vec![],
        );
        assert_eq!(value(&output, RelativeAxisCode::REL_WHEEL_HI_RES), 240);
        assert_eq!(value(&output, RelativeAxisCode::REL_HWHEEL_HI_RES), -360);
        assert_eq!(value(&output, RelativeAxisCode::REL_WHEEL), 2);
        assert_eq!(value(&output, RelativeAxisCode::REL_HWHEEL), -3);
    }

    #[test]
    fn legacy_identity_report_keeps_native_routing_but_observes_generation() {
        let enabled = MotionSnapshot::initial(MouseAxisMap::default(), WheelMask::default(), true);
        let mut state = AxisState::default();
        state.observe_snapshot(&enabled);
        state.wheel_key_remainder = [60; 2];
        state.wheel_raw_remainder = [60; 2];
        let legacy = enabled.next(enabled.map, enabled.wheels, false, true);
        let event = rel(RelativeAxisCode::REL_WHEEL_HI_RES, 60);
        let mut route =
            route_axis_report(&mut state, &legacy, &[event], |_| unreachable!()).unwrap();
        assert!(matches!(route, AxisReportRoute::Legacy));
        assert!(route.take_axis_output(event).is_none());
        assert_eq!(state.wheel_key_remainder, [0; 2]);
        assert_eq!(state.wheel_raw_remainder, [0; 2]);
    }

    #[test]
    fn two_wheels_use_separate_resolution_and_preserve_magnitude() {
        let mut map = MouseAxisMap::default();
        map.rows[MouseAxis::X as usize][MouseAxis::H as usize] = 0.25;
        map.rows[MouseAxis::Y as usize][MouseAxis::V as usize] = -0.25;
        let events = map_axis_report(
            &mut AxisState::default(),
            &MotionSnapshot::initial(map, WheelMask::default(), true),
            &[
                rel(RelativeAxisCode::REL_WHEEL, 1),
                rel(RelativeAxisCode::REL_WHEEL_HI_RES, 120),
                rel(RelativeAxisCode::REL_HWHEEL, 2),
            ],
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
            &MotionSnapshot::initial(map, WheelMask::default(), true),
            &[
                rel(RelativeAxisCode::REL_Z, 3),
                rel(RelativeAxisCode::REL_DIAL, 1),
                rel(RelativeAxisCode::REL_HWHEEL_HI_RES, 60),
            ],
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
            &MotionSnapshot::initial(map, WheelMask::new(|_| true), true),
            &[
                rel(RelativeAxisCode::REL_X, 1),
                rel(RelativeAxisCode::REL_Y, 1),
            ],
            |_| panic!("equal contributions must cancel"),
        )
        .unwrap();
    }

    #[test]
    fn directional_wheel_up_moves_pointer_and_down_remains_scroll() {
        use kanata_parser::mouse_axis::MouseAxisMapMode;
        let mut map = MouseAxisMap {
            mode: MouseAxisMapMode::Directional,
            ..Default::default()
        };
        map.rows[MouseAxis::X as usize][MouseAxis::V as usize] = 0.25;
        map.rows[MouseAxis::V as usize] = [0.0; AXIS_COUNT];
        let mut state = AxisState::default();
        let up = map_axis_report(
            &mut state,
            &MotionSnapshot::initial(map, WheelMask::default(), true),
            &[rel(RelativeAxisCode::REL_WHEEL_HI_RES, 120)],
            |_| unreachable!(),
        )
        .unwrap();
        assert_eq!(value(&up, RelativeAxisCode::REL_X), 30);
        assert_eq!(value(&up, RelativeAxisCode::REL_WHEEL_HI_RES), 0);
        let down = map_axis_report(
            &mut state,
            &MotionSnapshot::initial(map, WheelMask::default(), true),
            &[rel(RelativeAxisCode::REL_WHEEL_HI_RES, -120)],
            |_| unreachable!(),
        )
        .unwrap();
        assert_eq!(value(&down, RelativeAxisCode::REL_X), 0);
        assert_eq!(value(&down, RelativeAxisCode::REL_WHEEL_HI_RES), -120);
    }
}
