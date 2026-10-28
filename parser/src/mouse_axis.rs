//! Physical mouse axis mapping: pointer movement (X/Y) and wheel
//! (vertical/horizontal) axes can each be swapped and inverted.
//!
//! Each output axis draws from one input axis with an optional sign flip — a
//! signed permutation of the axis pair. This covers the flip/swap operations
//! requested in <https://github.com/jtroo/kanata/issues/2192> and is
//! deliberately shaped like a 2×2 linear transform so more general
//! projections (arbitrary-angle rotation, scaling) can later extend
//! [`MouseAxisMap::map_movement`] without changing the configuration surface.
//!
//! Sign conventions, shared by every platform backend:
//! - pointer X grows rightward, pointer Y grows downward;
//! - wheel V is positive scrolling up, wheel H is positive scrolling right.

use crate::keys::OsCode;

/// Pointer movement axis in screen coordinates.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum MovementAxis {
    #[default]
    X,
    Y,
}

/// Wheel axis: the vertical wheel (V) or the horizontal wheel (H).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum WheelAxis {
    #[default]
    V,
    H,
}

/// One input axis feeding an output pointer axis, optionally inverted
/// (e.g. `-y`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MovementSource {
    pub axis: MovementAxis,
    pub invert: bool,
}

impl MovementSource {
    pub const X: Self = Self {
        axis: MovementAxis::X,
        invert: false,
    };
    pub const Y: Self = Self {
        axis: MovementAxis::Y,
        invert: false,
    };

    pub fn parse(s: &str) -> Option<Self> {
        let (invert, axis) = split_sign(s)?;
        let axis = match axis.to_ascii_lowercase().as_str() {
            "x" => MovementAxis::X,
            "y" => MovementAxis::Y,
            _ => return None,
        };
        Some(Self { axis, invert })
    }
}

/// One input wheel axis feeding an output wheel axis, optionally inverted
/// (e.g. `-h`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct WheelSource {
    pub axis: WheelAxis,
    pub invert: bool,
}

impl WheelSource {
    pub const V: Self = Self {
        axis: WheelAxis::V,
        invert: false,
    };
    pub const H: Self = Self {
        axis: WheelAxis::H,
        invert: false,
    };

    pub fn parse(s: &str) -> Option<Self> {
        let (invert, axis) = split_sign(s)?;
        let axis = match axis.to_ascii_lowercase().as_str() {
            "v" => WheelAxis::V,
            "h" => WheelAxis::H,
            _ => return None,
        };
        Some(Self { axis, invert })
    }
}

/// Accepted values for `mouse-x-map` / `mouse-y-map`.
pub const MOVEMENT_SOURCE_VALUES: [&str; 4] = ["x", "y", "-x", "-y"];
/// Accepted values for `mouse-wheel-v-map` / `mouse-wheel-h-map`.
pub const WHEEL_SOURCE_VALUES: [&str; 4] = ["v", "h", "-v", "-h"];

fn split_sign(s: &str) -> Option<(bool, &str)> {
    if let Some(rest) = s.trim().strip_prefix('-') {
        Some((true, rest))
    } else if let Some(rest) = s.trim().strip_prefix('+') {
        Some((false, rest))
    } else {
        Some((false, s.trim()))
    }
}

/// Physical mouse axis mapping: the input source of each output pointer and
/// wheel axis. Defaults to the identity mapping.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MouseAxisMap {
    /// Input axis feeding output pointer X movement.
    pub movement_x: MovementSource,
    /// Input axis feeding output pointer Y movement.
    pub movement_y: MovementSource,
    /// Input wheel axis feeding the output vertical wheel.
    pub wheel_v: WheelSource,
    /// Input wheel axis feeding the output horizontal wheel.
    pub wheel_h: WheelSource,
}

impl Default for MouseAxisMap {
    fn default() -> Self {
        Self {
            movement_x: MovementSource::X,
            movement_y: MovementSource::Y,
            wheel_v: WheelSource::V,
            wheel_h: WheelSource::H,
        }
    }
}

impl MouseAxisMap {
    pub fn is_identity(&self) -> bool {
        *self == Self::default()
    }

    /// Whether pointer movement axes are remapped away from the identity.
    pub fn maps_movement(&self) -> bool {
        self.movement_x != MovementSource::X || self.movement_y != MovementSource::Y
    }

    /// Whether wheel axes are remapped away from the identity.
    pub fn maps_wheel(&self) -> bool {
        self.wheel_v != WheelSource::V || self.wheel_h != WheelSource::H
    }

    /// Apply the movement mapping to a pointer delta.
    ///
    /// When both output axes draw from the same input axis, that input feeds
    /// both outputs (moving diagonally) and the other input axis is
    /// discarded — the equivalent of Karabiner's axis `discard`.
    pub fn map_movement(&self, dx: i32, dy: i32) -> (i32, i32) {
        let project = |src: MovementSource| {
            let v = if src.axis == MovementAxis::X { dx } else { dy };
            if src.invert {
                v.saturating_neg()
            } else {
                v
            }
        };
        (project(self.movement_x), project(self.movement_y))
    }

