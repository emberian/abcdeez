use serde::{Deserialize, Serialize};

/// Configuration system to address methodological and ecological validity
/// Removes hardcoded priors and allows population-specific adaptations

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerConfig {
    /// Initial uncertainty for new nodes (default: 1.0)
    /// Lower values indicate prior knowledge assumption
    pub initial_uncertainty: f64,

    /// Initial memory strength (default: 0.5)
    /// Higher values assume faster initial learning
    pub initial_memory_strength: f64,

    /// Initial operation proficiency theta (default: 0.0)
    /// Positive values assume prior skill
    pub initial_proficiency: f64,

    /// Base learning rate (default: 0.3)
    /// Population-specific (children might need higher rates)
    pub learning_rate_base: f64,

    /// Learning rate decay exponent (default: 0.5)
    /// Controls how quickly learning slows with practice
    pub learning_rate_decay: f64,

    /// Position update weight for old vs new (default: 0.7)
    /// Higher values = more conservative updating
    pub position_update_weight: f64,

    /// Minimum uncertainty floor (default: 0.1)
    /// Prevents overconfidence in the model
    pub min_uncertainty: f64,

    /// Theta bounds for proficiency (default: (-3.0, 3.0))
    /// May need wider range for expert populations
    pub theta_bounds: (f64, f64),

    /// Memory update on correct response (default: 0.2)
    /// Higher values = faster reinforcement
    pub memory_update_correct: f64,

    /// Memory update on incorrect response (default: -0.1)
    /// More negative = stronger penalty for errors
    pub memory_update_incorrect: f64,

    /// Memory decay rate base (default: 0.05)
    /// Higher values = faster forgetting
    pub memory_decay_rate: f64,

    /// Edge emphasis percentage (default: 0.01)
    /// Primacy/recency effect strength
    pub edge_emphasis: f64,
}

impl LearnerConfig {
    /// Standard adult learner configuration
    pub fn adult() -> Self {
        Self {
            initial_uncertainty: 1.0,
            initial_memory_strength: 0.5,
            initial_proficiency: 0.0,
            learning_rate_base: 0.3,
            learning_rate_decay: 0.5,
            position_update_weight: 0.7,
            min_uncertainty: 0.1,
            theta_bounds: (-3.0, 3.0),
            memory_update_correct: 0.2,
            memory_update_incorrect: -0.1,
            memory_decay_rate: 0.05,
            edge_emphasis: 0.01,
        }
    }

    /// Configuration for child learners (5-12 years)
    pub fn child() -> Self {
        Self {
            initial_uncertainty: 1.5,       // More uncertain initially
            initial_memory_strength: 0.3,   // Weaker initial memory
            initial_proficiency: -0.5,      // Start with lower proficiency
            learning_rate_base: 0.5,        // Faster learning rate
            learning_rate_decay: 0.3,       // Slower decay
            position_update_weight: 0.5,    // More flexible updating
            min_uncertainty: 0.2,           // Higher uncertainty floor
            theta_bounds: (-2.0, 2.0),      // Narrower ability range
            memory_update_correct: 0.3,     // Stronger positive reinforcement
            memory_update_incorrect: -0.05, // Gentler error penalty
            memory_decay_rate: 0.08,        // Faster forgetting
            edge_emphasis: 0.02,            // Stronger primacy/recency
        }
    }

    /// Configuration for older adults (65+)
    pub fn older_adult() -> Self {
        Self {
            initial_uncertainty: 0.8,       // More confident initially
            initial_memory_strength: 0.4,   // Moderate initial memory
            initial_proficiency: 0.2,       // Some prior knowledge assumed
            learning_rate_base: 0.2,        // Slower learning rate
            learning_rate_decay: 0.6,       // Faster decay
            position_update_weight: 0.8,    // More conservative
            min_uncertainty: 0.15,          // Moderate floor
            theta_bounds: (-2.5, 2.5),      // Slightly narrower range
            memory_update_correct: 0.15,    // Slower reinforcement
            memory_update_incorrect: -0.15, // Balanced penalty
            memory_decay_rate: 0.07,        // Moderate forgetting
            edge_emphasis: 0.015,           // Moderate edge effects
        }
    }

