//! Startup selection is frozen before TCP/processing can reload. Requirements,
//! intended installation and successful installation are different facts.

use anyhow::{Result, bail};
use kanata_parser::cfg::{Cfg, CfgOptions};
use kanata_parser::mouse_axis::program::CaptureRequirements;

#[cfg(any(target_os = "linux", target_os = "android"))]
use kanata_parser::cfg::DeviceDetectMode;

#[cfg(any(target_os = "linux", target_os = "android"))]
#[derive(Clone, Debug)]
pub(crate) struct LinuxCapturePlan {
    pub paths: Vec<String>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub mode: DeviceDetectMode,
    pub continue_if_no_devices: bool,
}

// Windows variants also compile on Linux for pure capability/preflight tests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct WindowsCapture {
    pub mouse: bool,
    pub movement: bool,
}

impl WindowsCapture {
    #[cfg(any(test, all(target_os = "windows", not(feature = "interception_driver"))))]
    pub fn llhook(hook: bool) -> Self {
        Self {
            mouse: hook,
            movement: hook,
        }
    }
    #[cfg(any(test, all(target_os = "windows", feature = "interception_driver")))]
    pub fn interception(mouse: bool, movement: bool) -> Self {
        Self {
            mouse,
            movement: mouse && movement,
        }
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)] // Other native backends are exercised by pure tests/cross builds.
pub(crate) enum CapturePlan {
    Windows(WindowsCapture),
    #[cfg(any(target_os = "linux", target_os = "android"))]
    Linux(LinuxCapturePlan),
    Unsupported,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum CaptureCapabilities {
    Windows(WindowsCapture),
    #[cfg(any(target_os = "linux", target_os = "android"))]
    Linux {
        mode: DeviceDetectMode,
        fixed_paths: bool,
    },
    Unsupported,
}

impl CapturePlan {
    pub fn from_cfg(cfg: &Cfg, required: CaptureRequirements) -> Self {
        #[cfg(any(target_os = "linux", target_os = "android"))]
        {
            let _ = required;
            let opts = &cfg.options.linux_opts;
            Self::Linux(LinuxCapturePlan {
                paths: opts.linux_dev.clone(),
                include: opts.linux_dev_names_include.clone(),
                exclude: opts.linux_dev_names_exclude.clone(),
                mode: opts
                    .linux_device_detect_mode
                    .expect("derived detection policy"),
                continue_if_no_devices: opts.linux_continue_if_no_devs_found,
            })
        }
        #[cfg(all(target_os = "windows", feature = "interception_driver"))]
        {
            let opts = &cfg.options.wintercept_opts;
            let mouse = cfg.options.mouse_movement_key.is_some()
                || opts.windows_interception_mouse_hwids.is_some()
                || opts.windows_interception_mouse_hwids_exclude.is_some()
                || required.any();
            Self::Windows(WindowsCapture::interception(
                mouse,
                required.movement() || cfg.options.mouse_movement_key.is_some(),
            ))
        }
        #[cfg(all(target_os = "windows", not(feature = "interception_driver")))]
        {
            let hook = required.any() || cfg.mapped_keys.iter().any(|code| code.is_mouse_code());
            Self::Windows(WindowsCapture::llhook(hook))
        }
        #[cfg(not(any(target_os = "linux", target_os = "android", target_os = "windows")))]
        {
            let _ = (cfg, required);
            Self::Unsupported
        }
    }

    /// Expected capabilities while installation is pending. Backends call this
    /// only *after* their native installation has succeeded to report actual facts.
    pub fn capabilities(&self) -> CaptureCapabilities {
        match self {
            Self::Windows(caps) => CaptureCapabilities::Windows(*caps),
            #[cfg(any(target_os = "linux", target_os = "android"))]
            Self::Linux(plan) => CaptureCapabilities::Linux {
                mode: plan.mode,
                fixed_paths: !plan.paths.is_empty(),
            },
            Self::Unsupported => CaptureCapabilities::Unsupported,
        }
    }
}

#[derive(Debug)]
pub(crate) struct CaptureStatus {
    pub plan: CapturePlan,
    installed: Option<CaptureCapabilities>,
}

impl CaptureStatus {
    pub fn pending(plan: CapturePlan) -> Self {
        Self {
            plan,
            installed: None,
        }
    }

    pub fn installed(&mut self, actual: CaptureCapabilities) {
        log::debug!(
            "motion capture installed: {actual:?} (startup plan {:?})",
            self.plan
        );
        self.installed = Some(actual);
    }

