//! Shared analog-axis state. OS adapters retain report boundaries and normalize
//! wheel units before this transform, then encode the resulting native outputs.

use super::Kanata;
use anyhow::Result;
use kanata_parser::cfg::{Cfg, LayerInfo};
use kanata_parser::keys::OsCode;
#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "windows",
    test
))]
pub(crate) use kanata_parser::mouse_axis::AxisValues;
#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "windows",
    test
))]
use kanata_parser::mouse_axis::MouseAxisTransform;
use kanata_parser::mouse_axis::program::{CaptureRequirements, canonical_layer_order};
pub(crate) use kanata_parser::mouse_axis::{AXIS_COUNT, MouseAxis, MouseAxisMap};
use std::sync::Arc;

pub(crate) mod capture;

/// Routing belongs to the same generation as the coefficients, not to a mutable
/// global key table consulted partway through a native report.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct WheelMask([bool; 4]);

impl WheelMask {
    pub fn new(is_mapped: impl Fn(OsCode) -> bool) -> Self {
        Self(Self::CODES.map(is_mapped))
    }

    const CODES: [OsCode; 4] = [
        OsCode::MouseWheelUp,
        OsCode::MouseWheelDown,
        OsCode::MouseWheelRight,
        OsCode::MouseWheelLeft,
    ];

    #[cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "windows",
        test
    ))]
    pub fn contains(self, code: OsCode) -> bool {
        Self::CODES
            .iter()
            .position(|&c| c == code)
            .is_some_and(|i| self.0[i])
    }

    #[cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "windows",
        test
    ))]
    pub fn is_wheel(code: OsCode) -> bool {
        Self::CODES.contains(&code)
    }
}

/// Immutable, acquired once per native report. Epochs retain discontinuities
/// even if a device is inactive across A -> B -> A. Publication locks are never
/// held while evaluating programs, writing output or sending input events.
#[derive(Clone, Debug)]
pub(crate) struct MotionSnapshot {
    pub map: MouseAxisMap,
    /// Configuration-wide routing policy, stable across layer changes even when
    /// the effective map becomes identity. Legacy identity configs opt out.
    pub pipeline_enabled: bool,
    pub generation: u64,
    pub output_epochs: [u64; AXIS_COUNT],
    pub wheels: WheelMask,
}

impl MotionSnapshot {
    pub fn initial(map: MouseAxisMap, wheels: WheelMask, pipeline_enabled: bool) -> Self {
        Self {
            map,
            pipeline_enabled,
            generation: 1,
            output_epochs: [1; AXIS_COUNT],
            wheels,
        }
    }

    pub fn next(
        &self,
        map: MouseAxisMap,
        wheels: WheelMask,
        pipeline_enabled: bool,
        reload: bool,
    ) -> Arc<Self> {
        debug_assert!(reload || pipeline_enabled == self.pipeline_enabled);
        Arc::new(Self {
            map,
            wheels,
            pipeline_enabled,
            generation: if reload {
                self.generation
                    .checked_add(1)
                    .expect("motion generation exhausted")
            } else {
                self.generation
            },
            output_epochs: std::array::from_fn(|i| {
                if reload || !self.map.output_eq(&map, MouseAxis::ALL[i]) {
                    self.output_epochs[i]
                        .checked_add(1)
                        .expect("motion output epoch exhausted")
                } else {
                    self.output_epochs[i]
                }
            }),
        })
    }
}

pub(crate) struct PreparedMotion {
    baseline: MouseAxisMap,
    stack: Vec<u16>,
    pub map: MouseAxisMap,
    pub wheels: WheelMask,
    pub required: CaptureRequirements,
}

impl PreparedMotion {
    pub fn new(cfg: &mut Cfg, generation: u64) -> Result<Self> {
        let baseline = cfg.options.mouse_axis_map;
        let layout = cfg.layout.bm();
        let stack = canonical_layer_order(layout.active_held_layers(), layout.default_layer as u16);
        let map = resolve(&baseline, &cfg.layer_info, &stack, generation)?;
        log_program_order(&cfg.layer_info, &stack);
        let required = CaptureRequirements::for_programs(
            &baseline,
            cfg.layer_info.iter().map(|l| &l.mouse_axis),
        );
        Ok(Self {
            baseline,
            stack,
            map,
            wheels: WheelMask::new(|c| cfg.mapped_keys.contains(&c)),
            required,
        })
    }
}

pub(crate) struct MotionResolver {
    baseline: MouseAxisMap,
    stack: Vec<u16>,
    current: Arc<MotionSnapshot>,
}