    /// Configuration for learners with disabilities
    pub fn learning_disability() -> Self {
        Self {
            initial_uncertainty: 2.0,     // Much more uncertain
            initial_memory_strength: 0.2, // Weaker initial memory
            initial_proficiency: -1.0,    // Lower starting point
            learning_rate_base: 0.4,      // Moderate learning rate
            learning_rate_decay: 0.2,     // Very slow decay
            position_update_weight: 0.4,  // Flexible updating
            min_uncertainty: 0.3,         // Higher floor
            theta_bounds: (-2.0, 2.0),    // Adjusted range
            memory_update_correct: 0.4,   // Strong reinforcement needed
            memory_update_incorrect: 0.0, // No penalty for errors
            memory_decay_rate: 0.1,       // Faster forgetting
            edge_emphasis: 0.03,          // Strong edge effects
        }
    }

    /// Expert learner configuration (domain experts)
    pub fn expert() -> Self {
        Self {
            initial_uncertainty: 0.5,      // Less uncertain
            initial_memory_strength: 0.7,  // Strong initial memory
            initial_proficiency: 1.0,      // High starting proficiency
            learning_rate_base: 0.4,       // Fast refinement
            learning_rate_decay: 0.7,      // Quick stabilization
            position_update_weight: 0.6,   // Balanced updating
            min_uncertainty: 0.05,         // Very low floor
            theta_bounds: (-4.0, 4.0),     // Extended range for expertise
            memory_update_correct: 0.1,    // Small updates (already skilled)
            memory_update_incorrect: -0.2, // Strong error signal
            memory_decay_rate: 0.02,       // Slow forgetting
            edge_emphasis: 0.005,          // Minimal edge effects
        }
    }
}

