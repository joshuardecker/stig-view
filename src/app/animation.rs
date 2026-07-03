use std::{collections::HashMap, time::Instant};

/// A magic constant that determines how long an animation takes.
const ANIMATION_SPEED: f32 = 0.3;

/// All animations in this app are just opacity changes from 0 -> 1.
/// This handles remembering and changing animation opacitys based on names
/// like "main menu" or "sidebar".
#[derive(Debug, Clone)]
pub struct Animations {
    animations: HashMap<String, (Instant, f32)>,
}

impl Animations {
    pub fn new() -> Self {
        Self {
            animations: HashMap::new(),
        }
    }

    /// Call this when starting to play an animation.
    /// It registers the opacity as 0.
    pub fn start(&mut self, animation_name: &str) {
        self.animations
            .insert(animation_name.to_string(), (Instant::now(), 1.0));
    }

    /// Update every animations opacity with the given current time.
    pub fn tick_all(&mut self, now: Instant) {
        for (time, opacity) in self.animations.values_mut() {
            let delta_t = now.duration_since(*time).as_secs_f32();

            *opacity = *opacity - (delta_t / ANIMATION_SPEED);

            // Makes sure opacity does not go under 0.
            *opacity = opacity.max(0.0);

            *time = now;
        }
    }

    /// Get the current opacity of an animation.
    pub fn get_opacity(&self, animation_name: &str) -> f32 {
        if let Some(&(_, opacity)) = self.animations.get(animation_name) {
            opacity
        } else {
            0.0
        }
    }
}