fn resolve(
    baseline: &MouseAxisMap,
    layers: &[LayerInfo],
    stack: &[u16],
    generation: u64,
) -> Result<MouseAxisMap> {
    kanata_parser::cfg::resolve_motion(baseline, layers, stack)
        .map_err(|error| anyhow::anyhow!("motion generation {generation}: {error}"))
}

impl MotionResolver {
    pub fn initial(prepared: PreparedMotion) -> Self {
        Self {
            baseline: prepared.baseline,
            stack: prepared.stack,
            current: Arc::new(MotionSnapshot::initial(
                prepared.map,
                prepared.wheels,
                prepared.required.any(),
            )),
        }
    }

    pub fn publication(&self) -> Arc<MotionSnapshot> {
        self.current.clone()
    }

    pub fn install(&mut self, prepared: PreparedMotion) -> Arc<MotionSnapshot> {
        log_coefficient_changes(&self.current.map, &prepared.map);
        self.current =
            self.current
                .next(prepared.map, prepared.wheels, prepared.required.any(), true);
        self.baseline = prepared.baseline;
        self.stack = prepared.stack;
        log::debug!(
            "motion reload generation {}, canonical stack {:?}, all outputs reset",
            self.current.generation,
            self.stack
        );
        self.publication()
    }
}

fn log_coefficient_changes(before: &MouseAxisMap, after: &MouseAxisMap) {
    if !log::log_enabled!(log::Level::Debug) {
        return;
    }
    // Detailed instruction/product provenance is available by replaying
    // MotionProgram::explain_cell, off the native report/publication-lock path.
    let mut shown = 0;
    for output in MouseAxis::ALL {
        for input in MouseAxis::ALL {
            let o = output as usize;
            let i = input as usize;
            for (branch, old, new) in [
                ("+", before.rows[o][i], after.rows[o][i]),
                ("-", before.negative_rows[o][i], after.negative_rows[o][i]),
            ] {
                if branch == "-"
                    && after.mode == kanata_parser::mouse_axis::MouseAxisMapMode::Linear
                {
                    continue;
                }
                if old != new && shown < 8 {
                    log::debug!(
                        "motion {} <- {}{branch}: {old} -> {new}",
                        output.name(),
                        input.name()
                    );
                    shown += 1;
                }
            }
        }
    }
}

fn log_program_order(layers: &[LayerInfo], stack: &[u16]) {
    if !log::log_enabled!(log::Level::Debug) {
        return;
    }
    let mut count = 0;
    for &id in stack {
        let layer = &layers[usize::from(id)];
        for (index, instruction) in layer.mouse_axis.instructions().iter().enumerate() {
            if count == 8 {
                log::debug!(
                    "motion instruction summary truncated; immutable programs retain full provenance"
                );
                return;
            }
            log::debug!(
                "motion layer {id} ({:.96}), instruction {} {} at {}",
                layer.name,
                index + 1,
                instruction.operation.name(),
                instruction.source.use_site
            );
            count += 1;
        }
    }
}

impl Kanata {
    pub(crate) fn refresh_motion(&mut self) -> Result<()> {
        let layout = self.layout.bm();
        let stack = canonical_layer_order(layout.active_held_layers(), layout.default_layer as u16);
        if stack == self.motion.stack {
            return Ok(());
        }
        let map = resolve(
            &self.motion.baseline,
            &self.layer_info,
            &stack,
            self.motion.current.generation,
        )?;
        let next = self.motion.current.next(
            map,
            self.motion.current.wheels,
            self.motion.current.pipeline_enabled,
            false,
        );
        log::debug!(
            "motion generation {} stack {:?} -> {:?}, changed outputs {:?}",
            next.generation,
            self.motion.stack,
            stack,
            MouseAxis::ALL
                .into_iter()
                .filter(|&axis| !map.output_eq(&self.motion.current.map, axis))
                .collect::<Vec<_>>()
        );
        log_coefficient_changes(&self.motion.current.map, &map);
        log_program_order(&self.layer_info, &stack);
        self.motion.stack = stack;
        self.motion.current = next.clone();
        *self.motion_snapshot.lock() = next;
        Ok(())
    }

