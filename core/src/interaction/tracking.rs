use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::Duration;

/// Comprehensive Interaction Pattern Tracking for Research Applications
/// Captures keystroke dynamics, mouse movements, hesitations, and behavioral patterns

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionSession {
    pub session_id: String,
    pub participant_id: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub keystroke_events: Vec<KeystrokeEvent>,
    pub mouse_events: Vec<MouseEvent>,
    pub focus_events: Vec<FocusEvent>,
    pub scroll_events: Vec<ScrollEvent>,
    pub behavioral_patterns: Vec<BehavioralPattern>,
    pub hesitation_analysis: HesitationAnalysis,
    pub typing_dynamics: TypingDynamicsProfile,
    pub interaction_quality: InteractionQuality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystrokeEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: KeyEventType,
    pub key: String,
    pub key_code: Option<u32>,
    pub dwell_time: Option<Duration>, // Time key was held down
    pub inter_key_interval: Option<Duration>, // Time since previous key event
    pub context: KeystrokeContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyEventType {
    KeyDown,
    KeyUp,
    KeyPress,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystrokeContext {
    pub task_id: Option<String>,
    pub input_field: Option<String>,
    pub cursor_position: Option<usize>,
    pub text_length: Option<usize>,
    pub is_correction: bool, // If this keystroke is correcting a previous error
    pub correction_type: Option<CorrectionType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CorrectionType {
    Backspace,
    Delete,
    Cut,
    Overwrite,
    SelectAndReplace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: MouseEventType,
    pub position: Position,
    pub target_element: Option<String>,
    pub button: Option<MouseButton>,
    pub click_duration: Option<Duration>,
    pub movement_velocity: Option<f64>, // pixels per millisecond
    pub movement_acceleration: Option<f64>,
    pub trajectory_smoothness: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MouseEventType {
    Move,
    Click,
    DoubleClick,
    MouseDown,
    MouseUp,
    RightClick,
    Scroll,
    Hover,
    Leave,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: FocusEventType,
    pub element: String,
    pub focus_duration: Option<Duration>,
    pub tab_order: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FocusEventType {
    Focus,
    Blur,
    FocusIn,
    FocusOut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub scroll_position: Position,
    pub scroll_delta: f64,
    pub scroll_direction: ScrollDirection,
    pub element: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralPattern {
    pub pattern_type: PatternType,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub confidence: f64, // 0.0 to 1.0
    pub evidence: Vec<String>,
    pub context: PatternContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    Hesitation {
        location: HesitationLocation,
        duration: Duration,
    },
    TypingBurst {
        speed: f64,
        accuracy: f64,
    },
    CorrectionBehavior {
        correction_rate: f64,
        strategy: CorrectionStrategy,
    },
    SearchPattern {
        strategy: SearchStrategy,
        efficiency: f64,
    },
    FocusPattern {
        attention_span: Duration,
        focus_switches: usize,
    },
    ProcrastinationBehavior {
        indicators: Vec<ProcrastinationIndicator>,
    },
    ConfidenceIndicator {
        level: ConfidenceLevel,
        indicators: Vec<String>,
    },
    CognitiveLoad {
        level: CognitiveLoadLevel,
        metrics: HashMap<String, f64>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum HesitationLocation {
    BeforeInput,
    DuringInput,
    AfterError,
    BeforeSubmission,
    TaskTransition,
    DecisionPoint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CorrectionStrategy {
    ImmediateBackspace,
    DelayedCorrection,
    SelectAndReplace,
    CompleteRewrite,
    IgnoreError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchStrategy {
    Linear,
    Binary,
    Random,
    Systematic,
    Heuristic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcrastinationIndicator {
    ExtendedPauses,
    RepetitiveActions,
    AvoidanceBehavior,
    OfftaskActivity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CognitiveLoadLevel {
    Low,
    Medium,
    High,
    Overload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternContext {
    pub task_phase: Option<String>,
    pub difficulty_level: Option<f64>,
    pub time_pressure: Option<bool>,
    pub error_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HesitationAnalysis {
    pub total_hesitations: usize,
    pub average_hesitation_duration: Duration,
    pub hesitation_locations: HashMap<HesitationLocation, usize>,
    pub hesitation_triggers: Vec<HesitationTrigger>,
    pub hesitation_patterns: Vec<HesitationPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HesitationTrigger {
    pub trigger_type: TriggerType,
    pub frequency: usize,
    pub average_duration: Duration,
    pub correlation_strength: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerType {
    ComplexTask,
    ErrorOccurred,
    TimeRemaining,
    NewInformation,
    UncertainResponse,
    InterfaceElement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HesitationPattern {
    pub pattern_description: String,
    pub frequency: usize,
    pub typical_duration: Duration,
    pub context_sensitivity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingDynamicsProfile {
    pub words_per_minute: f64,
    pub characters_per_minute: f64,
    pub average_dwell_time: Duration,
    pub average_flight_time: Duration, // Time between keystrokes
    pub typing_rhythm: TypingRhythm,
    pub keystroke_intensity: KeystrokeIntensity,
    pub error_rate: f64,
    pub correction_latency: Duration,
    pub finger_usage_pattern: HashMap<String, usize>,
    pub typing_consistency: f64, // Variability in timing
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingRhythm {
    pub rhythm_score: f64, // Regularity of typing pattern
    pub burst_patterns: Vec<TypingBurst>,
    pub pause_patterns: Vec<TypingPause>,
    pub tempo_changes: Vec<TempoChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingBurst {
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub duration: Duration,
    pub speed: f64, // WPM during burst
    pub accuracy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingPause {
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub duration: Duration,
    pub context: PauseContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PauseContext {
    Thinking,
    Planning,
    Correcting,
    Resting,
    Distracted,
    Technical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TempoChange {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub from_tempo: f64,
    pub to_tempo: f64,
    pub change_rate: f64,
    pub trigger: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystrokeIntensity {
    pub pressure_variation: f64,
    pub timing_precision: f64,
    pub force_consistency: f64,
    pub stress_indicators: Vec<StressIndicator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StressIndicator {
    IncreasedTypingForce,
    IrregularRhythm,
    IncreasedErrors,
    LongerPauses,
    RepetitiveCorrections,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionQuality {
    pub overall_score: f64, // 0.0 to 1.0
    pub engagement_level: EngagementLevel,
    pub attention_consistency: f64,
    pub task_focus: f64,
    pub interaction_efficiency: f64,
    pub error_handling_quality: f64,
    pub learning_indicators: Vec<LearningIndicator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngagementLevel {
    Disengaged,
    Low,
    Medium,
    High,
    VeryHigh,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningIndicator {
    ImprovingSpeed,
    ReducingErrors,
    MoreConfidentInput,
    BetterErrorRecovery,
    AdaptingToInterface,
}

/// Main interaction tracking system
pub struct InteractionTracker {
    session: InteractionSession,
    event_buffer: VecDeque<InteractionEvent>,
    pattern_detector: PatternDetector,
    typing_analyzer: TypingAnalyzer,
    mouse_analyzer: MouseAnalyzer,
    hesitation_detector: HesitationDetector,
}

#[derive(Debug, Clone)]
enum InteractionEvent {
    Keystroke(KeystrokeEvent),
    Mouse(MouseEvent),
    Focus(FocusEvent),
    Scroll(ScrollEvent),
}

impl InteractionTracker {
    pub fn new(session_id: String, participant_id: String) -> Self {
        Self {
            session: InteractionSession {
                session_id,
                participant_id,
                start_time: chrono::Utc::now(),
                end_time: None,
                keystroke_events: Vec::new(),
                mouse_events: Vec::new(),
                focus_events: Vec::new(),
                scroll_events: Vec::new(),
                behavioral_patterns: Vec::new(),
                hesitation_analysis: HesitationAnalysis::default(),
                typing_dynamics: TypingDynamicsProfile::default(),
                interaction_quality: InteractionQuality::default(),
            },
            event_buffer: VecDeque::new(),
            pattern_detector: PatternDetector::new(),
            typing_analyzer: TypingAnalyzer::new(),
            mouse_analyzer: MouseAnalyzer::new(),
            hesitation_detector: HesitationDetector::new(),
        }
    }

    /// Record a keystroke event
    pub fn record_keystroke(
        &mut self,
        event_type: KeyEventType,
        key: String,
        key_code: Option<u32>,
        context: KeystrokeContext,
    ) {
        let timestamp = chrono::Utc::now();

        // Calculate timing metrics
        let inter_key_interval = self
            .session
            .keystroke_events
            .last()
            .map(|last_event| timestamp.signed_duration_since(last_event.timestamp))
            .map(|duration| Duration::from_millis(duration.num_milliseconds().max(0) as u64));

        let keystroke_event = KeystrokeEvent {
            timestamp,
            event_type: event_type.clone(),
            key: key.clone(),
            key_code,
            dwell_time: None, // Would be calculated for KeyUp events
            inter_key_interval,
            context,
        };

        // Add to session and buffer
        self.session.keystroke_events.push(keystroke_event.clone());
        self.event_buffer
            .push_back(InteractionEvent::Keystroke(keystroke_event));

        // Analyze typing patterns in real-time
        self.typing_analyzer
            .analyze_keystroke(&key, &event_type, timestamp);

        // Detect hesitations
        if let Some(interval) = inter_key_interval {
            self.hesitation_detector
                .analyze_interval(interval, timestamp);
        }

        // Update behavioral patterns
        self.update_behavioral_patterns();
    }

    /// Record a mouse event
    pub fn record_mouse_event(
        &mut self,
        event_type: MouseEventType,
        position: Position,
        target_element: Option<String>,
        button: Option<MouseButton>,
    ) {
        let timestamp = chrono::Utc::now();

        // Calculate movement metrics
        let (velocity, acceleration, smoothness) = self
            .mouse_analyzer
            .calculate_movement_metrics(&position, timestamp);

        let mouse_event = MouseEvent {
            timestamp,
            event_type,
            position,
            target_element,
            button,
            click_duration: None,
            movement_velocity: velocity,
            movement_acceleration: acceleration,
            trajectory_smoothness: smoothness,
        };

        self.session.mouse_events.push(mouse_event.clone());
        self.event_buffer
            .push_back(InteractionEvent::Mouse(mouse_event));

        // Update movement analysis
        self.mouse_analyzer.update_movement_patterns(timestamp);
    }

    /// Record a focus event
    pub fn record_focus_event(
        &mut self,
        event_type: FocusEventType,
        element: String,
        tab_order: Option<usize>,
    ) {
        let timestamp = chrono::Utc::now();

        // Calculate focus duration
        let focus_duration =
            if matches!(event_type, FocusEventType::Blur | FocusEventType::FocusOut) {
                self.session
                    .focus_events
                    .iter()
                    .rev()
                    .find(|e| {
                        e.element == element
                            && matches!(
                                e.event_type,
                                FocusEventType::Focus | FocusEventType::FocusIn
                            )
                    })
                    .map(|focus_event| timestamp.signed_duration_since(focus_event.timestamp))
                    .map(
                        |duration| Duration::from_millis(duration.num_milliseconds().max(0) as u64),
                    )
            } else {
                None
            };

        let focus_event = FocusEvent {
            timestamp,
            event_type,
            element,
            focus_duration,
            tab_order,
        };

        self.session.focus_events.push(focus_event.clone());
        self.event_buffer
            .push_back(InteractionEvent::Focus(focus_event));
    }

    /// Record a scroll event
    pub fn record_scroll_event(
        &mut self,
        scroll_position: Position,
        scroll_delta: f64,
        scroll_direction: ScrollDirection,
        element: Option<String>,
    ) {
        let timestamp = chrono::Utc::now();

        let scroll_event = ScrollEvent {
            timestamp,
            scroll_position,
            scroll_delta,
            scroll_direction,
            element,
        };

        self.session.scroll_events.push(scroll_event.clone());
        self.event_buffer
            .push_back(InteractionEvent::Scroll(scroll_event));
    }

    /// Finalize the session and generate comprehensive analysis
    pub fn finalize_session(mut self) -> InteractionSession {
        self.session.end_time = Some(chrono::Utc::now());

        // Generate final analyses
        self.session.hesitation_analysis = self.hesitation_detector.finalize_analysis();
        self.session.typing_dynamics = self.typing_analyzer.generate_profile();
        self.session.interaction_quality = self.calculate_interaction_quality();

        // Detect final behavioral patterns
        self.session.behavioral_patterns = self.pattern_detector.detect_all_patterns(
            &self.session.keystroke_events,
            &self.session.mouse_events,
            &self.session.focus_events,
        );

        self.session
    }

    /// Get real-time interaction metrics
    pub fn get_real_time_metrics(&self) -> InteractionMetrics {
        InteractionMetrics {
            current_typing_speed: self.typing_analyzer.get_current_wpm(),
            recent_error_rate: self.typing_analyzer.get_recent_error_rate(),
            hesitation_frequency: self.hesitation_detector.get_recent_hesitation_rate(),
            focus_stability: self.calculate_focus_stability(),
            engagement_level: self.calculate_current_engagement(),
            cognitive_load: self.estimate_cognitive_load(),
        }
    }

    // Helper methods
    fn update_behavioral_patterns(&mut self) {
        // Process recent events to detect emerging patterns
        let recent_events: Vec<_> = self.event_buffer.iter().rev().take(10).cloned().collect();

        self.pattern_detector.analyze_recent_events(&recent_events);
    }

    fn calculate_interaction_quality(&self) -> InteractionQuality {
        let engagement = self.calculate_current_engagement();
        let attention = self.calculate_focus_stability();
        let efficiency = self.calculate_interaction_efficiency();
        let error_handling = self.calculate_error_handling_quality();

        let overall_score = (engagement + attention + efficiency + error_handling) / 4.0;

        InteractionQuality {
            overall_score,
            engagement_level: if engagement > 0.8 {
                EngagementLevel::VeryHigh
            } else if engagement > 0.6 {
                EngagementLevel::High
            } else if engagement > 0.4 {
                EngagementLevel::Medium
            } else if engagement > 0.2 {
                EngagementLevel::Low
            } else {
                EngagementLevel::Disengaged
            },
            attention_consistency: attention,
            task_focus: self.calculate_task_focus(),
            interaction_efficiency: efficiency,
            error_handling_quality: error_handling,
            learning_indicators: self.detect_learning_indicators(),
        }
    }

    fn calculate_current_engagement(&self) -> f64 {
        // Simple engagement calculation based on activity level
        let recent_events = self.event_buffer.len() as f64;
        let time_window = 60.0; // seconds

        // Normalize activity level
        (recent_events / time_window).min(1.0)
    }

    fn calculate_focus_stability(&self) -> f64 {
        if self.session.focus_events.is_empty() {
            return 0.5; // Neutral score if no focus data
        }

        let focus_switches = self.session.focus_events.len() as f64;
        let session_duration = self
            .session
            .start_time
            .signed_duration_since(chrono::Utc::now())
            .num_seconds() as f64;

        // Lower switch rate indicates better focus
        let switch_rate = focus_switches / (session_duration / 60.0); // switches per minute
        (1.0 / (1.0 + switch_rate * 0.1)).max(0.0).min(1.0)
    }

    fn calculate_interaction_efficiency(&self) -> f64 {
        let typing_speed = self.typing_analyzer.get_current_wpm();
        let error_rate = self.typing_analyzer.get_recent_error_rate();

        // Efficiency combines speed and accuracy
        let speed_score = (typing_speed / 60.0).min(1.0); // Normalize to 60 WPM as max
        let accuracy_score = (1.0 - error_rate).max(0.0);

        (speed_score + accuracy_score) / 2.0
    }

    fn calculate_error_handling_quality(&self) -> f64 {
        // Quality of error correction behavior
        let correction_events: Vec<_> = self
            .session
            .keystroke_events
            .iter()
            .filter(|event| event.context.is_correction)
            .collect();

        if correction_events.is_empty() {
            return 1.0; // No errors to handle
        }

        // Analyze correction latency and effectiveness
        let average_latency = correction_events
            .iter()
            .filter_map(|event| event.inter_key_interval)
            .map(|duration| duration.as_millis() as f64)
            .collect::<Vec<_>>();

        if average_latency.is_empty() {
            return 0.5;
        }

        let mean_latency = average_latency.iter().sum::<f64>() / average_latency.len() as f64;

        // Lower latency indicates better error handling
        (1.0 / (1.0 + mean_latency / 1000.0)).max(0.0).min(1.0)
    }

    fn calculate_task_focus(&self) -> f64 {
        // Measure how much attention is directed at task-relevant elements
        let task_focused_events = self
            .session
            .focus_events
            .iter()
            .filter(|event| self.is_task_relevant_element(&event.element))
            .count() as f64;

        let total_focus_events = self.session.focus_events.len() as f64;

        if total_focus_events == 0.0 {
            0.5
        } else {
            task_focused_events / total_focus_events
        }
    }

    fn is_task_relevant_element(&self, element: &str) -> bool {
        // Simple heuristic - would be more sophisticated in practice
        element.contains("task") || element.contains("input") || element.contains("answer")
    }

    fn detect_learning_indicators(&self) -> Vec<LearningIndicator> {
        let mut indicators = Vec::new();

        // Check for improving typing speed
        if self.typing_analyzer.is_speed_improving() {
            indicators.push(LearningIndicator::ImprovingSpeed);
        }

        // Check for reducing errors
        if self.typing_analyzer.is_error_rate_decreasing() {
            indicators.push(LearningIndicator::ReducingErrors);
        }

        // Check for more confident input (fewer hesitations)
        if self.hesitation_detector.is_confidence_improving() {
            indicators.push(LearningIndicator::MoreConfidentInput);
        }

        indicators
    }

    fn estimate_cognitive_load(&self) -> CognitiveLoadLevel {
        let hesitation_rate = self.hesitation_detector.get_recent_hesitation_rate();
        let error_rate = self.typing_analyzer.get_recent_error_rate();
        let typing_irregularity = self.typing_analyzer.get_typing_irregularity();

        let load_score = (hesitation_rate + error_rate + typing_irregularity) / 3.0;

        if load_score > 0.8 {
            CognitiveLoadLevel::Overload
        } else if load_score > 0.6 {
            CognitiveLoadLevel::High
        } else if load_score > 0.4 {
            CognitiveLoadLevel::Medium
        } else {
            CognitiveLoadLevel::Low
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionMetrics {
    pub current_typing_speed: f64,
    pub recent_error_rate: f64,
    pub hesitation_frequency: f64,
    pub focus_stability: f64,
    pub engagement_level: f64,
    pub cognitive_load: CognitiveLoadLevel,
}

// Supporting analyzer classes
struct PatternDetector {
    detected_patterns: Vec<BehavioralPattern>,
    pattern_threshold: f64,
}

impl PatternDetector {
    fn new() -> Self {
        Self {
            detected_patterns: Vec::new(),
            pattern_threshold: 0.7,
        }
    }

    fn analyze_recent_events(&mut self, events: &[InteractionEvent]) {
        // Use pattern_threshold to determine if patterns are significant
        for pattern in self.detect_patterns_in_events(events) {
            if pattern.confidence >= self.pattern_threshold {
                self.detected_patterns.push(pattern);
            }
        }
    }

    fn detect_patterns_in_events(&self, events: &[InteractionEvent]) -> Vec<BehavioralPattern> {
        let mut patterns = Vec::new();
        
        // Detect hesitation patterns
        let hesitation_events: Vec<_> = events.iter()
            .filter_map(|e| if let InteractionEvent::Keystroke(ke) = e {
                ke.inter_key_interval.filter(|d| d.as_millis() > 1500).map(|d| (ke.timestamp, d))
            } else { None })
            .collect();
        
        if hesitation_events.len() >= 3 {
            patterns.push(BehavioralPattern {
                pattern_type: PatternType::Hesitation {
                    location: HesitationLocation::DuringInput,
                    duration: hesitation_events.iter().map(|(_, d)| *d).sum::<std::time::Duration>(),
                },
                start_time: hesitation_events.first().unwrap().0,
                end_time: hesitation_events.last().unwrap().0,
                confidence: if hesitation_events.len() > 5 { 0.9 } else { 0.7 },
                evidence: vec![format!("Detected {} hesitations in recent events", hesitation_events.len())],
                context: PatternContext {
                    task_phase: None,
                    difficulty_level: Some(0.7),
                    time_pressure: None,
                    error_rate: None,
                },
            });
        }
        
        patterns
    }

    fn detect_all_patterns(
        &mut self,
        keystroke_events: &[KeystrokeEvent],
        mouse_events: &[MouseEvent],
        focus_events: &[FocusEvent],
    ) -> Vec<BehavioralPattern> {
        // Comprehensive pattern detection across all event types
        self.detected_patterns.clone()
    }
}

struct TypingAnalyzer {
    recent_keystrokes: VecDeque<KeystrokeEvent>,
    typing_speed_history: VecDeque<f64>,
    error_history: VecDeque<bool>,
    typing_rhythm_buffer: VecDeque<Duration>,
}

impl TypingAnalyzer {
    fn new() -> Self {
        Self {
            recent_keystrokes: VecDeque::new(),
            typing_speed_history: VecDeque::new(),
            error_history: VecDeque::new(),
            typing_rhythm_buffer: VecDeque::new(),
        }
    }

    fn analyze_keystroke(
        &mut self,
        key: &str,
        event_type: &KeyEventType,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) {
        // Create keystroke event for analysis
        let keystroke_event = KeystrokeEvent {
            timestamp,
            event_type: event_type.clone(),
            key: key.to_string(),
            key_code: None,
            dwell_time: None,
            inter_key_interval: self.recent_keystrokes.back()
                .map(|last| timestamp.signed_duration_since(last.timestamp))
                .map(|d| Duration::from_millis(d.num_milliseconds().max(0) as u64)),
            context: KeystrokeContext {
                task_id: None,
                input_field: None,
                cursor_position: None,
                text_length: None,
                is_correction: self.is_correction_keystroke(key),
                correction_type: None,
            },
        };
        
        // Store interval before moving keystroke_event
        let inter_key_interval = keystroke_event.inter_key_interval;
        
        // Add to recent keystrokes buffer
        self.recent_keystrokes.push_back(keystroke_event);
        
        // Keep only recent keystrokes (last 50)
        if self.recent_keystrokes.len() > 50 {
            self.recent_keystrokes.pop_front();
        }
        
        // Calculate typing speed based on recent keystrokes
        if self.recent_keystrokes.len() >= 10 {
            let time_window = timestamp.signed_duration_since(
                self.recent_keystrokes.front().unwrap().timestamp
            );
            let duration_minutes = time_window.num_seconds() as f64 / 60.0;
            
            if duration_minutes > 0.0 {
                let words_estimate = self.recent_keystrokes.len() as f64 / 5.0; // 5 chars per word
                let wpm = words_estimate / duration_minutes;
                self.typing_speed_history.push_back(wpm);
                
                // Keep speed history limited
                if self.typing_speed_history.len() > 20 {
                    self.typing_speed_history.pop_front();
                }
            }
        }
        
        // Track errors
        let is_error = self.is_error_keystroke(key);
        self.error_history.push_back(is_error);
        if self.error_history.len() > 100 {
            self.error_history.pop_front();
        }
        
        // Track typing rhythm
        if let Some(interval) = inter_key_interval {
            self.typing_rhythm_buffer.push_back(interval);
            if self.typing_rhythm_buffer.len() > 30 {
                self.typing_rhythm_buffer.pop_front();
            }
        }
    }
    
    fn is_correction_keystroke(&self, key: &str) -> bool {
        matches!(key, "Backspace" | "Delete" | "ArrowLeft" | "ArrowRight")
    }
    
    fn is_error_keystroke(&self, key: &str) -> bool {
        // Simple heuristic - backspace often indicates error
        key == "Backspace"
    }

    fn get_current_wpm(&self) -> f64 {
        self.typing_speed_history.back().cloned().unwrap_or(0.0)
    }

    fn get_recent_error_rate(&self) -> f64 {
        if self.error_history.is_empty() {
            return 0.0;
        }

        let errors = self.error_history.iter().filter(|&&e| e).count();
        errors as f64 / self.error_history.len() as f64
    }

    fn is_speed_improving(&self) -> bool {
        if self.typing_speed_history.len() < 2 {
            return false;
        }

        let recent_speeds: Vec<_> = self
            .typing_speed_history
            .iter()
            .rev()
            .take(5)
            .cloned()
            .collect();

        // Simple trend analysis
        recent_speeds.first() > recent_speeds.last()
    }

    fn is_error_rate_decreasing(&self) -> bool {
        // Similar trend analysis for error rate
        true // Placeholder
    }

    fn get_typing_irregularity(&self) -> f64 {
        if self.typing_rhythm_buffer.len() < 2 {
            return 0.0;
        }

        let intervals: Vec<f64> = self
            .typing_rhythm_buffer
            .iter()
            .map(|d| d.as_millis() as f64)
            .collect();

        // Calculate coefficient of variation
        let mean = intervals.iter().sum::<f64>() / intervals.len() as f64;
        let variance =
            intervals.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / intervals.len() as f64;
        let std_dev = variance.sqrt();

        if mean == 0.0 {
            0.0
        } else {
            std_dev / mean
        }
    }

    fn generate_profile(&self) -> TypingDynamicsProfile {
        TypingDynamicsProfile::default()
    }
}

struct MouseAnalyzer {
    recent_positions: VecDeque<Position>,
    movement_history: VecDeque<f64>,
}

impl MouseAnalyzer {
    fn new() -> Self {
        Self {
            recent_positions: VecDeque::new(),
            movement_history: VecDeque::new(),
        }
    }

    fn calculate_movement_metrics(
        &mut self,
        position: &Position,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> (Option<f64>, Option<f64>, Option<f64>) {
        // Calculate velocity, acceleration, and smoothness
        self.recent_positions.push_back(position.clone());

        if self.recent_positions.len() > 10 {
            self.recent_positions.pop_front();
        }

        // Calculate actual movement metrics if we have enough data
        if self.recent_positions.len() >= 2 {
            let current_pos = self.recent_positions.back().unwrap();
            let prev_pos = &self.recent_positions[self.recent_positions.len() - 2];
            
            // Calculate distance and time difference
            let dx = current_pos.x - prev_pos.x;
            let dy = current_pos.y - prev_pos.y;
            let distance = (dx * dx + dy * dy).sqrt();
            
            let time_diff = current_pos.timestamp.signed_duration_since(prev_pos.timestamp);
            let time_ms = time_diff.num_milliseconds().max(1) as f64;
            
            let velocity = distance / time_ms; // pixels per millisecond
            self.movement_history.push_back(velocity);
            
            // Keep history limited
            if self.movement_history.len() > 20 {
                self.movement_history.pop_front();
            }
            
            // Calculate acceleration if we have velocity history
            let acceleration = if self.movement_history.len() >= 2 {
                let current_vel = *self.movement_history.back().unwrap();
                let prev_vel = self.movement_history[self.movement_history.len() - 2];
                Some((current_vel - prev_vel) / time_ms)
            } else {
                None
            };
            
            // Calculate smoothness based on velocity variations
            let smoothness = if self.movement_history.len() >= 3 {
                let velocities: Vec<f64> = self.movement_history.iter().cloned().collect();
                let mean_velocity = velocities.iter().sum::<f64>() / velocities.len() as f64;
                let variance = velocities.iter()
                    .map(|v| (v - mean_velocity).powi(2))
                    .sum::<f64>() / velocities.len() as f64;
                Some(1.0 / (1.0 + variance.sqrt())) // Higher smoothness = lower variance
            } else {
                None
            };
            
            (Some(velocity), acceleration, smoothness)
        } else {
            (None, None, None)
        }
    }

    fn update_movement_patterns(&mut self, timestamp: chrono::DateTime<chrono::Utc>) {
        // Update movement pattern analysis
    }
}

struct HesitationDetector {
    hesitation_threshold: Duration,
    recent_hesitations: VecDeque<Duration>,
    hesitation_contexts: Vec<HesitationContext>,
}

#[derive(Debug, Clone)]
struct HesitationContext {
    timestamp: chrono::DateTime<chrono::Utc>,
    duration: Duration,
    location: HesitationLocation,
    trigger: Option<TriggerType>,
}

impl HesitationDetector {
    fn new() -> Self {
        Self {
            hesitation_threshold: Duration::from_millis(1500), // 1.5 seconds
            recent_hesitations: VecDeque::new(),
            hesitation_contexts: Vec::new(),
        }
    }

    fn analyze_interval(&mut self, interval: Duration, timestamp: chrono::DateTime<chrono::Utc>) {
        if interval > self.hesitation_threshold {
            self.recent_hesitations.push_back(interval);

            // Determine hesitation location and trigger
            let location = self.classify_hesitation_location(timestamp);
            let trigger = self.identify_hesitation_trigger(interval);

            self.hesitation_contexts.push(HesitationContext {
                timestamp,
                duration: interval,
                location,
                trigger,
            });
        }

        // Keep only recent hesitations
        if self.recent_hesitations.len() > 20 {
            self.recent_hesitations.pop_front();
        }
    }

    fn get_recent_hesitation_rate(&self) -> f64 {
        self.recent_hesitations.len() as f64 / 20.0 // Normalize to window size
    }

    fn is_confidence_improving(&self) -> bool {
        if self.hesitation_contexts.len() < 10 {
            return false;
        }

        let recent_hesitations = self
            .hesitation_contexts
            .iter()
            .rev()
            .take(5)
            .map(|h| h.duration.as_millis())
            .collect::<Vec<_>>();

        let earlier_hesitations = self
            .hesitation_contexts
            .iter()
            .rev()
            .skip(5)
            .take(5)
            .map(|h| h.duration.as_millis())
            .collect::<Vec<_>>();

        if recent_hesitations.is_empty() || earlier_hesitations.is_empty() {
            return false;
        }

        let recent_avg: f64 =
            recent_hesitations.iter().sum::<u128>() as f64 / recent_hesitations.len() as f64;
        let earlier_avg: f64 =
            earlier_hesitations.iter().sum::<u128>() as f64 / earlier_hesitations.len() as f64;

        recent_avg < earlier_avg
    }

    fn finalize_analysis(&self) -> HesitationAnalysis {
        HesitationAnalysis::default()
    }

    fn classify_hesitation_location(
        &self,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> HesitationLocation {
        // Classify based on context - placeholder
        HesitationLocation::DuringInput
    }

    fn identify_hesitation_trigger(&self, duration: Duration) -> Option<TriggerType> {
        // Identify what triggered the hesitation - placeholder
        Some(TriggerType::UncertainResponse)
    }
}

// Default implementations
impl Default for HesitationAnalysis {
    fn default() -> Self {
        Self {
            total_hesitations: 0,
            average_hesitation_duration: Duration::from_millis(0),
            hesitation_locations: HashMap::new(),
            hesitation_triggers: Vec::new(),
            hesitation_patterns: Vec::new(),
        }
    }
}

impl Default for TypingDynamicsProfile {
    fn default() -> Self {
        Self {
            words_per_minute: 0.0,
            characters_per_minute: 0.0,
            average_dwell_time: Duration::from_millis(100),
            average_flight_time: Duration::from_millis(200),
            typing_rhythm: TypingRhythm {
                rhythm_score: 0.5,
                burst_patterns: Vec::new(),
                pause_patterns: Vec::new(),
                tempo_changes: Vec::new(),
            },
            keystroke_intensity: KeystrokeIntensity {
                pressure_variation: 0.0,
                timing_precision: 0.5,
                force_consistency: 0.5,
                stress_indicators: Vec::new(),
            },
            error_rate: 0.0,
            correction_latency: Duration::from_millis(1000),
            finger_usage_pattern: HashMap::new(),
            typing_consistency: 0.5,
        }
    }
}

impl Default for InteractionQuality {
    fn default() -> Self {
        Self {
            overall_score: 0.5,
            engagement_level: EngagementLevel::Medium,
            attention_consistency: 0.5,
            task_focus: 0.5,
            interaction_efficiency: 0.5,
            error_handling_quality: 0.5,
            learning_indicators: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interaction_tracking() {
        let mut tracker =
            InteractionTracker::new("session_123".to_string(), "participant_456".to_string());

        // Record some keystroke events
        tracker.record_keystroke(
            KeyEventType::KeyDown,
            "a".to_string(),
            Some(65),
            KeystrokeContext {
                task_id: Some("task_1".to_string()),
                input_field: Some("answer".to_string()),
                cursor_position: Some(0),
                text_length: Some(0),
                is_correction: false,
                correction_type: None,
            },
        );

        // Record a mouse event
        tracker.record_mouse_event(
            MouseEventType::Click,
            Position {
                x: 100.0,
                y: 200.0,
                timestamp: chrono::Utc::now(),
            },
            Some("submit_button".to_string()),
            Some(MouseButton::Left),
        );

        // Get real-time metrics
        let metrics = tracker.get_real_time_metrics();
        assert!(metrics.current_typing_speed >= 0.0);

        // Finalize session
        let session = tracker.finalize_session();
        assert!(!session.keystroke_events.is_empty());
        assert!(!session.mouse_events.is_empty());
    }
}