    pub fn validate(&self, required: CaptureRequirements, options: &CfgOptions) -> Result<()> {
        let capabilities = self.installed.unwrap_or_else(|| self.plan.capabilities());
        let phase = if self.installed.is_some() {
            "installed capture"
        } else {
            "frozen pending startup capture"
        };
        match capabilities {
            CaptureCapabilities::Windows(caps) => {
                if required.any() && !caps.mouse {
                    bail!(
                        "mouse-axis needs a mouse hook/filter absent from {phase}; restart required"
                    );
                }
                if required.movement() && !caps.movement {
                    bail!(
                        "mouse-axis needs relative MOVE capture absent from {phase}; restart required"
                    );
                }
            }
            #[cfg(any(target_os = "linux", target_os = "android"))]
            CaptureCapabilities::Linux { mode, fixed_paths } => {
                let candidate = &options.linux_opts;
                // Explicit paths and explicit detection remain user policy, not
                // proof of particular hardware axes. Do not widen that universe.
                let wants_auto_any = candidate.linux_dev.is_empty()
                    && !candidate.linux_device_detect_mode_explicit
                    && candidate.linux_device_detect_mode == Some(DeviceDetectMode::Any);
                if required.any() && wants_auto_any && !fixed_paths && mode != DeviceDetectMode::Any
                {
                    bail!(
                        "mouse-axis requires automatic relative-device discovery (Any), but {phase} uses {mode}; restart required (or retain an explicit Linux source policy)"
                    );
                }
            }
            CaptureCapabilities::Unsupported => {}
        }
        let _ = options;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanata_parser::mouse_axis::{MouseAxis, MouseAxisMap};

    fn movement() -> CaptureRequirements {
        let mut map = MouseAxisMap::default();
        map.rows[0][0] = 2.0;
        CaptureRequirements::for_programs(&map, [])
    }

    #[test]
    fn pending_and_actual_windows_capabilities_do_not_guess_from_effective_map() {
        let options = CfgOptions::default();
        let mut status = CaptureStatus::pending(CapturePlan::Windows(
            WindowsCapture::interception(true, false),
        ));
        assert!(
            status
                .validate(movement(), &options)
                .unwrap_err()
                .to_string()
                .contains("frozen pending")
        );
        let mut wheel = CaptureRequirements::default();
        wheel.inputs[MouseAxis::V as usize] = true;
        assert!(status.validate(wheel, &options).is_ok());
        status.installed(CaptureCapabilities::Windows(WindowsCapture::llhook(true)));
        assert!(status.validate(movement(), &options).is_ok());
        let mut status = CaptureStatus::pending(CapturePlan::Windows(WindowsCapture::llhook(true)));
        assert!(status.validate(movement(), &options).is_ok());
        status.installed(CaptureCapabilities::Windows(WindowsCapture::llhook(false)));
        assert!(
            status
                .validate(movement(), &options)
                .unwrap_err()
                .to_string()
                .contains("installed capture")
        );
        assert!(
            status
                .validate(CaptureRequirements::default(), &options)
                .is_ok()
        );
    }

    #[cfg(any(target_os = "linux", target_os = "android"))]
    #[test]
    fn linux_plan_freezes_selection_and_detects_new_automatic_discovery_needs() {
        let _lock = crate::tests::CFG_PARSE_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let cfg = kanata_parser::cfg::new_from_str("(defcfg linux-dev-names-include (chosen) linux-dev-names-exclude (excluded)) (defsrc) (deflayer base)", Default::default()).unwrap();
        let required = movement();
        let mut status =
            CaptureStatus::pending(CapturePlan::from_cfg(&cfg, CaptureRequirements::default()));
        let CapturePlan::Linux(ref plan) = status.plan else {
            panic!("Linux plan")
        };
        assert_eq!(plan.mode, DeviceDetectMode::KeyboardMice);
        assert_eq!(plan.include, Some(vec!["chosen".to_owned()]));
        assert_eq!(plan.exclude, Some(vec!["excluded".to_owned()]));
        let candidate = kanata_parser::cfg::new_from_str(
            "(defsrc) (deflayer base) (deflayer (later mouse-axis ((set (cell x y 2)))))",
            Default::default(),
        )
        .unwrap();
        assert!(status.validate(required, &candidate.options).is_err());
        status.installed(status.plan.capabilities());
        assert!(status.validate(required, &candidate.options).is_err());
        let mut explicit = CfgOptions::default();
        explicit.linux_opts.linux_device_detect_mode_explicit = true;
        explicit.linux_opts.linux_device_detect_mode = Some(DeviceDetectMode::KeyboardOnly);
        assert!(status.validate(required, &explicit).is_ok());
        let CapturePlan::Linux(ref mut plan) = status.plan else {
            panic!("Linux plan")
        };
        plan.paths = vec!["/dev/input/selected".to_owned()];
        status.installed(status.plan.capabilities());
        assert!(status.validate(required, &candidate.options).is_ok());
    }
}
