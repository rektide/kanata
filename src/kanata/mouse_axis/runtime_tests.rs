//! Real layout/tick/reload paths; simulated output only, never native devices.
use super::capture::{CaptureCapabilities, CapturePlan, CaptureStatus, WindowsCapture};
use super::*;
use crate::kanata::{FAKE_KEY_ROW, FakeKeyAction, handle_fakekey_action};
use crate::oskbd::{KeyEvent, KeyValue};

const STACK: &str = r"
(defsrc a b c d)
(defvirtualkeys virtual-low (layer-while-held low))
(deflayer (base mouse-axis ((set (cell x y 2))))
  (layer-while-held low) (layer-while-held high) (layer-while-held low) (layer-while-held empty))
(deflayer (low mouse-axis ((mul-elements (scalar 2 (outputs x))))) _ _ _ _)
(deflayer (high mouse-axis ((add (cell x y 1)))) _ _ _ _)
(deflayer (alternate mouse-axis ((set (cell x y 10)))) _ _ _ _)
(deflayer (empty mouse-axis ((set (cell x y null)))) _ _ _ _)
";

fn new(cfg: &str) -> Kanata {
    Kanata::new_from_str(cfg, Default::default()).unwrap()
}
fn snapshot(k: &Kanata) -> Arc<MotionSnapshot> {
    k.motion_snapshot.lock().clone()
}
fn coefficient(k: &Kanata) -> f64 {
    snapshot(k).map.rows[0][1]
}
fn key(k: &mut Kanata, code: OsCode, value: KeyValue) {
    k.handle_input_event(&KeyEvent::new(code, value)).unwrap();
    k.tick_ms(2, &None).unwrap();
}

#[test]
fn native_pipeline_policy_follows_declarations_not_current_map_and_resets_on_reload() {
    let _lock = crate::tests::CFG_PARSE_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    for (cfg, enabled) in [
        ("(defsrc) (deflayer base)", false),
        (
            "(defcfg mouse-axis-map ((x (x 1)))) (defsrc) (deflayer base)",
            false,
        ),
        (
            "(defsrc) (deflayer (base mouse-axis ((set (cell x y null)))))",
            false,
        ),
        (
            "(defcfg mouse-axis-map ((x (x 0.5)))) (defsrc) (deflayer base)",
            true,
        ),
        (
            "(defsrc) (deflayer base) (deflayer (inactive mouse-axis ((add (cell x y 0)))))",
            true,
        ),
        (
            "(defsrc) (deflayer base) (deflayer (inactive mouse-axis ((compose-output (factor (from x) (to x) (values (1)))))))",
            true,
        ),
    ] {
        let k = new(cfg);
        assert_eq!(snapshot(&k).pipeline_enabled, enabled, "{cfg}");
    }

    let enabled = "(defsrc) (deflayer base) (deflayer (precision mouse-axis ((mul-elements (scalar 0.25 (outputs x y))))))";
    let mut k = new(enabled);
    let base = snapshot(&k);
    assert!(base.map.is_identity() && base.pipeline_enabled);
    k.layout.bm().set_default_layer(1);
    k.tick_ms(0, &None).unwrap();
    let precision = snapshot(&k);
    assert!(!precision.map.is_identity() && precision.pipeline_enabled);
    assert_eq!(
        precision.output_epochs[MouseAxis::V as usize],
        base.output_epochs[MouseAxis::V as usize]
    );
    assert_eq!(
        precision.output_epochs[MouseAxis::H as usize],
        base.output_epochs[MouseAxis::H as usize]
    );

    let mut device = AxisState::default();
    device.observe_snapshot(&precision);
    device.wheel_key_remainder = [60; 2];
    device.wheel_raw_remainder = [60; 2];
    for (cfg, expected_enabled) in [("(defsrc) (deflayer base)", false), (enabled, true)] {
        let old = snapshot(&k);
        let candidate = kanata_parser::cfg::new_from_str(cfg, Default::default()).unwrap();
        k.install_config(candidate, &None).unwrap();
        let current = snapshot(&k);
        assert_eq!(current.pipeline_enabled, expected_enabled);
        assert!(current.map.is_identity());
        assert_eq!(current.generation, old.generation + 1);
        device.observe_snapshot(&current);
        assert_eq!(device.wheel_key_remainder, [0; 2]);
        assert_eq!(device.wheel_raw_remainder, [0; 2]);
        device.wheel_key_remainder = [30; 2];
        device.wheel_raw_remainder = [30; 2];
    }
}

