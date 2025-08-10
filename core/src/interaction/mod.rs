//! # User Interaction & Tracking
//! 
//! This module provides comprehensive tracking and analysis of user interactions:
//! 
//! - [`InteractionTracker`]: Track mouse, keyboard, and timing data
//! - [`AudioRecorder`]: Record and analyze think-aloud protocols  
//! - [`SensorManager`]: Integration with physiological sensors
//! - Multimodal interaction analysis and pattern detection
//! 
//! ## Interaction Tracking
//! 
//! - **Mouse Events**: Clicks, movements, hover patterns
//! - **Keyboard Events**: Keystrokes, timing, error patterns
//! - **Response Timing**: Precise millisecond-level timing
//! - **Interaction Patterns**: Behavioral signatures and strategies
//! 
//! ## Audio Recording & Analysis
//! 
//! - **Think-Aloud Protocols**: Record participant verbalizations
//! - **Speech Analysis**: Segment and categorize verbal responses
//! - **Cognitive Load**: Infer cognitive load from speech patterns
//! - **Strategy Identification**: Identify learning strategies from verbalizations
//! 
//! ## Sensor Integration
//! 
//! - **Eye Tracking**: Gaze patterns and visual attention
//! - **EEG**: Neural activity during learning tasks
//! - **GSR**: Arousal and stress response measurement
//! - **Heart Rate**: Physiological indicators of cognitive load
//! 
//! ## Multimodal Analysis
//! 
//! - **Data Fusion**: Combine multiple interaction modalities
//! - **Pattern Recognition**: Identify complex behavioral patterns
//! - **Real-time Analysis**: Process interactions as they occur
//! - **Privacy Protection**: Secure handling of sensitive biometric data
//! 
//! ## Quick Start
//! 
//! ```rust
//! use abcdeez_core::interaction::{InteractionTracker, SensorManager};
//! 
//! // Start tracking interactions
//! let mut tracker = InteractionTracker::new();
//! tracker.start_session("session_001");
//! 
//! // Record interaction events
//! tracker.record_mouse_click(x, y, timestamp);
//! tracker.record_keystroke(key, timestamp);
//! 
//! // Analyze interaction patterns
//! let metrics = tracker.analyze_session()?;
//! ```

pub mod tracking;
pub mod audio;
pub mod sensors;

// Re-export main types
pub use tracking::{
    InteractionTracker, InteractionSession, InteractionMetrics,
    MouseEvent, KeystrokeEvent
};
pub use audio::{
    AudioRecorder, AudioSession, AudioMetrics, 
    ThinkAloudAnalyzer, ThinkAloudSegment
};
pub use sensors::{
    SensorManager, SensorConfig, SensorSession, SensorType,
    MockEyeTracker, MockEEGSensor, MockGSRSensor
};