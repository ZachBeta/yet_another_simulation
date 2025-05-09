use crate::domain::{Action, WorldView};

/// Unified decision interface for all agents
pub trait Brain {
    /// Decide action based on full world view
    fn think(&mut self, view: &WorldView) -> Action;
}