    /// Map one wheel event, described by its input axis and direction, to
    /// the output wheel axis and direction.
    ///
    /// Wheel mappings must be permutations (one input axis feeds each output
    /// wheel; enforced during parsing), so every wheel event maps to exactly
    /// one output wheel event: the output whose source draws from the input
    /// axis consumes the event.
    pub fn map_wheel(&self, axis: WheelAxis, positive: bool) -> (WheelAxis, bool) {
        if self.wheel_v.axis == axis {
            (WheelAxis::V, positive ^ self.wheel_v.invert)
        } else {
            (WheelAxis::H, positive ^ self.wheel_h.invert)
        }
    }

    /// Map a wheel key code (`mwu`/`mwd`/`mwl`/`mwr`) through the wheel
    /// mapping. Non-wheel codes pass through unchanged.
    pub fn map_wheel_code(&self, code: OsCode) -> OsCode {
        use OsCode::*;
        let (axis, positive) = match code {
            MouseWheelUp => (WheelAxis::V, true),
            MouseWheelDown => (WheelAxis::V, false),
            MouseWheelRight => (WheelAxis::H, true),
            MouseWheelLeft => (WheelAxis::H, false),
            _ => return code,
        };
        let (axis, positive) = self.map_wheel(axis, positive);
        match (axis, positive) {
            (WheelAxis::V, true) => MouseWheelUp,
            (WheelAxis::V, false) => MouseWheelDown,
            (WheelAxis::H, true) => MouseWheelRight,
            (WheelAxis::H, false) => MouseWheelLeft,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use OsCode::*;

    #[test]
    fn parses_sources_with_optional_sign() {
        assert_eq!(MovementSource::parse("x"), Some(MovementSource::X));
        assert_eq!(MovementSource::parse("-y"), Some(MovementSource {
            axis: MovementAxis::Y,
            invert: true
        }));
        assert_eq!(MovementSource::parse("+X"), Some(MovementSource::X));
        assert_eq!(MovementSource::parse("nope"), None);
        assert_eq!(WheelSource::parse("h"), Some(WheelSource::H));
        assert_eq!(WheelSource::parse("-V"), Some(WheelSource {
            axis: WheelAxis::V,
            invert: true
        }));
        assert_eq!(WheelSource::parse("x"), None);
    }

    #[test]
    fn default_is_identity() {
        let m = MouseAxisMap::default();
        assert!(m.is_identity());
        assert!(!m.maps_movement());
        assert!(!m.maps_wheel());
        assert_eq!(m.map_movement(3, -5), (3, -5));
        assert_eq!(m.map_wheel_code(MouseWheelUp), MouseWheelUp);
    }

    #[test]
    fn movement_swap_and_invert() {
        let m = MouseAxisMap {
            movement_x: MovementSource {
                axis: MovementAxis::Y,
                invert: false,
            },
            movement_y: MovementSource {
                axis: MovementAxis::X,
                invert: true,
            },
            ..Default::default()
        };
        assert!(m.maps_movement());
        // x ← y, y ← -x: a 90° pointer rotation.
        assert_eq!(m.map_movement(3, 7), (7, -3));
        assert_eq!(m.map_movement(-2, 1), (1, 2));
    }

    #[test]
    fn movement_same_axis_source_discards_the_other() {
        let m = MouseAxisMap {
            movement_x: MovementSource::X,
            movement_y: MovementSource {
                axis: MovementAxis::X,
                invert: true,
            },
            ..Default::default()
        };
        // y ← -x: vertical input is discarded, horizontal input mirrors.
        assert_eq!(m.map_movement(4, 9), (4, -4));
        assert_eq!(m.map_movement(0, 9), (0, 0));
    }

    #[test]
    fn wheel_flip_and_swap() {
        let flip_v = MouseAxisMap {
            wheel_v: WheelSource {
                axis: WheelAxis::V,
                invert: true,
            },
            ..Default::default()
        };
        assert!(flip_v.maps_wheel());
        assert_eq!(flip_v.map_wheel_code(MouseWheelUp), MouseWheelDown);
        assert_eq!(flip_v.map_wheel_code(MouseWheelDown), MouseWheelUp);
        // H untouched.
        assert_eq!(flip_v.map_wheel_code(MouseWheelRight), MouseWheelRight);

        let swap = MouseAxisMap {
            wheel_v: WheelSource::H,
            wheel_h: WheelSource {
                axis: WheelAxis::V,
                invert: true,
            },
            ..Default::default()
        };
        assert_eq!(swap.map_wheel_code(MouseWheelUp), MouseWheelLeft);
        assert_eq!(swap.map_wheel_code(MouseWheelDown), MouseWheelRight);
        assert_eq!(swap.map_wheel_code(MouseWheelRight), MouseWheelUp);
        assert_eq!(swap.map_wheel_code(MouseWheelLeft), MouseWheelDown);
    }

    #[test]
    fn non_wheel_codes_pass_through() {
        let m = MouseAxisMap {
            wheel_v: WheelSource {
                axis: WheelAxis::H,
                invert: true,
            },
            wheel_h: WheelSource::V,
            ..Default::default()
        };
        assert_eq!(m.map_wheel_code(OsCode::KEY_A), OsCode::KEY_A);
    }
}
