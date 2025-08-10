// 🦀 The Little Crab Easter Egg - A delightful surprise for curious users
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::time::{Duration, Instant};
use xilem::{
    view::{button, flex, label, Axis},
    Color, TextAlignment, WidgetView,
};

use crate::AppData;
use chrono::Datelike;

/// The little crab that lives in the corner of your screen
#[derive(Debug, Clone)]
pub struct LittleCrab {
    pub active: bool,
    pub position: (f32, f32), // Screen coordinates (0.0-1.0)
    pub target_position: (f32, f32),
    pub animation_phase: CrabPhase,
    pub last_move: Instant,
    pub message: Option<String>,
    pub message_timer: Option<Instant>,
    pub personality: CrabPersonality,
    pub energy: f32, // 0.0-1.0, affects behavior
    pub discovered_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CrabPhase {
    Hidden,   // Crab is dormant in corner
    Peeking,  // Crab cautiously looks out
    Crawling, // Crab is moving across screen
    Dancing,  // Crab is celebrating something
    Helping,  // Crab shows up to offer encouragement
    Sleeping, // Crab is taking a nap
    Excited,  // Crab is bouncing with joy
    Curious,  // Crab is investigating something
    Waving,   // Crab greets the user
}

#[derive(Debug, Clone)]
pub enum CrabPersonality {
    Shy,         // Hides quickly, peeks occasionally
    Playful,     // Dances around, shows messages
    Helpful,     // Appears during struggles, cheers successes
    Mischievous, // Randomly appears, steals focus
    Wise,        // Shows deep learning insights
}

impl Default for LittleCrab {
    fn default() -> Self {
        Self {
            active: false,
            position: (0.95, 0.95), // Bottom-right corner
            target_position: (0.95, 0.95),
            animation_phase: CrabPhase::Hidden,
            last_move: Instant::now(),
            message: None,
            message_timer: None,
            personality: CrabPersonality::Shy,
            energy: 0.8,
            discovered_at: None,
        }
    }
}

impl LittleCrab {
    /// The secret trigger - user performs a specific action to discover the crab
    pub fn try_discover(&mut self, trigger_type: CrabTrigger) -> bool {
        if self.discovered_at.is_some() {
            return false; // Already discovered
        }

        let should_appear = match trigger_type {
            CrabTrigger::TripleClick => true,
            CrabTrigger::LongPress => true,
            CrabTrigger::PerfectStreak(count) if count >= 10 => true,
            CrabTrigger::IdleTime(duration) if duration.as_secs() >= 30 => true,
            CrabTrigger::SecretWord(word) if word.to_lowercase() == "crab" => true,
            _ => false,
        };

        if should_appear {
            self.active = true;
            self.animation_phase = CrabPhase::Peeking;
            self.discovered_at = Some(chrono::Utc::now());
            self.show_discovery_message();
            true
        } else {
            false
        }
    }

    /// Update crab behavior based on app state
    pub fn update(&mut self, app_data: &AppData) {
        if !self.active {
            return;
        }

        let now = Instant::now();
        let delta = now.duration_since(self.last_move);

        // Clear expired messages
        if let Some(msg_timer) = self.message_timer {
            if now.duration_since(msg_timer) > Duration::from_secs(3) {
                self.message = None;
                self.message_timer = None;
            }
        }

        // React to user's learning progress
        self.react_to_learning_context(app_data);

        // Update animation based on personality and energy
        match self.animation_phase {
            CrabPhase::Hidden => {
                if should_peek(&self.personality, self.energy, delta) {
                    self.start_peeking();
                }
            }
            CrabPhase::Peeking => {
                if delta > Duration::from_secs(2) {
                    if self.energy > 0.6 {
                        self.start_crawling();
                    } else {
                        self.hide();
                    }
                }
            }
            CrabPhase::Crawling => {
                self.update_crawling_animation(delta);
            }
            CrabPhase::Dancing => {
                if delta > Duration::from_secs(3) {
                    self.start_crawling();
                }
            }
            CrabPhase::Helping => {
                if delta > Duration::from_secs(5) {
                    self.hide();
                }
            }
            CrabPhase::Sleeping => {
                if delta > Duration::from_secs(10) || self.energy > 0.9 {
                    self.start_peeking();
                }
            }
            CrabPhase::Excited => {
                if delta > Duration::from_secs(4) {
                    self.start_dancing();
                }
            }
            CrabPhase::Curious => {
                if delta > Duration::from_secs(3) {
                    self.start_crawling();
                }
            }
            CrabPhase::Waving => {
                if delta > Duration::from_secs(2) {
                    self.hide();
                }
            }
        }

        // Slowly drain energy to make crab less active over time
        self.energy = (self.energy - 0.001).max(0.1);
        self.last_move = now;
    }

