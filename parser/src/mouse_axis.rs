//! Relative-axis mapping, linear by default with optional independent directions.
//!
//! Rows name outputs, columns name inputs. A mapping can use any subset on either
//! side: it need not be a permutation or have equal input/output counts. The
//! channel vocabulary includes Linux's extra relative axes; each OS adapter
//! validates its capabilities rather than limiting all platforms to X/Y/V/H.
//! Pointer units are native relative counts; wheel units are 1/120 of a notch.

use crate::keys::OsCode;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum MouseAxis {
    X,
    Y,
    V,
    H,
    Z,
    Rx,
    Ry,
    Rz,
    Dial,
    Misc,
}

pub const AXIS_COUNT: usize = MouseAxis::ALL.len();
pub type AxisValues = [i32; AXIS_COUNT];
pub type AxisRow = [f64; AXIS_COUNT];

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum MouseAxisMapMode {
    /// One coefficient per input; negative input reverses the output.
    #[default]
    Linear,
    /// Separate positive/negative input branches, each using a positive
    /// magnitude. The branch's coefficient chooses the output direction.
    Directional,
}

impl MouseAxis {
    pub const ALL: [Self; 10] = [
        Self::X,
        Self::Y,
        Self::V,
        Self::H,
        Self::Z,
        Self::Rx,
        Self::Ry,
        Self::Rz,
        Self::Dial,
        Self::Misc,
    ];
    pub const WHEELS: [Self; 2] = [Self::V, Self::H];

    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "x" => Some(Self::X),
            "y" => Some(Self::Y),
            "v" => Some(Self::V),
            "h" => Some(Self::H),
            "z" => Some(Self::Z),
            "rx" => Some(Self::Rx),
            "ry" => Some(Self::Ry),
            "rz" => Some(Self::Rz),
            "dial" => Some(Self::Dial),
            "misc" => Some(Self::Misc),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::X => "x",
            Self::Y => "y",
            Self::V => "v",
            Self::H => "h",
            Self::Z => "z",
            Self::Rx => "rx",
            Self::Ry => "ry",
            Self::Rz => "rz",
            Self::Dial => "dial",
            Self::Misc => "misc",
        }
    }

    pub fn is_wheel(self) -> bool {
        matches!(self, Self::V | Self::H)
    }

    pub fn supported(self) -> bool {
        cfg!(any(
            target_os = "linux",
            target_os = "android",
            target_os = "unknown"
        )) || matches!(self, Self::X | Self::Y | Self::V | Self::H)
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
    pub negative_rows: [AxisRow; AXIS_COUNT],
    pub mode: MouseAxisMapMode,
}

impl Default for MouseAxisMap {
    fn default() -> Self {
        Self {
            rows: std::array::from_fn(|output| {
                std::array::from_fn(|input| if input == output { 1.0 } else { 0.0 })
            }),
            negative_rows: std::array::from_fn(|output| {
                std::array::from_fn(|input| if input == output { -1.0 } else { 0.0 })
            }),
            mode: MouseAxisMapMode::Linear,
        }
    }
}

impl MouseAxisMap {
    pub fn is_identity(&self) -> bool {
        let identity = Self::default();
        self.rows == identity.rows
            && (self.mode == MouseAxisMapMode::Linear
                || self.negative_rows == identity.negative_rows)
    }

