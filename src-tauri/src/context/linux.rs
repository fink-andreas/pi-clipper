use anyhow::Result;

use crate::pipeline::context::ContextDecision;

pub fn detect() -> Result<ContextDecision> {
    // TODO: X11/Wayland active window detection adapter.
    Ok(ContextDecision::unknown())
}
