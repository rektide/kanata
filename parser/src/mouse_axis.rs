//! Linear mapping of relative pointer and wheel deltas.
//!
//! Rows name outputs, columns name inputs. The supported OS channels currently
//! form a four-dimensional space, but a mapping can use any subset on either
//! side: it need not be a permutation or have equal input/output counts.
//! Pointer units are native relative counts; wheel units are 1/120 of a notch.

use crate::keys::OsCode;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum MouseAxis {
    X,
    Y,
    V,
    H,
}

pub const AXIS_COUNT: usize = 4;
pub type AxisValues = [i32; AXIS_COUNT];
pub type AxisRow = [f64; AXIS_COUNT];

impl MouseAxis {
    pub const ALL: [Self; AXIS_COUNT] = [Self::X, Self::Y, Self::V, Self::H];
    pub const WHEELS: [Self; 2] = [Self::V, Self::H];

    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "x" => Some(Self::X),
            "y" => Some(Self::Y),
            "v" => Some(Self::V),
            "h" => Some(Self::H),
            _ => None,
        }
    }

    pub fn wheel_code(self, positive: bool) -> Option<OsCode> {
        use OsCode::*;
        match (self, positive) {
            (Self::V, true) => Some(MouseWheelUp),
            (Self::V, false) => Some(MouseWheelDown),
            (Self::H, true) => Some(MouseWheelRight),
            (Self::H, false) => Some(MouseWheelLeft),
            _ => None,
        }
    }
}

/// Identity by default. An explicitly configured row replaces that output's
/// identity row; an empty row discards that output. Sources are always physical
/// input channels, so rows are simultaneous, never recursively composed.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MouseAxisMap {
    pub rows: [AxisRow; AXIS_COUNT],
}

impl Default for MouseAxisMap {
    fn default() -> Self {
        Self {
            rows: std::array::from_fn(|output| {
                std::array::from_fn(|input| if input == output { 1.0 } else { 0.0 })
            }),
        }
    }
}

impl MouseAxisMap {
    pub fn is_identity(&self) -> bool {
        *self == Self::default()
    }

    /// Whether this physical input needs interception (including fan-out into
    /// another output), rather than native passthrough.
    pub fn maps_input(&self, input: MouseAxis) -> bool {
        let identity = Self::default();
        (0..AXIS_COUNT).any(|output| {
            self.rows[output][input as usize] != identity.rows[output][input as usize]
        })
    }

    pub fn maps_movement(&self) -> bool {
        self.maps_input(MouseAxis::X) || self.maps_input(MouseAxis::Y)
    }

    pub fn maps_wheel(&self) -> bool {
        self.maps_input(MouseAxis::V) || self.maps_input(MouseAxis::H)
    }
}

/// Stateful quantization, independent of the OS adapter. Fractions are retained
/// per output and reset when the mapping changes. Sum a report's contributions
/// before quantizing, so opposite contributions can cancel without emitting a
/// spurious movement or scroll in either direction.
#[derive(Default)]
pub struct MouseAxisTransform {
    pub map: MouseAxisMap,
    pub remainder: AxisRow,
}

impl MouseAxisTransform {
    pub fn reconfigure(&mut self, map: MouseAxisMap) -> bool {
        if self.map == map {
            return false;
        }
        self.map = map;
        self.remainder = [0.0; AXIS_COUNT];
        true
    }

    pub fn apply(&mut self, input: AxisValues) -> AxisValues {
        std::array::from_fn(|output| {
            let total = self.map.rows[output]
                .iter()
                .zip(input)
                .fold(self.remainder[output], |sum, (gain, value)| {
                    sum + gain * f64::from(value)
                });
            // Saturate rather than wrap; don't retain an unrepresentable
            // backlog. NaN is only possible from extreme finite coefficients
            // whose overflowing products have opposite signs.
            if !total.is_finite() || total < i32::MIN as f64 || total > i32::MAX as f64 {
                self.remainder[output] = 0.0;
                return total as i32;
            }
            // Avoid losing a unit to rounding error, e.g. ten 0.1 steps.
            let nearest = total.round();
            let total = if (total - nearest).abs() <= 8.0 * f64::EPSILON * total.abs().max(1.0) {
                nearest
            } else {
                total
            };
            let whole = total.trunc();
            self.remainder[output] = total - whole;
            whole as i32
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_matrix_mixes_axes_and_cancels_before_quantization() {
        let mut transform = MouseAxisTransform::default();
        transform.map.rows[0] = [0.5, -0.5, 0.0, 0.0];
        transform.map.rows[1] = [0.5, 0.5, 0.0, 0.0];
        assert_eq!(transform.apply([3, 3, 0, 0]), [0, 3, 0, 0]);
        assert_eq!(transform.apply([4, -2, 0, 0]), [3, 1, 0, 0]);
    }

    #[test]
    fn wheels_can_drive_pointer_and_fan_out() {
        let mut transform = MouseAxisTransform::default();
        transform.map.rows = [
            [0.0, 0.0, 0.0, 0.25],
            [0.0, 0.0, -0.25, 0.0],
            [0.0; AXIS_COUNT],
            [0.0; AXIS_COUNT],
        ];
        assert_eq!(transform.apply([0, 0, 120, 240]), [60, -30, 0, 0]);
        transform.map.rows[2] = [0.0, 0.0, 1.0, 1.0];
        assert_eq!(transform.apply([0, 0, 120, -120]), [-30, -30, 0, 0]);
        assert!(transform.map.maps_wheel());
    }

    #[test]
    fn fractional_steps_accumulate_and_reverse_without_bias() {
        let mut transform = MouseAxisTransform::default();
        transform.map.rows[0][0] = 0.1;
        for _ in 0..9 {
            assert_eq!(transform.apply([1, 0, 0, 0])[0], 0);
        }
        assert_eq!(transform.apply([1, 0, 0, 0])[0], 1);
        for _ in 0..9 {
            assert_eq!(transform.apply([-1, 0, 0, 0])[0], 0);
        }
        assert_eq!(transform.apply([-1, 0, 0, 0])[0], -1);
        assert_eq!(transform.remainder, [0.0; AXIS_COUNT]);
    }

    #[test]
    fn input_detection_includes_cross_channel_fanout() {
        let mut map = MouseAxisMap::default();
        map.rows[MouseAxis::V as usize][MouseAxis::X as usize] = 2.0;
        assert!(map.maps_movement());
        assert!(!map.maps_wheel());
        assert!(map.maps_input(MouseAxis::X));
        assert!(!map.maps_input(MouseAxis::Y));
    }

    #[test]
    fn map_change_resets_fractional_state() {
        let mut transform = MouseAxisTransform::default();
        transform.map.rows[0][0] = 0.5;
        assert_eq!(transform.apply([1, 0, 0, 0])[0], 0);
        assert!(transform.reconfigure(MouseAxisMap::default()));
        assert_eq!(transform.remainder, [0.0; AXIS_COUNT]);
        assert_eq!(transform.apply([1, 0, 0, 0])[0], 1);
    }

    #[test]
    fn inversion_saturates_minimum_and_recovers() {
        let mut transform = MouseAxisTransform::default();
        transform.map.rows[0][0] = -1.0;
        assert_eq!(transform.apply([i32::MIN, 0, 0, 0])[0], i32::MAX);
        assert_eq!(transform.apply([1, 0, 0, 0])[0], -1);
    }
}