#[test]
fn full_stack_handles_lower_release_duplicate_priority_defaults_and_no_drift() {
    let _lock = crate::tests::CFG_PARSE_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    for behavior in ["layer-stack", "to-base-layer"] {
        let mut k = new(&format!(
            "(defcfg delegate-to-first-layer yes transparent-key-resolution {behavior}) {STACK}"
        ));
        assert_eq!(coefficient(&k), 2.0);
        key(&mut k, OsCode::KEY_A, KeyValue::Press);
        assert_eq!(coefficient(&k), 4.0);
        key(&mut k, OsCode::KEY_B, KeyValue::Press);
        assert_eq!(coefficient(&k), 5.0);
        key(&mut k, OsCode::KEY_C, KeyValue::Press);
        assert_eq!(coefficient(&k), 6.0); // M,L: newest duplicate moves L above M
        key(&mut k, OsCode::KEY_C, KeyValue::Release);
        assert_eq!(coefficient(&k), 5.0); // older L survives below M
        k.layout.bm().set_default_layer(1); // same as surviving held L
        k.tick_ms(0, &None).unwrap();
        assert_eq!(snapshot(&k).map.rows[0][0], 2.0); // L once, not twice
        assert_eq!(coefficient(&k), 1.0); // no implicit first-layer program
        k.layout.bm().set_default_layer(0);
        k.tick_ms(0, &None).unwrap();
        let top = k.layout.bm().current_layer();
        key(&mut k, OsCode::KEY_A, KeyValue::Release);
        assert_eq!(k.layout.bm().current_layer(), top);
        assert_eq!(coefficient(&k), 3.0);
        k.layout.bm().set_default_layer(3);
        k.tick_ms(0, &None).unwrap();
        assert_eq!(coefficient(&k), 11.0); // external default under unchanged high
        let stable = snapshot(&k);
        k.tick_ms(100, &None).unwrap();
        assert!(Arc::ptr_eq(&stable, &snapshot(&k)));
    }
}

#[test]
fn queued_virtual_holder_applies_once_and_null_layers_preserve_credit() {
    let _lock = crate::tests::CFG_PARSE_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let mut k = new(STACK);
    let idx = k.virtual_keys["virtual-low"] as u16;
    handle_fakekey_action(FakeKeyAction::Press, k.layout.bm(), FAKE_KEY_ROW, idx);
    k.tick_ms(0, &None).unwrap();
    assert_eq!(coefficient(&k), 2.0); // queued is not processed state
    k.tick_ms(2, &None).unwrap();
    assert_eq!(coefficient(&k), 4.0);
    key(&mut k, OsCode::KEY_A, KeyValue::Press);
    assert_eq!(coefficient(&k), 4.0);
    handle_fakekey_action(FakeKeyAction::Release, k.layout.bm(), FAKE_KEY_ROW, idx);
    k.tick_ms(2, &None).unwrap();
    assert_eq!(coefficient(&k), 4.0); // physical holder survives
    let before = snapshot(&k);
    key(&mut k, OsCode::KEY_D, KeyValue::Press);
    assert_eq!(snapshot(&k).output_epochs, before.output_epochs);
    key(&mut k, OsCode::KEY_A, KeyValue::Release);
    assert_eq!(coefficient(&k), 2.0);
}

#[test]
fn all_inner_transitions_publish_even_without_reports_between_them() {
    let _lock = crate::tests::CFG_PARSE_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let mut k = new(
        "(defsrc a) (deflayer base (one-shot 2 (layer-while-held precision))) (deflayer (precision mouse-axis ((mul-elements (scalar 0.25 (outputs x y))))) _)",
    );
    let before = snapshot(&k);
    let mut device = AxisState::default();
    device.observe_snapshot(&before);
    device.transform.remainder[0] = 0.75;
    k.handle_input_event(&KeyEvent::new(OsCode::KEY_A, KeyValue::Press))
        .unwrap();
    k.handle_input_event(&KeyEvent::new(OsCode::KEY_A, KeyValue::Release))
        .unwrap();
    k.tick_ms(6, &None).unwrap();
    let after = snapshot(&k);
    assert!(after.map.semantic_eq(&before.map));
    assert_eq!(after.output_epochs[0], before.output_epochs[0] + 2);
    assert_eq!(after.output_epochs[2], before.output_epochs[2]);
    device.observe_snapshot(&after);
    assert_eq!(device.transform.remainder[0], 0.0);
}

