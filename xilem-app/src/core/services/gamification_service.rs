// Gamification system for the Adaptive Learning System
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};

/// Achievement categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AchievementCategory {
    Streak,       // Daily practice streaks
    Accuracy,     // Performance milestones
    Speed,        // Response time achievements
    Volume,       // Total tasks completed
    Mastery,      // Domain mastery levels
    Explorer,     // Trying different features
    Social,       // Leaderboard and competition
    Special,      // Time-limited or special events
}

/// Individual achievement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: AchievementCategory,
    pub icon: String,
    pub points: u32,
    pub rarity: Rarity,
    pub unlocked: bool,
    pub unlocked_at: Option<DateTime<Utc>>,
    pub progress: f32,  // 0.0 to 1.0
    pub requirement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl Rarity {
    pub fn color(&self) -> &str {
        match self {
            Rarity::Common => "#808080",     // Gray
            Rarity::Uncommon => "#48bb78",   // Green
            Rarity::Rare => "#4299e1",       // Blue
            Rarity::Epic => "#9f7aea",       // Purple
            Rarity::Legendary => "#f6ad55",  // Gold
        }
    }
    
    pub fn multiplier(&self) -> f32 {
        match self {
            Rarity::Common => 1.0,
            Rarity::Uncommon => 1.5,
            Rarity::Rare => 2.0,
            Rarity::Epic => 3.0,
            Rarity::Legendary => 5.0,
        }
    }
}

/// User's gamification profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamificationProfile {
    pub user_id: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub level: u32,
    pub experience: u32,
    pub total_points: u32,
    pub rank: Rank,
    pub achievements: Vec<Achievement>,
    pub badges: Vec<Badge>,
    pub streak: StreakInfo,
    pub statistics: UserStatistics,
    pub weekly_goals: WeeklyGoals,
    pub power_ups: Vec<PowerUp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rank {
    pub title: String,
    pub tier: u32,  // 1-10 within each title
    pub icon: String,
    pub color: String,
}

