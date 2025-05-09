use crate::domain::{Action, WorldView};

/// Unified decision interface for all agents
pub trait Brain {
    /// Decide action based on full world view and sensor inputs
    fn think(&mut self, view: &WorldView, inputs: &[f32]) -> Action;
}