#[test]
fn on_load_is_not_predicted_and_invalid_runtime_combinations_do_not_publish() {
    let _lock = crate::tests::CFG_PARSE_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let mut k = new(
        "(defcfg alias-to-trigger-on-load start) (defalias start (on-press press-virtualkey p)) (defvirtualkeys p (layer-while-held precision)) (defsrc) (deflayer base) (deflayer (precision mouse-axis ((mul-elements (scalar 0.25 (outputs x y))))))",
    );
    assert_eq!(snapshot(&k).map.rows[0][0], 1.0);
    k.tick_ms(8, &None).unwrap();
    assert_eq!(snapshot(&k).map.rows[0][0], 0.25);
    let mut k = new(
        "(defsrc a b) (deflayer base (layer-while-held huge) (layer-while-held product)) (deflayer (huge mouse-axis ((set (cell x x 1e308)))) _ _) (deflayer (product mouse-axis ((compose-output (factor (from x) (to x) (values (2)))))) _ _)",
    );
    key(&mut k, OsCode::KEY_A, KeyValue::Press);
    let before = snapshot(&k);
    k.handle_input_event(&KeyEvent::new(OsCode::KEY_B, KeyValue::Press))
        .unwrap();
    let error = k.tick_ms(2, &None).unwrap_err().to_string();
    for expected in [
        "generation 1",
        "product",
        "instruction 1",
        "canonical stack",
        "configuration:",
    ] {
        assert!(error.contains(expected), "{error}");
    }
    assert!(Arc::ptr_eq(&before, &snapshot(&k)));
}

#[test]
fn reload_preflight_failure_is_atomic_and_equal_reload_resets_every_output() {
    let _lock = crate::tests::CFG_PARSE_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let mut k = new("(defsrc a) (deflayer original b)");
    // Exercise Windows capability rules on Linux without installing a hook.
    k.motion_capture = CaptureStatus::pending(CapturePlan::Windows(WindowsCapture::interception(
        true, false,
    )));
    let before = snapshot(&k);
    let candidate = kanata_parser::cfg::new_from_str("(defsrc a) (deflayer changed c) (deflayer (inactive mouse-axis ((mul-elements (scalar 0.25)))) _)", Default::default()).unwrap();
    let err = k.install_config(candidate, &None).unwrap_err().to_string();
    assert!(
        err.contains("MOVE") && err.contains("restart required"),
        "{err}"
    );
    assert_eq!(k.layer_info[0].name, "original");
    assert!(Arc::ptr_eq(&before, &snapshot(&k)));
    assert_eq!(k.motion.stack, vec![0]);
    #[cfg(feature = "tcp_server")]
    assert!(!k.last_reload_ok);
    // An actual installed buttons-only LLHOOK still receives movement, even
    // when the current effective map is identity.
    k.motion_capture
        .installed(CaptureCapabilities::Windows(WindowsCapture::llhook(true)));
    let candidate = kanata_parser::cfg::new_from_str("(defsrc a) (deflayer changed c) (deflayer (inactive mouse-axis ((mul-elements (scalar 0.25)))) _)", Default::default()).unwrap();
    k.install_config(candidate, &None).unwrap();
    let after = snapshot(&k);
    assert!(after.map.semantic_eq(&before.map));
    assert_eq!(after.generation, before.generation + 1);
    assert_eq!(
        after.output_epochs,
        before.output_epochs.map(|epoch| epoch + 1)
    );
    assert_eq!(k.layer_info[0].name, "changed");
    key(&mut k, OsCode::KEY_A, KeyValue::Press);
    assert_eq!(snapshot(&k).generation, after.generation);
}

#[test]
fn candidate_actual_initial_stack_is_preflighted_before_installation() {
    let _lock = crate::tests::CFG_PARSE_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let mut k = new(STACK);
    let before = snapshot(&k);
    let mut candidate = kanata_parser::cfg::new_from_str("(defcfg mouse-axis-map ((x (x 1e308)))) (defsrc) (deflayer base) (deflayer (overflow mouse-axis ((mul-elements (cell x x 2)))))", Default::default()).unwrap();
    candidate.layout.bm().set_default_layer(1);
    let error = k.install_config(candidate, &None).unwrap_err().to_string();
    assert!(
        error.contains("generation 2") && error.contains("overflow"),
        "{error}"
    );
    assert!(Arc::ptr_eq(&before, &snapshot(&k)));
    assert_eq!(k.layer_info[0].name, "base");
    assert_eq!(coefficient(&k), 2.0);
}