impl Rank {
    pub fn from_level(level: u32) -> Self {
        let (title, icon, color) = match level {
            0..=9 => ("Novice", "🌱", "#48bb78"),
            10..=24 => ("Apprentice", "📚", "#4299e1"),
            25..=49 => ("Scholar", "🎓", "#9f7aea"),
            50..=74 => ("Expert", "⭐", "#ed8936"),
            75..=99 => ("Master", "🏆", "#f6ad55"),
            100..=149 => ("Grandmaster", "👑", "#e53e3e"),
            150..=199 => ("Sage", "🧙", "#b794f4"),
            _ => ("Legend", "🌟", "#ffd700"),
        };
        
        Rank {
            title: title.to_string(),
            tier: (level % 10) + 1,
            icon: icon.to_string(),
            color: color.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Badge {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub description: String,
    pub earned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreakInfo {
    pub current: u32,
    pub longest: u32,
    pub last_practice: Option<DateTime<Utc>>,
    pub freeze_charges: u32,  // Allow missing a day without breaking streak
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatistics {
    pub total_sessions: u32,
    pub total_tasks: u32,
    pub total_time_minutes: u32,
    pub average_accuracy: f32,
    pub best_accuracy_session: f32,
    pub fastest_response_ms: u32,
    pub domains_mastered: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyGoals {
    pub tasks_goal: u32,
    pub tasks_completed: u32,
    pub accuracy_goal: f32,
    pub current_accuracy: f32,
    pub streak_goal: u32,
    pub current_streak: u32,
    pub bonus_multiplier: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerUp {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub duration: Duration,
    pub effect: PowerUpEffect,
    pub active_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PowerUpEffect {
    DoubleXP,
    HintBoost,      // Extra hints
    TimeFreeze,     // More time to answer
    StreakShield,   // Protect streak for one day
    FocusMode,      // Hide distractions
}

/// Leaderboard entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub rank: u32,
    pub user_id: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub score: u32,
    pub level: u32,
    pub achievement_count: u32,
    pub trend: Trend,  // Up, down, or stable
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Trend {
    Up(u32),    // Positions gained
    Down(u32),  // Positions lost
    Stable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LeaderboardType {
    Global,
    Weekly,
    Monthly,
    Friends,
    Domain(String),
}

/// Manager for all gamification features
pub struct GamificationManager {
    achievements: HashMap<String, Achievement>,
    user_profiles: HashMap<String, GamificationProfile>,
}

impl GamificationManager {
    pub fn new() -> Self {
        let mut manager = Self {
            achievements: HashMap::new(),
            user_profiles: HashMap::new(),
        };
        manager.initialize_achievements();
        manager
    }
    
    fn initialize_achievements(&mut self) {
        // Streak achievements
        self.add_achievement(Achievement {
            id: "streak_7".to_string(),
            name: "Week Warrior".to_string(),
            description: "Practice for 7 days in a row".to_string(),
            category: AchievementCategory::Streak,
            icon: "🔥".to_string(),
            points: 50,
            rarity: Rarity::Common,
            unlocked: false,
            unlocked_at: None,
            progress: 0.0,
            requirement: "7 day streak".to_string(),
        });
        
        self.add_achievement(Achievement {
            id: "streak_30".to_string(),
            name: "Monthly Master".to_string(),
            description: "Practice for 30 days in a row".to_string(),
            category: AchievementCategory::Streak,
            icon: "🌟".to_string(),
            points: 200,
            rarity: Rarity::Rare,
            unlocked: false,
            unlocked_at: None,
            progress: 0.0,
            requirement: "30 day streak".to_string(),
        });
        
        // Accuracy achievements
        self.add_achievement(Achievement {
            id: "accuracy_90".to_string(),
            name: "Sharp Shooter".to_string(),
            description: "Complete a session with 90% accuracy".to_string(),
            category: AchievementCategory::Accuracy,
            icon: "🎯".to_string(),
            points: 75,
            rarity: Rarity::Uncommon,
            unlocked: false,
            unlocked_at: None,
            progress: 0.0,
            requirement: "90% accuracy in session".to_string(),
        });
        
        self.add_achievement(Achievement {
            id: "perfect_session".to_string(),
            name: "Perfectionist".to_string(),
            description: "Complete a session with 100% accuracy".to_string(),
            category: AchievementCategory::Accuracy,
            icon: "💯".to_string(),
            points: 150,
            rarity: Rarity::Epic,
            unlocked: false,
            unlocked_at: None,
            progress: 0.0,
            requirement: "100% accuracy in session".to_string(),
        });
        
        // Speed achievements
        self.add_achievement(Achievement {
            id: "speed_demon".to_string(),
            name: "Speed Demon".to_string(),
            description: "Answer 10 questions in under 10 seconds each".to_string(),
            category: AchievementCategory::Speed,
            icon: "⚡".to_string(),
            points: 100,
            rarity: Rarity::Rare,
            unlocked: false,
            unlocked_at: None,
            progress: 0.0,
            requirement: "10 fast responses".to_string(),
        });
        
        // Volume achievements
        self.add_achievement(Achievement {
            id: "century".to_string(),
            name: "Centurion".to_string(),
            description: "Complete 100 tasks".to_string(),
            category: AchievementCategory::Volume,
            icon: "💪".to_string(),
            points: 50,
            rarity: Rarity::Common,
            unlocked: false,
            unlocked_at: None,
            progress: 0.0,
            requirement: "100 tasks completed".to_string(),
        });
        
        self.add_achievement(Achievement {
            id: "millennium".to_string(),
            name: "Task Master".to_string(),
            description: "Complete 1000 tasks".to_string(),
            category: AchievementCategory::Volume,
            icon: "🏅".to_string(),
            points: 250,
            rarity: Rarity::Epic,
            unlocked: false,
            unlocked_at: None,
            progress: 0.0,
            requirement: "1000 tasks completed".to_string(),
        });
        
        // Mastery achievements
        self.add_achievement(Achievement {
            id: "alphabet_master".to_string(),
            name: "Alphabet Master".to_string(),
            description: "Master the alphabet domain".to_string(),
            category: AchievementCategory::Mastery,
            icon: "🔤".to_string(),
            points: 100,
            rarity: Rarity::Uncommon,
            unlocked: false,
            unlocked_at: None,
            progress: 0.0,
            requirement: "Complete alphabet with 95% accuracy".to_string(),
        });
        
        // Explorer achievements
        self.add_achievement(Achievement {
            id: "curious_cat".to_string(),
            name: "Curious Cat".to_string(),
            description: "Try all learning domains".to_string(),
            category: AchievementCategory::Explorer,
            icon: "🐱".to_string(),
            points: 75,
            rarity: Rarity::Uncommon,
            unlocked: false,
            unlocked_at: None,
            progress: 0.0,
            requirement: "Try all 4 domains".to_string(),
        });
        
        // Social achievements
        self.add_achievement(Achievement {
            id: "leaderboard_top10".to_string(),
            name: "Elite Player".to_string(),
            description: "Reach top 10 on any leaderboard".to_string(),
            category: AchievementCategory::Social,
            icon: "🏆".to_string(),
            points: 200,
            rarity: Rarity::Rare,
            unlocked: false,
            unlocked_at: None,
            progress: 0.0,
            requirement: "Top 10 leaderboard position".to_string(),
        });
        
        // Special achievements
        self.add_achievement(Achievement {
            id: "early_bird".to_string(),
            name: "Early Bird".to_string(),
            description: "Practice before 6 AM".to_string(),
            category: AchievementCategory::Special,
            icon: "🌅".to_string(),
            points: 50,
            rarity: Rarity::Common,
            unlocked: false,
            unlocked_at: None,
            progress: 0.0,
            requirement: "Practice before 6 AM".to_string(),
        });
        
        self.add_achievement(Achievement {
            id: "night_owl".to_string(),
            name: "Night Owl".to_string(),
            description: "Practice after midnight".to_string(),
            category: AchievementCategory::Special,
            icon: "🦉".to_string(),
            points: 50,
            rarity: Rarity::Common,
            unlocked: false,
            unlocked_at: None,
            progress: 0.0,
            requirement: "Practice after midnight".to_string(),
        });
    }
    
    fn add_achievement(&mut self, achievement: Achievement) {
        self.achievements.insert(achievement.id.clone(), achievement);
    }
    
    pub fn check_achievements(&mut self, user_id: &str, event: &AchievementEvent) -> Vec<Achievement> {
        let mut unlocked = Vec::new();
        
        // Create profile if it doesn't exist
        let user_key = user_id.to_string();
        if !self.user_profiles.contains_key(&user_key) {
            let new_profile = self.create_new_profile(user_id);
            self.user_profiles.insert(user_key.clone(), new_profile);
        }

        // Check achievements without borrowing profile mutably
        let mut achievements_to_unlock = Vec::new();
        {
            let profile = self.user_profiles.get(&user_key).unwrap();
            for achievement in self.achievements.values() {
                if !achievement.unlocked && self.check_achievement_condition_readonly(achievement, event, profile) {
                    achievements_to_unlock.push(achievement.id.clone());
                }
            }
        }

        // Now unlock achievements and update profile
        for achievement_id in achievements_to_unlock {
            if let Some(achievement) = self.achievements.get_mut(&achievement_id) {
                achievement.unlocked = true;
                achievement.unlocked_at = Some(Utc::now());
                achievement.progress = 1.0;
                
                unlocked.push(achievement.clone());
            }
        }

        // Update profile points and level
        if let Some(profile) = self.user_profiles.get_mut(&user_key) {
            for achievement in &unlocked {
                let points = (achievement.points as f32 * achievement.rarity.multiplier()) as u32;
                profile.total_points += points;
            }
            Self::update_level_static(profile);
        }
        
        unlocked
    }
    
    fn check_achievement_condition_readonly(&self, achievement: &Achievement, event: &AchievementEvent, profile: &GamificationProfile) -> bool {
        self.check_achievement_condition(achievement, event, profile)
    }

    fn check_achievement_condition(&self, achievement: &Achievement, event: &AchievementEvent, profile: &GamificationProfile) -> bool {
        match (&achievement.id[..], event) {
            ("streak_7", AchievementEvent::DailyPractice) => profile.streak.current >= 7,
            ("streak_30", AchievementEvent::DailyPractice) => profile.streak.current >= 30,
            ("accuracy_90", AchievementEvent::SessionComplete { accuracy, .. }) => *accuracy >= 0.9,
            ("perfect_session", AchievementEvent::SessionComplete { accuracy, .. }) => *accuracy >= 1.0,
            ("century", AchievementEvent::TaskComplete { total_tasks, .. }) => *total_tasks >= 100,
            ("millennium", AchievementEvent::TaskComplete { total_tasks, .. }) => *total_tasks >= 1000,
            _ => false,
        }
    }
    
    fn update_level(&mut self, profile: &mut GamificationProfile) {
        Self::update_level_static(profile);
    }

    fn update_level_static(profile: &mut GamificationProfile) {
        // XP required for next level: 100 * level^1.5
        let xp_for_next = (100.0 * (profile.level as f32 + 1.0).powf(1.5)) as u32;
        
        while profile.experience >= xp_for_next {
            profile.level += 1;
            profile.experience -= xp_for_next;
            profile.rank = Rank::from_level(profile.level);
        }
    }
    
    fn create_new_profile(&self, user_id: &str) -> GamificationProfile {
        GamificationProfile {
            user_id: user_id.to_string(),
            display_name: format!("User {}", &user_id[..8]),
            avatar_url: None,
            level: 1,
            experience: 0,
            total_points: 0,
            rank: Rank::from_level(1),
            achievements: Vec::new(),
            badges: Vec::new(),
            streak: StreakInfo {
                current: 0,
                longest: 0,
                last_practice: None,
                freeze_charges: 3,
            },
            statistics: UserStatistics {
                total_sessions: 0,
                total_tasks: 0,
                total_time_minutes: 0,
                average_accuracy: 0.0,
                best_accuracy_session: 0.0,
                fastest_response_ms: u32::MAX,
                domains_mastered: Vec::new(),
            },
            weekly_goals: WeeklyGoals {
                tasks_goal: 100,
                tasks_completed: 0,
                accuracy_goal: 0.8,
                current_accuracy: 0.0,
                streak_goal: 7,
                current_streak: 0,
                bonus_multiplier: 1.0,
            },
            power_ups: Vec::new(),
        }
    }
    
    pub fn get_leaderboard(&self, leaderboard_type: LeaderboardType, limit: usize) -> Vec<LeaderboardEntry> {
        let mut entries: Vec<LeaderboardEntry> = self.user_profiles.values()
            .map(|profile| LeaderboardEntry {
                rank: 0,  // Will be set after sorting
                user_id: profile.user_id.clone(),
                display_name: profile.display_name.clone(),
                avatar_url: profile.avatar_url.clone(),
                score: profile.total_points,
                level: profile.level,
                achievement_count: profile.achievements.len() as u32,
                trend: Trend::Stable,
            })
            .collect();
        
        // Sort by score descending
        entries.sort_by(|a, b| b.score.cmp(&a.score));
        
        // Assign ranks
        for (i, entry) in entries.iter_mut().enumerate() {
            entry.rank = (i + 1) as u32;
        }
        
        entries.truncate(limit);
        entries
    }
    
    pub fn activate_power_up(&mut self, user_id: &str, power_up_id: &str) -> Result<(), String> {
        let profile = self.user_profiles.get_mut(user_id)
            .ok_or("User not found")?;
        
        let power_up = profile.power_ups.iter_mut()
            .find(|p| p.id == power_up_id)
            .ok_or("PowerUp not found")?;
        
        if power_up.active_until.is_some() {
            return Err("PowerUp already active".to_string());
        }
        
        power_up.active_until = Some(Utc::now() + power_up.duration);
        Ok(())
    }
}

/// Events that can trigger achievement checks
#[derive(Debug, Clone)]
pub enum AchievementEvent {
    DailyPractice,
    SessionComplete { accuracy: f32, duration: Duration },
    TaskComplete { correct: bool, response_time: Duration, total_tasks: u32 },
    DomainMastered { domain: String },
    LeaderboardRankChange { old_rank: u32, new_rank: u32 },
    SpecialCondition { condition: String },
}

// Helper functions for UI
impl Achievement {
    pub fn display_progress(&self) -> String {
        if self.unlocked {
            "✅ Unlocked".to_string()
        } else {
            format!("{:.0}%", self.progress * 100.0)
        }
    }
    
    pub fn rarity_color(&self) -> &str {
        self.rarity.color()
    }
}