    pub(crate) fn prepare_motion_reload(&self, cfg: &mut Cfg) -> Result<PreparedMotion> {
        let prepared = PreparedMotion::new(cfg, self.motion.current.generation + 1)?;
        self.motion_capture
            .validate(prepared.required, &cfg.options)
            .map_err(|error| {
                anyhow::anyhow!(
                    "motion generation {}, canonical stack {:?}, required inputs {:?}: {error}",
                    self.motion.current.generation + 1,
                    prepared.stack,
                    MouseAxis::ALL
                        .into_iter()
                        .filter(|&axis| prepared.required.inputs[axis as usize])
                        .collect::<Vec<_>>()
                )
            })?;
        Ok(prepared)
    }
}

#[derive(Default)]
#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "windows",
    test
))]
pub(crate) struct AxisState {
    pub transform: MouseAxisTransform,
    pub wheel_key_remainder: [i32; 2],
    pub wheel_raw_remainder: [i32; 2],
    observed: Option<(u64, [u64; AXIS_COUNT])>,
}

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "windows",
    test
))]
impl AxisState {
    /// Must run before identity and all other early-return paths. This is the
    /// sole reset authority; do not follow it with legacy all-map reconfigure.
    pub fn observe_snapshot(&mut self, snapshot: &MotionSnapshot) {
        for axis in MouseAxis::ALL {
            let i = axis as usize;
            if self.observed.is_none_or(|(generation, epochs)| {
                generation != snapshot.generation || epochs[i] != snapshot.output_epochs[i]
            }) {
                self.transform.remainder[i] = 0.0;
                if let Some(wheel) = MouseAxis::WHEELS.iter().position(|&a| a == axis) {
                    self.wheel_key_remainder[wheel] = 0;
                    self.wheel_raw_remainder[wheel] = 0;
                }
            }
        }
        self.transform.map = snapshot.map;
        self.observed = Some((snapshot.generation, snapshot.output_epochs));
    }

    pub fn apply(&mut self, input: AxisValues) -> AxisValues {
        self.transform.apply(input)
    }

    /// Mapped wheel directions become key taps at full-notch boundaries. Other
    /// wheel outputs keep their high-resolution deltas for native emission.
    pub fn route_wheels(
        &mut self,
        output: &mut AxisValues,
        wheels: WheelMask,
        mut tap: impl FnMut(OsCode) -> Result<()>,
    ) -> Result<()> {
        for (index, axis) in MouseAxis::WHEELS.into_iter().enumerate() {
            let delta = output[axis as usize];
            if delta == 0 {
                continue;
            }
            let code = axis.wheel_code(delta > 0).unwrap();
            if wheels.contains(code) {
                let total = i64::from(self.wheel_key_remainder[index]) + i64::from(delta);
                let notches = total / 120;
                self.wheel_key_remainder[index] = (total % 120) as i32;
                for _ in 0..notches.unsigned_abs() {
                    tap(axis.wheel_code(notches > 0).unwrap())?;
                }
                output[axis as usize] = 0;
            } else {
                self.wheel_key_remainder[index] = 0;
            }
        }
        Ok(())
    }

