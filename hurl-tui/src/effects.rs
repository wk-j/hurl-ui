//! Effects module for tachyonfx animations
//!
//! This module manages visual effects and animations for the TUI.

use std::collections::HashMap;
use std::time::{Duration as StdDuration, Instant};

use ratatui::layout::Rect;
use ratatui::style::Color;
use tachyonfx::{fx, Effect, Motion, Shader};

use crate::app::ActivePanel;

/// Effect identifiers for different UI elements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum EffectId {
    /// Panel focus highlight effect
    PanelFocus(ActivePanel),
    /// Status message notification
    StatusNotification,
    /// Execution start pulse
    ExecutionStart,
    /// Execution complete flash
    ExecutionComplete,
    /// Help overlay
    HelpOverlay,
    /// Response panel update
    ResponseUpdate,
}

/// Manages all active effects and their lifecycle
pub struct EffectManager {
    /// Active effects with their areas
    effects: HashMap<EffectId, (Effect, Rect)>,
    /// Last frame timestamp for delta calculation
    last_frame: Instant,
    /// Cached delta from last tick
    last_delta: StdDuration,
}

impl EffectManager {
    /// Create a new effect manager
    pub fn new() -> Self {
        Self {
            effects: HashMap::new(),
            last_frame: Instant::now(),
            last_delta: StdDuration::ZERO,
        }
    }

    /// Calculate time since last frame and update timestamp
    /// Returns the duration since last call
    pub fn tick(&mut self) -> StdDuration {
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame);
        self.last_frame = now;
        self.last_delta = delta;
        delta
    }

    /// Get the last computed delta
    pub fn last_delta(&self) -> StdDuration {
        self.last_delta
    }

    /// Add or replace an effect
    pub fn add_effect(&mut self, id: EffectId, effect: Effect, area: Rect) {
        self.effects.insert(id, (effect, area));
    }

    /// Remove an effect
    #[allow(dead_code)]
    pub fn remove_effect(&mut self, id: &EffectId) {
        self.effects.remove(id);
    }

    /// Check if an effect exists
    #[allow(dead_code)]
    pub fn has_effect(&self, id: &EffectId) -> bool {
        self.effects.contains_key(id)
    }

    /// Process all effects and remove completed ones
    /// Returns true if any effects are still active
    pub fn process_effects(&mut self) -> bool {
        let mut to_remove = Vec::new();

        for (id, (effect, _area)) in &self.effects {
            if effect.done() {
                to_remove.push(*id);
            }
        }

        // Remove completed effects
        for id in to_remove {
            self.effects.remove(&id);
        }

        !self.effects.is_empty()
    }

    /// Get mutable iterator over effects for rendering (includes EffectId for area mapping)
    pub fn effects_iter_mut(&mut self) -> impl Iterator<Item = (&EffectId, &mut Effect, Rect)> {
        self.effects
            .iter_mut()
            .map(|(id, (effect, area))| (id, effect, *area))
    }

    /// Clear all effects
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.effects.clear();
    }

    /// Check if there are any active effects
    #[allow(dead_code)]
    pub fn has_active_effects(&self) -> bool {
        !self.effects.is_empty()
    }

    /// Get the number of active effects
    pub fn effect_count(&self) -> usize {
        self.effects.len()
    }
}

impl Default for EffectManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Effect presets for common animations
#[allow(dead_code)]
pub mod presets {
    use super::*;

    /// Duration constants (in milliseconds for tachyonfx)
    pub const FAST: u32 = 150;
    pub const NORMAL: u32 = 300;
    pub const SLOW: u32 = 500;

    /// Create a fade-in effect (coalesce = reverse dissolve)
    pub fn fade_in(duration_ms: u32) -> Effect {
        fx::coalesce(duration_ms)
    }

    /// Create a fade-out effect (dissolve)
    pub fn fade_out(duration_ms: u32) -> Effect {
        fx::dissolve(duration_ms)
    }

    /// Create a panel focus highlight effect - brief color flash
    pub fn panel_focus() -> Effect {
        fx::sequence(&[
            // Flash to bright color
            fx::fade_to_fg(Color::Cyan, FAST),
            // Return to original
            fx::fade_from_fg(Color::Cyan, FAST),
        ])
    }

    /// Create a pulse effect for execution start - yellow pulsing
    pub fn execution_pulse() -> Effect {
        fx::ping_pong(fx::fade_to_fg(Color::Yellow, NORMAL))
    }

    /// Create a success flash effect - green flash
    pub fn success_flash() -> Effect {
        fx::sequence(&[
            fx::fade_to_fg(Color::Green, FAST),
            fx::fade_from_fg(Color::Green, NORMAL),
        ])
    }

    /// Create an error flash effect - red flash
    pub fn error_flash() -> Effect {
        fx::sequence(&[
            fx::fade_to_fg(Color::Red, FAST),
            fx::fade_from_fg(Color::Red, NORMAL),
        ])
    }

    /// Create a slide-in effect from the right
    pub fn slide_in_right() -> Effect {
        fx::slide_in(Motion::RightToLeft, 20, 0, Color::Reset, NORMAL)
    }

    /// Create a slide-in effect from the bottom
    pub fn slide_in_bottom() -> Effect {
        fx::slide_in(Motion::DownToUp, 10, 0, Color::Reset, NORMAL)
    }

    /// Create a dissolve-in effect for overlays (fade in from dark)
    pub fn dissolve_in() -> Effect {
        fx::fade_from_fg(Color::DarkGray, NORMAL)
    }

    /// Create a status notification effect
    pub fn status_notification() -> Effect {
        fx::sequence(&[
            fx::coalesce(FAST),
            fx::sleep(1000), // Hold for 1 second
            fx::dissolve(SLOW),
        ])
    }

    /// Create a subtle highlight effect for response updates
    pub fn response_update() -> Effect {
        fx::coalesce(FAST)
    }

    /// Create a sweep effect for new content
    pub fn sweep_reveal() -> Effect {
        fx::sweep_in(Motion::LeftToRight, 30, 0, Color::Reset, NORMAL)
    }
}