#[test]
fn report_wheel_routing_is_snapshot_owned_and_reload_uses_the_new_generation() {
    let _lock = crate::tests::CFG_PARSE_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let mut k = new("(defsrc mwu) (deflayer original a)");
    let before = snapshot(&k);
    assert!(before.wheels.contains(OsCode::MouseWheelUp));
    let candidate =
        kanata_parser::cfg::new_from_str("(defsrc mwd) (deflayer changed b)", Default::default())
            .unwrap();
    k.install_config(candidate, &None).unwrap();
    let after = snapshot(&k);
    assert!(before.wheels.contains(OsCode::MouseWheelUp));
    assert!(!before.wheels.contains(OsCode::MouseWheelDown));
    assert!(!after.wheels.contains(OsCode::MouseWheelUp));
    assert!(after.wheels.contains(OsCode::MouseWheelDown));
    let mut device = AxisState::default();
    device.observe_snapshot(&before);
    let mut output = [0; AXIS_COUNT];
    output[MouseAxis::V as usize] = 240;
    let mut taps = vec![];
    device
        .route_wheels(&mut output, before.wheels, |code| {
            taps.push(code);
            Ok(())
        })
        .unwrap();
    assert_eq!(taps, vec![OsCode::MouseWheelUp; 2]);
    assert_eq!(output, [0; AXIS_COUNT]);
}

#[cfg(any(target_os = "linux", target_os = "android"))]
#[test]
fn pending_startup_installs_the_original_plan_despite_an_early_identity_reload() {
    let _lock = crate::tests::CFG_PARSE_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let mut k =
        new("(defsrc) (deflayer base) (deflayer (inactive mouse-axis ((set (cell x y 2)))))");
    let candidate =
        kanata_parser::cfg::new_from_str("(defsrc) (deflayer new)", Default::default()).unwrap();
    k.install_config(candidate, &None).unwrap();
    assert!(snapshot(&k).map.is_identity());
    let CapturePlan::Linux(ref plan) = k.motion_capture.plan else {
        panic!("Linux plan")
    };
    assert_eq!(plan.mode, kanata_parser::cfg::DeviceDetectMode::Any);
    // The backend consumes this original plan; no device constructor is called here.
    k.motion_capture
        .installed(k.motion_capture.plan.capabilities());
    let candidate = kanata_parser::cfg::new_from_str(
        "(defsrc) (deflayer new) (deflayer (later mouse-axis ((set (cell y x 2)))))",
        Default::default(),
    )
    .unwrap();
    k.install_config(candidate, &None).unwrap();
}

#[test]
fn precision_sample_is_a_complete_layout_to_physical_vector_path() {
    let _lock = crate::tests::CFG_PARSE_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let mut k = new(include_str!("../../../cfg_samples/mouse-axis-layers.kbd"));
    let mut device = AxisState::default();
    let mut input = [0; AXIS_COUNT];
    input[..4].copy_from_slice(&[8, 4, 120, 0]);
    let physical = |k: &Kanata, device: &mut AxisState| {
        device.observe_snapshot(&snapshot(k));
        let output = device.apply(input);
        <[i32; 4]>::try_from(&output[..4]).unwrap()
    };
    assert_eq!(physical(&k, &mut device), [4, -8, 120, 0]);
    key(&mut k, OsCode::KEY_F1, KeyValue::Press);
    assert_eq!(physical(&k, &mut device), [1, -2, 120, 0]);
    key(&mut k, OsCode::KEY_F2, KeyValue::Press);
    key(&mut k, OsCode::KEY_F3, KeyValue::Press);
    assert_eq!(physical(&k, &mut device), [2, -4, 240, 0]);
    key(&mut k, OsCode::KEY_F1, KeyValue::Release);
    assert_eq!(physical(&k, &mut device), [8, -16, 240, 0]);
    key(&mut k, OsCode::KEY_F2, KeyValue::Release);
    key(&mut k, OsCode::KEY_F3, KeyValue::Release);
    assert_eq!(physical(&k, &mut device), [4, -8, 120, 0]);
}
