//! Shared analog-axis state. OS adapters retain report boundaries and normalize
//! wheel units before this transform, then encode the resulting native outputs.

use anyhow::Result;
use kanata_parser::keys::OsCode;
use kanata_parser::mouse_axis::MouseAxisTransform;
pub(crate) use kanata_parser::mouse_axis::{AXIS_COUNT, AxisValues, MouseAxis, MouseAxisMap};

#[derive(Default)]
pub(crate) struct AxisState {
    pub transform: MouseAxisTransform,
    pub wheel_key_remainder: [i32; 2],
    pub wheel_raw_remainder: [i32; 2],
}

impl AxisState {
    pub fn apply(&mut self, map: MouseAxisMap, input: AxisValues) -> AxisValues {
        if self.transform.reconfigure(map) {
            self.wheel_key_remainder = [0; 2];
            self.wheel_raw_remainder = [0; 2];
        }
        self.transform.apply(input)
    }

    /// Mapped wheel directions become key taps at full-notch boundaries. Other
    /// wheel outputs keep their high-resolution deltas for native emission.
    pub fn route_wheels(
        &mut self,
        output: &mut AxisValues,
        is_mapped: impl Fn(OsCode) -> bool,
        mut tap: impl FnMut(OsCode) -> Result<()>,
    ) -> Result<()> {
        for (index, axis) in MouseAxis::WHEELS.into_iter().enumerate() {
            let delta = output[axis as usize];
            if delta == 0 {
                continue;
            }
            let code = axis.wheel_code(delta > 0).unwrap();
            if is_mapped(code) {
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
        let mut input = [0; AXIS_COUNT];
        input[MouseAxis::V as usize] = 120;
        let mut taps = vec![];
        for _ in 0..2 {
            let mut output = state.apply(map, input);
            state
                .route_wheels(
                    &mut output,
                    |_| true,
                    |code| {
                        taps.push(code);
                        Ok(())
                    },
                )
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
    fn signed_wheel_remainders_cancel_and_reset_on_map_change() {
        let mut state = AxisState::default();
        assert_eq!(state.wheel_notches(0, 90), 0);
        assert_eq!(state.wheel_notches(0, -90), 0);
        assert_eq!(state.wheel_notches(0, -60), 0);
        assert_eq!(state.wheel_notches(0, -60), -1);
        state.wheel_notches(0, 90);
        let mut map = MouseAxisMap::default();
        map.rows[0][0] = -1.0;
        state.apply(map, [0; AXIS_COUNT]);
        assert_eq!(state.wheel_notches(0, 30), 0);
    }
}