    /// Signed accumulation avoids inventing a notch when partial movements
    /// reverse direction. Linux emits this legacy companion to hi-res wheels.
    #[cfg(any(target_os = "linux", target_os = "android", test))]
    pub fn wheel_notches(&mut self, index: usize, delta: i32) -> i32 {
        let total = i64::from(self.wheel_raw_remainder[index]) + i64::from(delta);
        self.wheel_raw_remainder[index] = (total % 120) as i32;
        (total / 120) as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaled_wheel_keys_wait_for_notches_and_fan_out_independently() {
        let mut state = AxisState::default();
        let mut map = MouseAxisMap::default();
        map.rows[MouseAxis::V as usize][MouseAxis::V as usize] = 0.5;
        map.rows[MouseAxis::H as usize][MouseAxis::V as usize] = -1.0;
        let snapshot = MotionSnapshot::initial(map, WheelMask::new(|_| true), true);
        state.observe_snapshot(&snapshot);
        let mut input = [0; AXIS_COUNT];
        input[MouseAxis::V as usize] = 120;
        let mut taps = vec![];
        for _ in 0..2 {
            let mut output = state.apply(input);
            state
                .route_wheels(&mut output, snapshot.wheels, |code| {
                    taps.push(code);
                    Ok(())
                })
                .unwrap();
            assert_eq!(output, [0; AXIS_COUNT]);
        }
        assert_eq!(
            taps,
            [
                OsCode::MouseWheelLeft,
                OsCode::MouseWheelUp,
                OsCode::MouseWheelLeft
            ]
        );
    }

    #[test]
    fn signed_wheel_remainders_cancel_and_preserve_on_pointer_change() {
        let mut state = AxisState::default();
        let snapshot = MotionSnapshot::initial(MouseAxisMap::default(), WheelMask::default(), true);
        state.observe_snapshot(&snapshot);
        assert_eq!(state.wheel_notches(0, 90), 0);
        assert_eq!(state.wheel_notches(0, -90), 0);
        assert_eq!(state.wheel_notches(0, -60), 0);
        assert_eq!(state.wheel_notches(0, -60), -1);
        state.wheel_notches(0, 90);
        let mut map = MouseAxisMap::default();
        map.rows[0][0] = -1.0;
        state.observe_snapshot(&snapshot.next(map, snapshot.wheels, true, false));
        assert_eq!(state.wheel_notches(0, 30), 1);
    }

    #[test]
    fn selective_epochs_survive_unobserved_round_trip_and_equal_reload() {
        let mut map = MouseAxisMap::default();
        map.rows[0][0] = 0.25;
        let a = MotionSnapshot::initial(map, WheelMask::default(), true);
        let mut state = AxisState::default();
        state.observe_snapshot(&a);
        state.apply([1; AXIS_COUNT]);
        state.wheel_key_remainder = [37, -42];
        state.wheel_raw_remainder = [22, 61];
        let mut b_map = map;
        b_map.rows[0][0] = 0.5;
        let b = a.next(b_map, a.wheels, true, false);
        let a_again = b.next(map, a.wheels, true, false);
        state.observe_snapshot(&a_again);
        assert_eq!(state.transform.remainder[0], 0.0);
        assert_eq!(state.wheel_key_remainder, [37, -42]);
        assert_eq!(state.wheel_raw_remainder, [22, 61]);
        let mut unused = map;
        unused.negative_rows[0][0] = 999.0;
        let same = a_again.next(unused, a.wheels, true, false);
        assert_eq!(same.output_epochs, a_again.output_epochs);
        state.transform.remainder[0] = 0.75;
        state.observe_snapshot(&same);
        assert_eq!(state.transform.remainder[0], 0.75);
        let reload = same.next(map, a.wheels, true, true);
        state.observe_snapshot(&reload);
        assert_eq!(state.transform.remainder, [0.0; AXIS_COUNT]);
        assert_eq!(state.wheel_key_remainder, [0; 2]);
        assert_eq!(state.wheel_raw_remainder, [0; 2]);
    }

    #[test]
    fn changed_wheel_resets_both_notch_credits_only_for_that_wheel() {
        let a = MotionSnapshot::initial(MouseAxisMap::default(), WheelMask::default(), true);
        let mut state = AxisState::default();
        state.observe_snapshot(&a);
        state.wheel_key_remainder = [5, 6];
        state.wheel_raw_remainder = [7, 8];
        let mut map = a.map;
        map.rows[MouseAxis::H as usize][0] = 3.0;
        state.observe_snapshot(&a.next(map, a.wheels, true, false));
        assert_eq!(state.wheel_key_remainder, [5, 0]);
        assert_eq!(state.wheel_raw_remainder, [7, 0]);
    }

    #[test]
    fn independent_devices_observe_negative_branch_changes_and_mode_discontinuities() {
        use kanata_parser::mouse_axis::MouseAxisMapMode;
        let a = MotionSnapshot::initial(
            MouseAxisMap {
                mode: MouseAxisMapMode::Directional,
                ..Default::default()
            },
            WheelMask::default(),
            true,
        );
        let mut devices = [AxisState::default(), AxisState::default()];
        for device in &mut devices {
            device.observe_snapshot(&a);
            device.transform.remainder = [0.5; AXIS_COUNT];
        }
        let mut map = a.map;
        map.negative_rows[0][0] = -0.25;
        let b = a.next(map, a.wheels, true, false);
        devices[0].observe_snapshot(&b);
        assert_eq!(devices[0].transform.remainder[0], 0.0);
        assert_eq!(devices[0].transform.remainder[1], 0.5);
        let again = b.next(a.map, a.wheels, true, false);
        devices[1].observe_snapshot(&again);
        assert_eq!(devices[1].transform.remainder[0], 0.0);
        assert_eq!(devices[1].transform.remainder[1], 0.5);
        let linear = again.next(MouseAxisMap::default(), a.wheels, true, false);
        devices[1].observe_snapshot(&linear);
        assert_eq!(devices[1].transform.remainder, [0.0; AXIS_COUNT]);
    }
}

#[cfg(all(test, feature = "simulated_output"))]
mod runtime_tests;