    fn react_to_learning_context(&mut self, app_data: &AppData) {
        // React to user's learning progress and emotional state

        // Celebrate perfect answers
        if app_data.show_feedback && app_data.last_response_correct {
            if self.animation_phase == CrabPhase::Hidden && self.energy > 0.5 {
                self.celebrate_success();
            }
        }

        // Offer encouragement during struggles
        if app_data.current_metrics.streak_count == 0
            && app_data.session_responses.len() > 3
            && app_data.current_metrics.accuracy_rate < 0.5
        {
            if self.animation_phase == CrabPhase::Hidden {
                self.offer_encouragement();
            }
        }

        // Get excited about new achievements
        if matches!(
            self.personality,
            CrabPersonality::Playful | CrabPersonality::Helpful
        ) {
            if app_data.current_metrics.best_streak > 5 {
                self.energy = (self.energy + 0.1).min(1.0);
            }
        }

        // React to domain selection
        if matches!(app_data.selected_domain, crate::models::Domain::Alphabet) {
            // Crab loves the alphabet! (since it has 26 letters and crabs have fun with patterns)
            self.energy = (self.energy + 0.05).min(1.0);
        }
    }

    fn start_peeking(&mut self) {
        self.animation_phase = CrabPhase::Peeking;
        self.position = (0.92, 0.95); // Peek out a little
        self.target_position = (0.90, 0.93);
    }

    fn start_crawling(&mut self) {
        self.animation_phase = CrabPhase::Crawling;
        // Pick a random target position
        let mut rng = StdRng::from_entropy();
        self.target_position = (
            rng.gen_range(0.1..0.9),
            rng.gen_range(0.8..0.95), // Stay near bottom
        );
    }

    fn start_dancing(&mut self) {
        self.animation_phase = CrabPhase::Dancing;
        self.show_message("🦀 *happy crab noises* 🦀");
    }

    fn hide(&mut self) {
        self.animation_phase = CrabPhase::Hidden;
        self.position = (0.95, 0.95);
        self.target_position = (0.95, 0.95);
        self.message = None;
        self.message_timer = None;
    }

    fn celebrate_success(&mut self) {
        self.animation_phase = CrabPhase::Excited;
        self.energy = (self.energy + 0.2).min(1.0);

        let celebrations = [
            "🦀 Excellent! 🦀",
            "🎉 Crab-tastic! 🎉",
            "✨ You're crushing it! ✨",
            "🌟 Keep going! 🌟",
            "🦀 *celebratory claw snapping* 🦀",
        ];

        let mut rng = StdRng::from_entropy();
        let msg = celebrations[rng.gen_range(0..celebrations.len())];
        self.show_message(msg);
    }

    fn offer_encouragement(&mut self) {
        self.animation_phase = CrabPhase::Helping;

        let encouragements = [
            "🦀 Don't give up! Learning takes time 💪",
            "🌊 Every expert was once a beginner 🌊",
            "🦀 You've got this! Keep trying! 🦀",
            "✨ Mistakes help you learn faster ✨",
            "🦀 *supportive crab gestures* 🦀",
            "🌟 Progress, not perfection! 🌟",
        ];

        let mut rng = StdRng::from_entropy();
        let msg = encouragements[rng.gen_range(0..encouragements.len())];
        self.show_message(msg);
    }

    fn show_discovery_message(&mut self) {
        let discovery_messages = [
            "🦀 Oh! You found me! 🦀",
            "👋 Hello there, curious human! 👋",
            "🎉 Welcome to the secret crab club! 🎉",
            "🦀 I've been waiting for someone to notice me! 🦀",
            "✨ You have discovered: The Learning Crab! ✨",
        ];

        let mut rng = StdRng::from_entropy();
        let msg = discovery_messages[rng.gen_range(0..discovery_messages.len())];
        self.show_message(msg);
    }

    fn show_message(&mut self, message: &str) {
        self.message = Some(message.to_string());
        self.message_timer = Some(Instant::now());
    }

    fn update_crawling_animation(&mut self, delta: Duration) {
        if delta > Duration::from_millis(100) {
            // Smoothly move towards target
            let speed = 0.02;
            let dx = self.target_position.0 - self.position.0;
            let dy = self.target_position.1 - self.position.1;

            if dx.abs() < 0.01 && dy.abs() < 0.01 {
                // Reached target, pick new behavior
                if self.energy > 0.7 {
                    self.start_dancing();
                } else {
                    self.hide();
                }
            } else {
                self.position.0 += dx * speed;
                self.position.1 += dy * speed;
            }
        }
    }

    /// Get the crab's current visual representation
    pub fn get_crab_emoji(&self) -> String {
        match self.animation_phase {
            CrabPhase::Hidden => "".to_string(),
            CrabPhase::Peeking => "👀".to_string(),
            CrabPhase::Crawling => "🦀".to_string(),
            CrabPhase::Dancing => "💃🦀".to_string(),
            CrabPhase::Helping => "🦀💪".to_string(),
            CrabPhase::Sleeping => "😴🦀".to_string(),
            CrabPhase::Excited => "🎉🦀🎉".to_string(),
            CrabPhase::Curious => "🤔🦀".to_string(),
            CrabPhase::Waving => "👋🦀".to_string(),
        }
    }

