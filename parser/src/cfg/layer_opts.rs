use crate::cfg::*;
use crate::mouse_axis::program::{AxisDomain, MotionProgram};
use crate::*;

pub(crate) const DEFLAYER_ICON: [&str; 3] = ["icon", "🖻", "🖼"];
pub(crate) type LayerOptionsByName = HashMap<String, LayerOptions>;

#[derive(Debug)]
pub struct LayerOptions {
    pub icon: Option<String>,
    pub mouse_axis: MotionProgram,
}

impl LayerOptions {
    pub fn empty(domain: AxisDomain) -> Self {
        Self {
            icon: None,
            mouse_axis: MotionProgram::new(domain, vec![]).expect("empty program"),
        }
    }
}

pub fn parse_layer_opts(
    list: &[SExpr],
    vars: &HashMap<String, SExpr>,
    domain: AxisDomain,
) -> Result<LayerOptions> {
    let mut options = LayerOptions::empty(domain);
    let mut seen = HashSet::default();
    let (pairs, remainder) = list.as_chunks::<2>();
    for [key_expr, val_expr] in pairs {
        let key = key_expr.atom(Some(vars)).ok_or_else(|| {
            anyhow_expr!(key_expr, "No lists are allowed in {DEFLAYER} option names")
        })?;
        let key = if DEFLAYER_ICON.contains(&key) {
            "icon"
        } else {
            key
        };
        if !seen.insert(key.to_owned()) {
            bail_expr!(key_expr, "Duplicate option found in {DEFLAYER}: {key}");
        }
        match key {
            "icon" => {
                options.icon = Some(
                    val_expr
                        .atom(Some(vars))
                        .ok_or_else(|| {
                            anyhow_expr!(
                                val_expr,
                                "No lists are allowed in {DEFLAYER}'s icon value"
                            )
                        })?
                        .trim_atom_quotes()
                        .to_owned(),
                );
            }
            "mouse-axis" => {
                options.mouse_axis =
                    mouse_axis_program::parse_motion_program(val_expr, vars, domain)?
            }
            _ => bail_expr!(
                key_expr,
                "Invalid option in {DEFLAYER}: {key}, expected icon, 🖻, 🖼 or mouse-axis"
            ),
        }
    }
    if let Some(rem) = remainder.first() {
        bail_expr!(rem, "This option is missing a value.");
    }
    Ok(options)
}
