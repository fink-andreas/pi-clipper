use anyhow::Result;

use crate::pipeline::context::ContextDecision;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "linux")]
mod linux;

pub fn detect_active_context() -> Result<ContextDecision> {
    #[cfg(target_os = "windows")]
    {
        return windows::detect();
    }

    #[cfg(target_os = "macos")]
    {
        return macos::detect();
    }

    #[cfg(target_os = "linux")]
    {
        return linux::detect();
    }

    #[allow(unreachable_code)]
    Ok(ContextDecision::unknown())
}