    /// Check if crab should be clicked/interacted with
    pub fn handle_interaction(&mut self) {
        if self.animation_phase == CrabPhase::Hidden {
            return;
        }

        // Crab reacts to being clicked
        match self.personality {
            CrabPersonality::Playful => {
                self.start_dancing();
                self.show_message("🦀 Wheee! Click me more! 🦀");
            }
            CrabPersonality::Shy => {
                self.hide();
                self.show_message("🫥 *shy crab retreats* 🫥");
            }
            CrabPersonality::Helpful => {
                self.show_message("🦀 Need help? Keep practicing! 🦀");
                self.animation_phase = CrabPhase::Helping;
            }
            CrabPersonality::Mischievous => {
                self.energy = 1.0; // Full energy burst
                self.start_crawling();
                self.show_message("🦀 Catch me if you can! 🦀");
            }
            CrabPersonality::Wise => {
                let wisdom = [
                    "🧠 Learning is like the ocean - vast and full of treasures 🌊",
                    "🦀 The secret to mastery: consistent practice, like my daily scuttling 🦀",
                    "⭐ Every expert was once a beginner who refused to give up ⭐",
                    "🌊 Knowledge flows like tides - sometimes fast, sometimes slow 🌊",
                ];
                let mut rng = StdRng::from_entropy();
                let msg = wisdom[rng.gen_range(0..wisdom.len())];
                self.show_message(msg);
            }
        }

        self.energy = (self.energy + 0.1).min(1.0);
    }

    /// Special seasonal or contextual behaviors
    pub fn seasonal_behavior(&mut self) {
        let now = chrono::Utc::now();

        // Holiday behaviors
        if now.month() == 12 && now.day() >= 24 && now.day() <= 26 {
            // Christmas behavior
            self.show_message("🎄🦀 Merry Crabmas! 🦀🎄");
        } else if now.month() == 10 && now.day() == 31 {
            // Halloween behavior
            self.show_message("🎃🦀 Boo! Spooky learning crab! 🦀🎃");
        } else if now.month() == 7 && now.day() == 4 {
            // July 4th (or any celebration of independence/freedom to learn!)
            self.show_message("🎆🦀 Celebrate your learning independence! 🦀🎆");
        }
    }
}

#[derive(Debug, Clone)]
pub enum CrabTrigger {
    TripleClick,
    LongPress,
    PerfectStreak(usize),
    IdleTime(Duration),
    SecretWord(String),
    KonamiCode,
}

/// Helper function to determine if crab should peek based on personality
fn should_peek(personality: &CrabPersonality, energy: f32, time_since_last_move: Duration) -> bool {
    let base_chance = match personality {
        CrabPersonality::Shy => 0.01,
        CrabPersonality::Playful => 0.05,
        CrabPersonality::Helpful => 0.03,
        CrabPersonality::Mischievous => 0.07,
        CrabPersonality::Wise => 0.02,
    };

    let energy_modifier = energy * 2.0;
    let time_modifier = if time_since_last_move > Duration::from_secs(60) {
        2.0
    } else {
        1.0
    };

    let mut rng = StdRng::from_entropy();
    rng.gen_bool((base_chance * energy_modifier * time_modifier) as f64)
}

/// Render the crab as an overlay widget
pub fn render_crab_overlay(crab: &LittleCrab) -> Option<impl WidgetView<AppData>> {
    if !crab.active || crab.animation_phase == CrabPhase::Hidden {
        return None;
    }

    let crab_display = flex((
        button(crab.get_crab_emoji(), |data: &mut AppData| {
            if let Some(crab) = &mut data.little_crab {
                crab.handle_interaction();
            }
        }),
        if let Some(message) = &crab.message {
            Some(
                label(message.as_str())
                    .brush(Color::from_rgb8(255, 255, 255))
                    .alignment(TextAlignment::Start),
            )
        } else {
            None
        },
    ))
    .direction(Axis::Vertical);

    Some(crab_display)
}

/// Initialize crab with random personality for variety
pub fn init_random_crab() -> LittleCrab {
    let mut rng = StdRng::from_entropy();
    let personalities = [
        CrabPersonality::Shy,
        CrabPersonality::Playful,
        CrabPersonality::Helpful,
        CrabPersonality::Mischievous,
        CrabPersonality::Wise,
    ];

    let personality = personalities[rng.gen_range(0..personalities.len())].clone();

    LittleCrab {
        personality,
        energy: rng.gen_range(0.3..0.9),
        ..Default::default()
    }
}