impl Default for LearnerConfig {
    fn default() -> Self {
        Self::adult()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveSchedulingConfig {
    /// Initial exploration rate (default: 0.15)
    pub initial_epsilon: f64,

    /// Exploration decay rate per trial (default: 0.995)
    pub epsilon_decay: f64,

    /// Minimum exploration rate (default: 0.01)
    pub min_epsilon: f64,

    /// Target success rate for optimal difficulty (default: 0.75)
    pub target_success_rate: f64,

    /// Tolerance around target success rate (default: 0.15)
    pub success_tolerance: f64,

    /// Scoring weights for task selection
    pub scoring_weights: ScoringWeights,

    /// Use Expected Information Gain
    pub use_eig: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringWeights {
    pub difficulty: f64,
    pub uncertainty: f64,
    pub practice_need: f64,
    pub weak_link: f64,
}

impl AdaptiveSchedulingConfig {
    /// Standard configuration
    pub fn standard() -> Self {
        Self {
            initial_epsilon: 0.15,
            epsilon_decay: 0.995,
            min_epsilon: 0.01,
            target_success_rate: 0.75,
            success_tolerance: 0.15,
            scoring_weights: ScoringWeights {
                difficulty: 0.3,
                uncertainty: 0.3,
                practice_need: 0.2,
                weak_link: 0.2,
            },
            use_eig: false,
        }
    }

    /// Exploration-focused (for novel domains)
    pub fn exploratory() -> Self {
        Self {
            initial_epsilon: 0.3,      // Much more exploration
            epsilon_decay: 0.999,      // Slower decay
            min_epsilon: 0.05,         // Higher floor
            target_success_rate: 0.65, // Accept more errors
            success_tolerance: 0.25,   // Wider tolerance
            scoring_weights: ScoringWeights {
                difficulty: 0.2,
                uncertainty: 0.5, // Prioritize uncertain areas
                practice_need: 0.2,
                weak_link: 0.1,
            },
            use_eig: true, // Use information gain
        }
    }

    /// Mastery-focused (for skill development)
    pub fn mastery() -> Self {
        Self {
            initial_epsilon: 0.05,     // Minimal exploration
            epsilon_decay: 0.99,       // Fast decay
            min_epsilon: 0.001,        // Very low floor
            target_success_rate: 0.85, // High success target
            success_tolerance: 0.1,    // Narrow tolerance
            scoring_weights: ScoringWeights {
                difficulty: 0.4, // Focus on appropriate difficulty
                uncertainty: 0.1,
                practice_need: 0.3, // Emphasize practice
                weak_link: 0.2,
            },
            use_eig: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HintInterventionConfig {
    /// Response time threshold for struggle (default: 5000ms)
    pub struggle_rt_threshold_ms: u64,

    /// Consecutive errors before intervention (default: 3)
    pub error_streak_threshold: usize,

    /// Delay before showing hints (default: 3000ms)
    pub hint_delay_ms: u64,

    /// Multiplier for adaptive RT threshold (default: 2.0)
    pub adaptive_rt_multiplier: f64,

    /// Difficulty adjustment thresholds
    pub too_easy_error_rate: f64, // default: 0.1
    pub too_easy_rt_ms: u64,      // default: 2000
    pub too_hard_error_rate: f64, // default: 0.5

    /// Struggle level thresholds
    pub mild_errors: usize, // default: 1
    pub mild_rt_multiplier: f64,     // default: 1.5
    pub moderate_errors: usize,      // default: 3
    pub moderate_rt_multiplier: f64, // default: 2.0
    pub severe_errors: usize,        // default: 5
    pub severe_rt_multiplier: f64,   // default: 3.0
}

impl HintInterventionConfig {
    /// Standard adult configuration
    pub fn standard() -> Self {
        Self {
            struggle_rt_threshold_ms: 5000,
            error_streak_threshold: 3,
            hint_delay_ms: 3000,
            adaptive_rt_multiplier: 2.0,
            too_easy_error_rate: 0.1,
            too_easy_rt_ms: 2000,
            too_hard_error_rate: 0.5,
            mild_errors: 1,
            mild_rt_multiplier: 1.5,
            moderate_errors: 3,
            moderate_rt_multiplier: 2.0,
            severe_errors: 5,
            severe_rt_multiplier: 3.0,
        }
    }

    /// Child-friendly configuration
    pub fn child() -> Self {
        Self {
            struggle_rt_threshold_ms: 3000, // Shorter attention span
            error_streak_threshold: 2,      // Earlier intervention
            hint_delay_ms: 2000,            // Faster hints
            adaptive_rt_multiplier: 1.5,    // Lower threshold
            too_easy_error_rate: 0.05,      // Stricter easy threshold
            too_easy_rt_ms: 1500,
            too_hard_error_rate: 0.4, // More lenient
            mild_errors: 1,
            mild_rt_multiplier: 1.3,
            moderate_errors: 2,
            moderate_rt_multiplier: 1.5,
            severe_errors: 3,
            severe_rt_multiplier: 2.0,
        }
    }

    /// Supportive configuration (learning disabilities)
    pub fn supportive() -> Self {
        Self {
            struggle_rt_threshold_ms: 8000, // Much more time
            error_streak_threshold: 1,      // Immediate support
            hint_delay_ms: 1000,            // Quick hints
            adaptive_rt_multiplier: 3.0,    // Very lenient
            too_easy_error_rate: 0.0,       // Never too easy
            too_easy_rt_ms: 10000,
            too_hard_error_rate: 0.3, // Early adjustment
            mild_errors: 0,           // Always provide support
            mild_rt_multiplier: 1.2,
            moderate_errors: 1,
            moderate_rt_multiplier: 1.5,
            severe_errors: 2,
            severe_rt_multiplier: 2.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainConfig {
    /// Domain-specific difficulty calibrations
    pub task_difficulties: TaskDifficultyConfig,

    /// Domain-specific chunking patterns
    pub chunk_boundaries: Vec<ChunkBoundaryConfig>,

    /// Expected baseline response times
    pub baseline_rt_ms: ResponseTimeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDifficultyConfig {
    pub successor: f64,
    pub predecessor: f64,
    pub k_jump_base: f64,
    pub k_jump_increment: f64,
    pub segment_base: f64,
    pub segment_length_factor: f64,
    pub segment_reverse_penalty: f64,
    pub distance_difficulty_scale: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkBoundaryConfig {
    pub name: String,
    pub positions: Vec<usize>,
    pub strength: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimeConfig {
    pub simple_task: u64,
    pub moderate_task: u64,
    pub complex_task: u64,
}

impl DomainConfig {
    /// Alphabet domain configuration
    pub fn alphabet() -> Self {
        Self {
            task_difficulties: TaskDifficultyConfig {
                successor: 0.3,
                predecessor: 0.4,
                k_jump_base: 0.3,
                k_jump_increment: 0.1,
                segment_base: 0.3,
                segment_length_factor: 0.1,
                segment_reverse_penalty: 0.2,
                distance_difficulty_scale: 0.1,
            },
            chunk_boundaries: vec![
                ChunkBoundaryConfig {
                    name: "Major".to_string(),
                    positions: vec![6, 13, 19], // After F, M, S
                    strength: 0.8,
                },
                ChunkBoundaryConfig {
                    name: "Minor".to_string(),
                    positions: vec![3, 9, 16, 22], // Every ~3 letters
                    strength: 0.4,
                },
                ChunkBoundaryConfig {
                    name: "Vowels".to_string(),
                    positions: vec![0, 4, 8, 14, 20], // A, E, I, O, U
                    strength: 0.3,
                },
            ],
            baseline_rt_ms: ResponseTimeConfig {
                simple_task: 1500,
                moderate_task: 3000,
                complex_task: 5000,
            },
        }
    }

    /// Musical notes configuration
    pub fn music() -> Self {
        Self {
            task_difficulties: TaskDifficultyConfig {
                successor: 0.25, // Easier in octave
                predecessor: 0.35,
                k_jump_base: 0.4, // Intervals are harder
                k_jump_increment: 0.15,
                segment_base: 0.4,
                segment_length_factor: 0.15,
                segment_reverse_penalty: 0.25,
                distance_difficulty_scale: 0.12,
            },
            chunk_boundaries: vec![
                ChunkBoundaryConfig {
                    name: "Octave".to_string(),
                    positions: vec![7], // After full octave
                    strength: 0.9,
                },
                ChunkBoundaryConfig {
                    name: "Tetrachord".to_string(),
                    positions: vec![3], // After first tetrachord
                    strength: 0.5,
                },
            ],
            baseline_rt_ms: ResponseTimeConfig {
                simple_task: 2000,
                moderate_task: 4000,
                complex_task: 6000,
            },
        }
    }

    /// Numbers/mathematics configuration
    pub fn mathematics() -> Self {
        Self {
            task_difficulties: TaskDifficultyConfig {
                successor: 0.2, // Very easy
                predecessor: 0.25,
                k_jump_base: 0.25, // Addition is familiar
                k_jump_increment: 0.05,
                segment_base: 0.2,
                segment_length_factor: 0.05,
                segment_reverse_penalty: 0.15,
                distance_difficulty_scale: 0.05,
            },
            chunk_boundaries: vec![
                ChunkBoundaryConfig {
                    name: "Decade".to_string(),
                    positions: vec![9, 19, 29], // 10, 20, 30
                    strength: 0.7,
                },
                ChunkBoundaryConfig {
                    name: "Five".to_string(),
                    positions: vec![4, 14, 24], // 5, 15, 25
                    strength: 0.4,
                },
            ],
            baseline_rt_ms: ResponseTimeConfig {
                simple_task: 1000,
                moderate_task: 2000,
                complex_task: 3500,
            },
        }
    }
}

/// Complete configuration for a learning system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub learner: LearnerConfig,
    pub adaptive: AdaptiveSchedulingConfig,
    pub hints: HintInterventionConfig,
    pub domain: DomainConfig,
}

impl SystemConfig {
    /// Create a preset configuration
    pub fn preset(population: PopulationType, domain: DomainType, goal: LearningGoal) -> Self {
        let learner = match population {
            PopulationType::Adult => LearnerConfig::adult(),
            PopulationType::Child => LearnerConfig::child(),
            PopulationType::OlderAdult => LearnerConfig::older_adult(),
            PopulationType::LearningDisability => LearnerConfig::learning_disability(),
            PopulationType::Expert => LearnerConfig::expert(),
        };

        let adaptive = match goal {
            LearningGoal::Exploration => AdaptiveSchedulingConfig::exploratory(),
            LearningGoal::Mastery => AdaptiveSchedulingConfig::mastery(),
            LearningGoal::Standard => AdaptiveSchedulingConfig::standard(),
        };

        let hints = match population {
            PopulationType::Child => HintInterventionConfig::child(),
            PopulationType::LearningDisability => HintInterventionConfig::supportive(),
            _ => HintInterventionConfig::standard(),
        };

        let domain = match domain {
            DomainType::Alphabet => DomainConfig::alphabet(),
            DomainType::Music => DomainConfig::music(),
            DomainType::Mathematics => DomainConfig::mathematics(),
        };

        Self {
            learner,
            adaptive,
            hints,
            domain,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PopulationType {
    Adult,
    Child,
    OlderAdult,
    LearningDisability,
    Expert,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DomainType {
    Alphabet,
    Music,
    Mathematics,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LearningGoal {
    Exploration,
    Mastery,
    Standard,
}
