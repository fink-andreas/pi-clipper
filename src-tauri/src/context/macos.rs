use anyhow::Result;

use crate::pipeline::context::ContextDecision;

pub fn detect() -> Result<ContextDecision> {
    // TODO: NSWorkspace / accessibility based detection.
    Ok(ContextDecision::unknown())
}