    /// Whether this physical input needs interception (including fan-out into
    /// another output), rather than native passthrough.
    pub fn maps_input(&self, input: MouseAxis) -> bool {
        let identity = Self::default();
        (0..AXIS_COUNT).any(|output| {
            self.rows[output][input as usize] != identity.rows[output][input as usize]
                || (self.mode == MouseAxisMapMode::Directional
                    && self.negative_rows[output][input as usize]
                        != identity.negative_rows[output][input as usize])
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
            let total =
                input
                    .iter()
                    .enumerate()
                    .fold(self.remainder[output], |sum, (axis, &value)| {
                        let contribution = if self.map.mode == MouseAxisMapMode::Directional
                            && value < 0
                        {
                            self.map.negative_rows[output][axis] * f64::from(value.unsigned_abs())
                        } else {
                            self.map.rows[output][axis] * f64::from(value)
                        };
                        sum + contribution
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

    fn values(x: i32, y: i32, v: i32, h: i32) -> AxisValues {
        let mut values = [0; AXIS_COUNT];
        values[..4].copy_from_slice(&[x, y, v, h]);
        values
    }

    fn row(x: f64, y: f64, v: f64, h: f64) -> AxisRow {
        let mut row = [0.0; AXIS_COUNT];
        row[..4].copy_from_slice(&[x, y, v, h]);
        row
    }

    #[test]
    fn full_matrix_mixes_axes_and_cancels_before_quantization() {
        let mut transform = MouseAxisTransform::default();
        transform.map.rows[0] = row(0.5, -0.5, 0.0, 0.0);
        transform.map.rows[1] = row(0.5, 0.5, 0.0, 0.0);
        assert_eq!(transform.apply(values(3, 3, 0, 0)), values(0, 3, 0, 0));
        assert_eq!(transform.apply(values(4, -2, 0, 0)), values(3, 1, 0, 0));
    }

    #[test]
    fn wheels_can_drive_pointer_and_fan_out() {
        let mut transform = MouseAxisTransform::default();
        transform.map.rows[0] = row(0.0, 0.0, 0.0, 0.25);
        transform.map.rows[1] = row(0.0, 0.0, -0.25, 0.0);
        transform.map.rows[2] = [0.0; AXIS_COUNT];
        transform.map.rows[3] = [0.0; AXIS_COUNT];
        assert_eq!(
            transform.apply(values(0, 0, 120, 240)),
            values(60, -30, 0, 0)
        );
        transform.map.rows[2] = row(0.0, 0.0, 1.0, 1.0);
        assert_eq!(
            transform.apply(values(0, 0, 120, -120)),
            values(-30, -30, 0, 0)
        );
        assert!(transform.map.maps_wheel());
    }

    #[test]
    fn fractional_steps_accumulate_and_reverse_without_bias() {
        let mut transform = MouseAxisTransform::default();
        transform.map.rows[0][0] = 0.1;
        for _ in 0..9 {
            assert_eq!(transform.apply(values(1, 0, 0, 0))[0], 0);
        }
        assert_eq!(transform.apply(values(1, 0, 0, 0))[0], 1);
        for _ in 0..9 {
            assert_eq!(transform.apply(values(-1, 0, 0, 0))[0], 0);
        }
        assert_eq!(transform.apply(values(-1, 0, 0, 0))[0], -1);
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
        assert_eq!(transform.apply(values(1, 0, 0, 0))[0], 0);
        assert!(transform.reconfigure(MouseAxisMap::default()));
        assert_eq!(transform.remainder, [0.0; AXIS_COUNT]);
        assert_eq!(transform.apply(values(1, 0, 0, 0))[0], 1);
    }

    #[test]
    fn inversion_saturates_minimum_and_recovers() {
        let mut transform = MouseAxisTransform::default();
        transform.map.rows[0][0] = -1.0;
        assert_eq!(transform.apply(values(i32::MIN, 0, 0, 0))[0], i32::MAX);
        assert_eq!(transform.apply(values(1, 0, 0, 0))[0], -1);
    }

    #[test]
    fn extended_axes_can_feed_pointer_and_receive_other_inputs() {
        let mut transform = MouseAxisTransform::default();
        transform.map.rows[MouseAxis::X as usize] = [0.0; AXIS_COUNT];
        transform.map.rows[MouseAxis::X as usize][MouseAxis::Z as usize] = 2.0;
        transform.map.rows[MouseAxis::X as usize][MouseAxis::Dial as usize] = -1.0;
        transform.map.rows[MouseAxis::Rz as usize] = [0.0; AXIS_COUNT];
        transform.map.rows[MouseAxis::Rz as usize][MouseAxis::V as usize] = 0.5;
        let mut input = values(0, 0, 120, 0);
        input[MouseAxis::Z as usize] = 3;
        input[MouseAxis::Dial as usize] = 2;
        let output = transform.apply(input);
        assert_eq!(output[MouseAxis::X as usize], 4);
        assert_eq!(output[MouseAxis::Rz as usize], 60);
        assert_eq!(output[MouseAxis::Z as usize], 3); // omitted row passes through
    }

    #[test]
    fn directional_mode_separates_pointer_and_wheel_polarities() {
        let mut transform = MouseAxisTransform::default();
        transform.map.mode = MouseAxisMapMode::Directional;
        // Positive V -> right; negative V -> down, with a different gain.
        transform.map.rows[MouseAxis::X as usize][MouseAxis::V as usize] = 0.25;
        transform.map.negative_rows[MouseAxis::Y as usize][MouseAxis::V as usize] = 0.5;
        transform.map.rows[MouseAxis::V as usize] = [0.0; AXIS_COUNT];
        transform.map.negative_rows[MouseAxis::V as usize] = [0.0; AXIS_COUNT];
        assert_eq!(transform.apply(values(0, 0, 120, 0)), values(30, 0, 0, 0));
        assert_eq!(transform.apply(values(0, 0, -120, 0)), values(0, 60, 0, 0));
        // An omitted pointer row still has normal negative-input behavior.
        assert_eq!(transform.apply(values(-3, -4, 0, 0)), values(-3, -4, 0, 0));
    }

    #[test]
    fn directional_default_is_identity_and_negative_only_changes_need_capture() {
        let mut map = MouseAxisMap {
            mode: MouseAxisMapMode::Directional,
            ..Default::default()
        };
        assert!(map.is_identity());
        assert!(!map.maps_movement());
        map.negative_rows[MouseAxis::V as usize][MouseAxis::Y as usize] = 1.0;
        assert!(map.maps_movement());
        assert!(!map.is_identity());
    }

    #[test]
    fn directional_negative_minimum_has_positive_magnitude_without_overflow() {
        let mut transform = MouseAxisTransform::default();
        transform.map.mode = MouseAxisMapMode::Directional;
        transform.map.negative_rows[0][0] = 0.5;
        assert_eq!(transform.apply(values(i32::MIN, 0, 0, 0))[0], 1 << 30);
    }
}
