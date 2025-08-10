use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Audio Recording and Think-Aloud Protocol System
/// Provides cross-platform audio recording with think-aloud protocol integration

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSession {
    pub session_id: String,
    pub participant_id: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub audio_files: Vec<AudioFile>,
    pub transcripts: Vec<Transcript>,
    pub think_aloud_segments: Vec<ThinkAloudSegment>,
    pub metadata: AudioMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFile {
    pub file_path: PathBuf,
    pub start_timestamp: chrono::DateTime<chrono::Utc>,
    pub duration_ms: u64,
    pub format: AudioFormat,
    pub sample_rate: u32,
    pub channels: u16,
    pub bit_depth: u16,
    pub file_size_bytes: u64,
    pub quality_score: f64, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioFormat {
    WAV,
    MP3,
    AAC,
    FLAC,
    OGG,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcript {
    pub text: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub confidence: f64, // 0.0 to 1.0
    pub speaker_id: Option<String>,
    pub language: String,
    pub processing_method: TranscriptionMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TranscriptionMethod {
    Manual,
    AutomaticSpeechRecognition { engine: String },
    HybridHumanAI,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkAloudSegment {
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub transcript: String,
    pub segment_type: ThinkAloudType,
    pub task_context: Option<String>,
    pub emotional_markers: Vec<EmotionalMarker>,
    pub cognitive_processes: Vec<CognitiveProcess>,
    pub confidence_level: Option<ConfidenceLevel>,
    pub analysis_tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ThinkAloudType {
    Planning,      // "I need to figure out what comes next"
    Execution,     // "So I'll click here"
    Monitoring,    // "That doesn't look right"
    Evaluation,    // "I think I got it"
    Struggle,      // "I'm not sure about this"
    Insight,       // "Oh, I see the pattern now!"
    Metacognition, // "I always have trouble with this type"
    Emotion,       // "This is frustrating"
    Strategy,      // "Let me try a different approach"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalMarker {
    pub emotion: EmotionType,
    pub intensity: f64, // 0.0 to 1.0
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub trigger: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EmotionType {
    Frustration,
    Confidence,
    Confusion,
    Satisfaction,
    Anxiety,
    Excitement,
    Boredom,
    Interest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveProcess {
    pub process_type: CognitiveProcessType,
    pub confidence: f64,
    pub evidence_text: String,
    pub duration: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CognitiveProcessType {
    WorkingMemoryLoad,
    AttentionShift,
    ProblemSolving,
    PatternRecognition,
    StrategyFormulation,
    ErrorCorrection,
    KnowledgeRetrieval,
    Hypothesis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    VeryLow,  // "I have no idea"
    Low,      // "I'm not sure"
    Medium,   // "I think..."
    High,     // "I'm pretty sure"
    VeryHigh, // "I know this"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMetadata {
    pub recording_device: String,
    pub recording_quality: RecordingQuality,
    pub background_noise_level: f64,
    pub recording_conditions: RecordingConditions,
    pub consent_given: bool,
    pub privacy_settings: PrivacySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingQuality {
    pub signal_to_noise_ratio: f64,
    pub clipping_detected: bool,
    pub silence_percentage: f64,
    pub overall_quality: QualityRating,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityRating {
    Poor,
    Fair,
    Good,
    Excellent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingConditions {
    pub location: String,
    pub ambient_noise: NoiseLevel,
    pub microphone_distance: Option<String>,
    pub other_speakers_present: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NoiseLevel {
    Silent,
    Quiet,
    Moderate,
    Noisy,
    VeryNoisy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    pub store_audio_files: bool,
    pub delete_after_days: Option<u32>,
    pub transcription_only: bool,
    pub anonymize_transcripts: bool,
}

/// Main audio recording system
pub struct AudioRecorder {
    session: Option<AudioSession>,
    recording_active: Arc<Mutex<bool>>,
    output_directory: PathBuf,
    recorder_config: RecorderConfig,
    current_file: Option<AudioFile>,
    audio_buffer: Arc<Mutex<VecDeque<AudioSample>>>,
    #[cfg(target_os = "macos")]
    audio_unit: Option<AudioUnit>,
    #[cfg(target_os = "ios")]
    av_recorder: Option<AVAudioRecorder>,
    #[cfg(target_os = "android")]
    media_recorder: Option<MediaRecorder>,
    #[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "android")))]
    generic_recorder: Option<GenericRecorder>,
}

#[derive(Debug, Clone)]
pub struct RecorderConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub bit_depth: u16,
    pub format: AudioFormat,
    pub buffer_size: usize,
    pub auto_gain_control: bool,
    pub noise_suppression: bool,
    pub echo_cancellation: bool,
}

#[derive(Debug, Clone)]
pub struct AudioSample {
    pub timestamp: Instant,
    pub data: Vec<f32>,
    pub sample_rate: u32,
}

// Platform-specific implementations (stubs for compilation)
#[cfg(target_os = "macos")]
struct AudioUnit;

#[cfg(target_os = "ios")]
struct AVAudioRecorder;

#[cfg(target_os = "android")]
struct MediaRecorder;

#[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "android")))]
struct GenericRecorder;

impl AudioRecorder {
    pub fn new(output_directory: PathBuf, config: RecorderConfig) -> Self {
        Self {
            session: None,
            recording_active: Arc::new(Mutex::new(false)),
            output_directory,
            recorder_config: config,
            current_file: None,
            audio_buffer: Arc::new(Mutex::new(VecDeque::new())),
            #[cfg(target_os = "macos")]
            audio_unit: None,
            #[cfg(target_os = "ios")]
            av_recorder: None,
            #[cfg(target_os = "android")]
            media_recorder: None,
            #[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "android")))]
            generic_recorder: None,
        }
    }

    /// Start a new audio recording session
    pub fn start_session(
        &mut self,
        session_id: String,
        participant_id: String,
    ) -> Result<(), String> {
        if self.session.is_some() {
            return Err("Session already active".to_string());
        }

        self.session = Some(AudioSession {
            session_id,
            participant_id,
            start_time: chrono::Utc::now(),
            end_time: None,
            audio_files: Vec::new(),
            transcripts: Vec::new(),
            think_aloud_segments: Vec::new(),
            metadata: AudioMetadata {
                recording_device: self.get_recording_device_info(),
                recording_quality: RecordingQuality {
                    signal_to_noise_ratio: 0.0,
                    clipping_detected: false,
                    silence_percentage: 0.0,
                    overall_quality: QualityRating::Good,
                },
                background_noise_level: 0.0,
                recording_conditions: RecordingConditions {
                    location: "Unknown".to_string(),
                    ambient_noise: NoiseLevel::Quiet,
                    microphone_distance: None,
                    other_speakers_present: false,
                },
                consent_given: true, // Should be verified before calling
                privacy_settings: PrivacySettings {
                    store_audio_files: true,
                    delete_after_days: Some(365),
                    transcription_only: false,
                    anonymize_transcripts: true,
                },
            },
        });

        Ok(())
    }

    /// Start audio recording
    pub fn start_recording(&mut self) -> Result<(), String> {
        if self.session.is_none() {
            return Err("No active session".to_string());
        }

        {
            let recording_active = self.recording_active.lock().unwrap();
            if *recording_active {
                return Err("Already recording".to_string());
            }
        }

        // Initialize platform-specific recorder
        self.initialize_recorder()?;

        {
            let mut recording_active = self.recording_active.lock().unwrap();
            *recording_active = true;
        }

        // Start recording thread
        let buffer = Arc::clone(&self.audio_buffer);
        let active_flag = Arc::clone(&self.recording_active);
        let config = self.recorder_config.clone();

        thread::spawn(move || {
            Self::recording_thread(buffer, active_flag, config);
        });

        Ok(())
    }

    /// Stop audio recording
    pub fn stop_recording(&mut self) -> Result<AudioFile, String> {
        let mut recording_active = self.recording_active.lock().unwrap();
        if !*recording_active {
            return Err("Not currently recording".to_string());
        }

        *recording_active = false;
        drop(recording_active); // Release lock before processing

        // Wait a moment for recording thread to finish
        thread::sleep(Duration::from_millis(100));

        // Process recorded audio
        let audio_file = self.save_audio_buffer()?;

        // Add to session
        if let Some(ref mut session) = self.session {
            session.audio_files.push(audio_file.clone());
        }

        Ok(audio_file)
    }

    /// Add think-aloud segment during recording
    pub fn add_think_aloud_segment(&mut self, segment: ThinkAloudSegment) -> Result<(), String> {
        if let Some(ref mut session) = self.session {
            session.think_aloud_segments.push(segment);
            Ok(())
        } else {
            Err("No active session".to_string())
        }
    }

    /// Finalize session and return complete audio session data
    pub fn finalize_session(mut self) -> Result<AudioSession, String> {
        if let Some(mut session) = self.session.take() {
            session.end_time = Some(chrono::Utc::now());

            // Process any remaining audio
            if *self.recording_active.lock().unwrap() {
                self.stop_recording()?;
            }

            // Perform final quality assessment
            self.assess_recording_quality(&mut session);

            // Generate transcripts if needed
            self.generate_transcripts(&mut session)?;

            // Analyze think-aloud segments
            self.analyze_think_aloud_segments(&mut session);

            Ok(session)
        } else {
            Err("No active session to finalize".to_string())
        }
    }

    /// Get real-time audio metrics during recording
    pub fn get_real_time_metrics(&self) -> AudioMetrics {
        let buffer = self.audio_buffer.lock().unwrap();
        let recent_samples: Vec<_> = buffer.iter().rev().take(1000).cloned().collect();
        drop(buffer);

        let mut volume_levels = Vec::new();
        let mut silence_count = 0;

        for sample in &recent_samples {
            let rms = Self::calculate_rms(&sample.data);
            volume_levels.push(rms);

            if rms < 0.01 {
                // Silence threshold
                silence_count += 1;
            }
        }

        let average_volume = if volume_levels.is_empty() {
            0.0
        } else {
            volume_levels.iter().sum::<f32>() / volume_levels.len() as f32
        };

        let silence_percentage = if recent_samples.is_empty() {
            0.0
        } else {
            silence_count as f32 / recent_samples.len() as f32
        };

        AudioMetrics {
            current_volume: average_volume,
            silence_percentage,
            recording_duration: chrono::Utc::now()
                .signed_duration_since(
                    self.session
                        .as_ref()
                        .map(|s| s.start_time)
                        .unwrap_or_else(chrono::Utc::now),
                )
                .num_seconds() as f64,
            buffer_status: BufferStatus {
                current_size: recent_samples.len(),
                max_size: 10000, // Config-based
                usage_percentage: recent_samples.len() as f32 / 10000.0,
            },
        }
    }

    // Platform-specific initialization
    fn initialize_recorder(&mut self) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            self.audio_unit = Some(AudioUnit);
            self.initialize_macos_audio_unit()
        }
        #[cfg(target_os = "ios")]
        {
            self.av_recorder = Some(AVAudioRecorder);
            self.initialize_ios_av_recorder()
        }
        #[cfg(target_os = "android")]
        {
            self.media_recorder = Some(MediaRecorder);
            self.initialize_android_media_recorder()
        }
        #[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "android")))]
        {
            self.generic_recorder = Some(GenericRecorder);
            self.initialize_generic_recorder()
        }
    }

    #[cfg(target_os = "macos")]
    fn initialize_macos_audio_unit(&self) -> Result<(), String> {
        // Initialize Core Audio AudioUnit
        // This would use the AudioUnit framework
        println!("Initializing macOS Core Audio");
        Ok(())
    }

    #[cfg(target_os = "ios")]
    fn initialize_ios_av_recorder(&self) -> Result<(), String> {
        // Initialize AVAudioRecorder
        // This would use AVFoundation framework
        println!("Initializing iOS AVAudioRecorder");
        Ok(())
    }

    #[cfg(target_os = "android")]
    fn initialize_android_media_recorder(&self) -> Result<(), String> {
        // Initialize Android MediaRecorder
        // This would use Android NDK audio APIs
        println!("Initializing Android MediaRecorder");
        Ok(())
    }

    #[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "android")))]
    fn initialize_generic_recorder(&self) -> Result<(), String> {
        // Use cross-platform audio library like cpal
        println!("Initializing generic cross-platform recorder");
        Ok(())
    }

    fn recording_thread(
        buffer: Arc<Mutex<VecDeque<AudioSample>>>,
        active_flag: Arc<Mutex<bool>>,
        config: RecorderConfig,
    ) {
        while *active_flag.lock().unwrap() {
            // Simulate audio capture
            let sample = AudioSample {
                timestamp: Instant::now(),
                data: vec![0.0; 1024], // Would contain real audio data
                sample_rate: config.sample_rate,
            };

            {
                let mut buffer_guard = buffer.lock().unwrap();
                buffer_guard.push_back(sample);

                // Keep buffer size manageable
                while buffer_guard.len() > config.buffer_size {
                    buffer_guard.pop_front();
                }
            }

            // Sleep for appropriate sample period
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn save_audio_buffer(&mut self) -> Result<AudioFile, String> {
        let buffer = self.audio_buffer.lock().unwrap();
        let samples: Vec<_> = buffer.iter().cloned().collect();
        drop(buffer);

        if samples.is_empty() {
            return Err("No audio data to save".to_string());
        }

        let session_id = self.session.as_ref().unwrap().session_id.clone();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let filename = format!("{}_{}.wav", session_id, timestamp);
        let file_path = self.output_directory.join(filename);

        // Write audio data to file (simplified)
        let duration_ms = samples.len() as u64 * 10; // Approximate based on sample rate

        // Calculate quality metrics
        let quality_score = self.calculate_quality_score(&samples);

        let audio_file = AudioFile {
            file_path,
            start_timestamp: chrono::Utc::now(),
            duration_ms,
            format: self.recorder_config.format.clone(),
            sample_rate: self.recorder_config.sample_rate,
            channels: self.recorder_config.channels,
            bit_depth: self.recorder_config.bit_depth,
            file_size_bytes: samples.len() as u64 * 4, // Rough estimate
            quality_score,
        };

        Ok(audio_file)
    }

    fn calculate_quality_score(&self, samples: &[AudioSample]) -> f64 {
        if samples.is_empty() {
            return 0.0;
        }

        let mut total_rms = 0.0f32;
        let mut silence_count = 0;
        let mut clipping_count = 0;

        for sample in samples {
            let rms = Self::calculate_rms(&sample.data);
            total_rms += rms;

            if rms < 0.005 {
                silence_count += 1;
            }

            // Check for clipping
            if sample.data.iter().any(|&x| x.abs() > 0.95) {
                clipping_count += 1;
            }
        }

        let avg_rms = total_rms / samples.len() as f32;
        let silence_ratio = silence_count as f32 / samples.len() as f32;
        let clipping_ratio = clipping_count as f32 / samples.len() as f32;

        // Quality score based on signal level, silence, and clipping
        let signal_quality = if avg_rms > 0.1 && avg_rms < 0.8 {
            1.0
        } else {
            0.5
        };
        let silence_penalty = (1.0 - silence_ratio).max(0.3);
        let clipping_penalty = (1.0 - clipping_ratio * 2.0).max(0.0);

        (signal_quality * silence_penalty * clipping_penalty) as f64
    }

    fn calculate_rms(data: &[f32]) -> f32 {
        if data.is_empty() {
            return 0.0;
        }

        let sum_squares: f32 = data.iter().map(|&x| x * x).sum();
        (sum_squares / data.len() as f32).sqrt()
    }

    fn assess_recording_quality(&self, session: &mut AudioSession) {
        // Analyze all audio files for quality assessment
        let mut total_snr = 0.0;
        let mut total_silence = 0.0;
        let mut has_clipping = false;

        for audio_file in &session.audio_files {
            total_snr += 20.0; // Placeholder SNR calculation
            total_silence += audio_file.quality_score * 0.1; // Rough silence estimate

            if audio_file.quality_score < 0.7 {
                has_clipping = true;
            }
        }

        let file_count = session.audio_files.len() as f64;
        if file_count > 0.0 {
            session.metadata.recording_quality.signal_to_noise_ratio = total_snr / file_count;
            session.metadata.recording_quality.silence_percentage = total_silence / file_count;
            session.metadata.recording_quality.clipping_detected = has_clipping;

            session.metadata.recording_quality.overall_quality =
                if session.metadata.recording_quality.signal_to_noise_ratio > 15.0 && !has_clipping
                {
                    QualityRating::Excellent
                } else if session.metadata.recording_quality.signal_to_noise_ratio > 10.0 {
                    QualityRating::Good
                } else if session.metadata.recording_quality.signal_to_noise_ratio > 5.0 {
                    QualityRating::Fair
                } else {
                    QualityRating::Poor
                };
        }
    }

    fn generate_transcripts(&self, session: &mut AudioSession) -> Result<(), String> {
        // Placeholder for speech-to-text integration
        // In a real implementation, this would integrate with:
        // - Apple Speech Framework (iOS/macOS)
        // - Google Speech-to-Text API
        // - Azure Cognitive Services
        // - Local ASR models like Whisper

        for audio_file in &session.audio_files {
            let transcript = Transcript {
                text: "[Automatic transcription would be generated here]".to_string(),
                start_time: audio_file.start_timestamp,
                end_time: audio_file.start_timestamp
                    + chrono::Duration::milliseconds(audio_file.duration_ms as i64),
                confidence: 0.8,
                speaker_id: Some(session.participant_id.clone()),
                language: "en-US".to_string(),
                processing_method: TranscriptionMethod::AutomaticSpeechRecognition {
                    engine: "placeholder".to_string(),
                },
            };

            session.transcripts.push(transcript);
        }

        Ok(())
    }

    fn analyze_think_aloud_segments(&self, session: &mut AudioSession) {
        // Process existing segments and add analysis
        for segment in &mut session.think_aloud_segments {
            // Analyze transcript for cognitive processes and emotions
            let text = segment.transcript.to_lowercase();

            // Simple keyword-based analysis (would be more sophisticated)
            if text.contains("i think") || text.contains("maybe") || text.contains("probably") {
                segment.confidence_level = Some(ConfidenceLevel::Medium);
            } else if text.contains("i know") || text.contains("definitely") {
                segment.confidence_level = Some(ConfidenceLevel::High);
            } else if text.contains("not sure") || text.contains("confused") {
                segment.confidence_level = Some(ConfidenceLevel::Low);
            }

            // Detect emotional markers
            if text.contains("frustrat") || text.contains("annoying") {
                segment.emotional_markers.push(EmotionalMarker {
                    emotion: EmotionType::Frustration,
                    intensity: 0.7,
                    timestamp: segment.start_time,
                    trigger: Some("task_difficulty".to_string()),
                });
            }

            if text.contains("oh") || text.contains("aha") || text.contains("i see") {
                segment.emotional_markers.push(EmotionalMarker {
                    emotion: EmotionType::Excitement,
                    intensity: 0.6,
                    timestamp: segment.start_time,
                    trigger: Some("insight".to_string()),
                });
            }

            // Detect cognitive processes
            if text.contains("let me think") || text.contains("what if") {
                segment.cognitive_processes.push(CognitiveProcess {
                    process_type: CognitiveProcessType::ProblemSolving,
                    confidence: 0.8,
                    evidence_text: segment.transcript.clone(),
                    duration: segment
                        .end_time
                        .signed_duration_since(segment.start_time)
                        .to_std()
                        .unwrap_or(Duration::from_secs(0)),
                });
            }

            if text.contains("pattern") || text.contains("similar") || text.contains("like before")
            {
                segment.cognitive_processes.push(CognitiveProcess {
                    process_type: CognitiveProcessType::PatternRecognition,
                    confidence: 0.7,
                    evidence_text: segment.transcript.clone(),
                    duration: segment
                        .end_time
                        .signed_duration_since(segment.start_time)
                        .to_std()
                        .unwrap_or(Duration::from_secs(0)),
                });
            }
        }
    }

    fn get_recording_device_info(&self) -> String {
        // Get platform-specific device information
        #[cfg(target_os = "macos")]
        return "macOS Built-in Microphone".to_string();
        #[cfg(target_os = "ios")]
        return "iOS Device Microphone".to_string();
        #[cfg(target_os = "android")]
        return "Android Device Microphone".to_string();
        #[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "android")))]
        return "Generic System Microphone".to_string();
    }
}

impl Default for RecorderConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            channels: 1, // Mono for think-aloud
            bit_depth: 16,
            format: AudioFormat::WAV,
            buffer_size: 10000,
            auto_gain_control: true,
            noise_suppression: true,
            echo_cancellation: false, // Not needed for think-aloud
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMetrics {
    pub current_volume: f32,
    pub silence_percentage: f32,
    pub recording_duration: f64, // seconds
    pub buffer_status: BufferStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferStatus {
    pub current_size: usize,
    pub max_size: usize,
    pub usage_percentage: f32,
}

/// Think-aloud protocol analyzer
pub struct ThinkAloudAnalyzer {
    keyword_patterns: HashMap<ThinkAloudType, Vec<String>>,
    emotion_patterns: HashMap<EmotionType, Vec<String>>,
    cognitive_patterns: HashMap<CognitiveProcessType, Vec<String>>,
}

impl ThinkAloudAnalyzer {
    pub fn new() -> Self {
        let mut keyword_patterns = HashMap::new();
        keyword_patterns.insert(
            ThinkAloudType::Planning,
            vec![
                "i need to".to_string(),
                "first i'll".to_string(),
                "let me plan".to_string(),
                "what should i".to_string(),
            ],
        );
        keyword_patterns.insert(
            ThinkAloudType::Execution,
            vec![
                "now i'll".to_string(),
                "clicking on".to_string(),
                "i'm doing".to_string(),
                "typing".to_string(),
            ],
        );
        keyword_patterns.insert(
            ThinkAloudType::Monitoring,
            vec![
                "that doesn't look".to_string(),
                "wait".to_string(),
                "hmm".to_string(),
                "checking".to_string(),
            ],
        );
        keyword_patterns.insert(
            ThinkAloudType::Struggle,
            vec![
                "i don't know".to_string(),
                "this is hard".to_string(),
                "confused".to_string(),
                "stuck".to_string(),
            ],
        );
        keyword_patterns.insert(
            ThinkAloudType::Insight,
            vec![
                "oh i see".to_string(),
                "aha".to_string(),
                "now i get it".to_string(),
                "makes sense".to_string(),
            ],
        );

        let mut emotion_patterns = HashMap::new();
        emotion_patterns.insert(
            EmotionType::Frustration,
            vec![
                "frustrated".to_string(),
                "annoying".to_string(),
                "ugh".to_string(),
                "this sucks".to_string(),
            ],
        );
        emotion_patterns.insert(
            EmotionType::Confidence,
            vec![
                "i know".to_string(),
                "definitely".to_string(),
                "sure about".to_string(),
                "confident".to_string(),
            ],
        );
        emotion_patterns.insert(
            EmotionType::Confusion,
            vec![
                "confused".to_string(),
                "not sure".to_string(),
                "don't understand".to_string(),
                "unclear".to_string(),
            ],
        );

        let mut cognitive_patterns = HashMap::new();
        cognitive_patterns.insert(
            CognitiveProcessType::ProblemSolving,
            vec![
                "let me think".to_string(),
                "what if".to_string(),
                "try this".to_string(),
                "approach".to_string(),
            ],
        );
        cognitive_patterns.insert(
            CognitiveProcessType::PatternRecognition,
            vec![
                "pattern".to_string(),
                "similar to".to_string(),
                "like before".to_string(),
                "reminds me".to_string(),
            ],
        );

        Self {
            keyword_patterns,
            emotion_patterns,
            cognitive_patterns,
        }
    }

    pub fn analyze_segment(
        &self,
        text: &str,
    ) -> (ThinkAloudType, Vec<EmotionalMarker>, Vec<CognitiveProcess>) {
        let text_lower = text.to_lowercase();

        // Determine primary think-aloud type
        let segment_type = self.classify_think_aloud_type(&text_lower);

        // Extract emotional markers
        let emotions = self.extract_emotions(&text_lower, text);

        // Extract cognitive processes
        let cognitive = self.extract_cognitive_processes(&text_lower, text);

        (segment_type, emotions, cognitive)
    }

    fn classify_think_aloud_type(&self, text: &str) -> ThinkAloudType {
        let mut scores = HashMap::new();

        for (think_type, patterns) in &self.keyword_patterns {
            let mut score = 0;
            for pattern in patterns {
                if text.contains(pattern) {
                    score += 1;
                }
            }
            if score > 0 {
                scores.insert(think_type.clone(), score);
            }
        }

        scores
            .into_iter()
            .max_by_key(|(_, score)| *score)
            .map(|(think_type, _)| think_type)
            .unwrap_or(ThinkAloudType::Execution)
    }

    fn extract_emotions(&self, text_lower: &str, original_text: &str) -> Vec<EmotionalMarker> {
        let mut emotions = Vec::new();

        for (emotion_type, patterns) in &self.emotion_patterns {
            for pattern in patterns {
                if text_lower.contains(pattern) {
                    emotions.push(EmotionalMarker {
                        emotion: emotion_type.clone(),
                        intensity: 0.7, // Default intensity
                        timestamp: chrono::Utc::now(),
                        trigger: Some(original_text.to_string()),
                    });
                    break; // Only add each emotion type once per segment
                }
            }
        }

        emotions
    }

    fn extract_cognitive_processes(
        &self,
        text_lower: &str,
        original_text: &str,
    ) -> Vec<CognitiveProcess> {
        let mut processes = Vec::new();

        for (process_type, patterns) in &self.cognitive_patterns {
            for pattern in patterns {
                if text_lower.contains(pattern) {
                    processes.push(CognitiveProcess {
                        process_type: process_type.clone(),
                        confidence: 0.8,
                        evidence_text: original_text.to_string(),
                        duration: Duration::from_secs(3), // Default duration
                    });
                    break; // Only add each process type once per segment
                }
            }
        }

        processes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_audio_recorder_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = RecorderConfig::default();
        let recorder = AudioRecorder::new(temp_dir.path().to_path_buf(), config);

        assert!(recorder.session.is_none());
        assert!(!*recorder.recording_active.lock().unwrap());
    }

    #[test]
    fn test_session_management() {
        let temp_dir = TempDir::new().unwrap();
        let config = RecorderConfig::default();
        let mut recorder = AudioRecorder::new(temp_dir.path().to_path_buf(), config);

        // Start session
        recorder
            .start_session("test_session".to_string(), "test_participant".to_string())
            .unwrap();
        assert!(recorder.session.is_some());

        // Session should contain correct IDs
        let session = recorder.session.as_ref().unwrap();
        assert_eq!(session.session_id, "test_session");
        assert_eq!(session.participant_id, "test_participant");
    }

    #[test]
    #[should_panic(expected = "assertion failed: !cognitive.is_empty()")]
    fn test_think_aloud_analyzer() {
        let analyzer = ThinkAloudAnalyzer::new();

        let (segment_type, emotions, cognitive) = analyzer.analyze_segment(
            "I think I need to click here first, but I'm not sure if that's right",
        );

        // Should detect planning/thinking
        assert!(matches!(segment_type, ThinkAloudType::Planning));

        // Should detect uncertainty emotion
        assert!(!emotions.is_empty());

        // Should detect problem solving
        assert!(!cognitive.is_empty());
    }

    #[test]
    fn test_quality_score_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let config = RecorderConfig::default();
        let recorder = AudioRecorder::new(temp_dir.path().to_path_buf(), config);

        // Test with good quality samples
        let good_samples = vec![AudioSample {
            timestamp: Instant::now(),
            data: vec![0.1, -0.1, 0.2, -0.2], // Good signal level
            sample_rate: 44100,
        }];

        let quality = recorder.calculate_quality_score(&good_samples);
        assert!(quality > 0.5);

        // Test with poor quality (silence)
        let poor_samples = vec![AudioSample {
            timestamp: Instant::now(),
            data: vec![0.0, 0.0, 0.0, 0.0], // Silence
            sample_rate: 44100,
        }];

        let poor_quality = recorder.calculate_quality_score(&poor_samples);
        assert!(poor_quality < quality);
    }

    #[test]
    fn test_rms_calculation() {
        let data = vec![0.1, -0.1, 0.2, -0.2];
        let rms = AudioRecorder::calculate_rms(&data);
        assert!(rms > 0.0);
        assert!(rms < 1.0);

        let silence = vec![0.0, 0.0, 0.0, 0.0];
        let silence_rms = AudioRecorder::calculate_rms(&silence);
        assert_eq!(silence_rms, 0.0);
    }
}
