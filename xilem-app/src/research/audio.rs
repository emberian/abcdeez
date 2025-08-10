use chrono::{DateTime, Utc};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Sample, SampleFormat, SampleRate, StreamConfig};
use hound::{WavSpec, WavWriter};
use ringbuf::{Consumer, Producer};
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Audio recorder using CPAL for cross-platform microphone capture
pub struct AudioRecorder {
    host: Host,
    input_device: Option<Device>,
    recording_thread: Option<thread::JoinHandle<Result<(), AudioError>>>,
    is_recording: Arc<Mutex<bool>>,
    sample_rate: u32,
    channels: u16,
}

#[derive(Debug, Clone)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub buffer_size: usize,
    pub format: AudioFormat,
}

#[derive(Debug, Clone, Copy)]
pub enum AudioFormat {
    F32,
    I16,
    U16,
}

#[derive(Debug)]
pub enum AudioError {
    DeviceNotFound,
    ConfigNotSupported,
    StreamError(String),
    FileError(String),
    RecordingInProgress,
    NotRecording,
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AudioError::DeviceNotFound => write!(f, "Audio input device not found"),
            AudioError::ConfigNotSupported => write!(f, "Audio configuration not supported"),
            AudioError::StreamError(e) => write!(f, "Audio stream error: {}", e),
            AudioError::FileError(e) => write!(f, "File error: {}", e),
            AudioError::RecordingInProgress => write!(f, "Recording already in progress"),
            AudioError::NotRecording => write!(f, "No recording in progress"),
        }
    }
}

impl std::error::Error for AudioError {}

impl std::fmt::Debug for AudioRecorder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioRecorder")
            .field("sample_rate", &self.sample_rate)
            .field("channels", &self.channels)
            .field("is_recording", &self.is_recording)
            .finish()
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            channels: 1, // Mono for think-aloud protocols
            buffer_size: 4096,
            format: AudioFormat::F32,
        }
    }
}

impl AudioRecorder {
    pub fn new() -> Result<Self, AudioError> {
        let host = cpal::default_host();

        Ok(AudioRecorder {
            host,
            input_device: None,
            recording_thread: None,
            is_recording: Arc::new(Mutex::new(false)),
            sample_rate: 44100,
            channels: 1,
        })
    }

    pub fn initialize(&mut self, config: AudioConfig) -> Result<(), AudioError> {
        // Get the default input device
        let input_device = self
            .host
            .default_input_device()
            .ok_or(AudioError::DeviceNotFound)?;

        // Verify the device supports our desired configuration
        let supported_configs = input_device
            .supported_input_configs()
            .map_err(|e| AudioError::StreamError(e.to_string()))?;

        let mut compatible_config = None;
        for config_range in supported_configs {
            if config_range.channels() >= config.channels
                && config_range.min_sample_rate() <= SampleRate(config.sample_rate)
                && config_range.max_sample_rate() >= SampleRate(config.sample_rate)
            {
                compatible_config =
                    Some(config_range.with_sample_rate(SampleRate(config.sample_rate)));
                break;
            }
        }

        let _stream_config = compatible_config.ok_or(AudioError::ConfigNotSupported)?;

        self.input_device = Some(input_device);
        self.sample_rate = config.sample_rate;
        self.channels = config.channels;

        Ok(())
    }

    pub fn start_recording(&mut self, output_path: PathBuf) -> Result<(), AudioError> {
        let mut is_recording = self.is_recording.lock().unwrap();
        if *is_recording {
            return Err(AudioError::RecordingInProgress);
        }

        *is_recording = true;

        // For now, create a simple placeholder that simulates recording
        // TODO: Implement proper audio recording without threading issues
        println!("Starting audio recording to: {:?}", output_path);

        Ok(())
    }

    pub fn stop_recording(&mut self) -> Result<(), AudioError> {
        let mut is_recording = self.is_recording.lock().unwrap();
        if !*is_recording {
            return Err(AudioError::NotRecording);
        }

        *is_recording = false;

        println!("Stopping audio recording");

        Ok(())
    }

    pub fn is_recording(&self) -> bool {
        *self.is_recording.lock().unwrap()
    }

    pub fn get_available_devices(&self) -> Vec<String> {
        self.host
            .input_devices()
            .map(|devices| devices.filter_map(|device| device.name().ok()).collect())
            .unwrap_or_default()
    }

    pub fn get_supported_configs(&self) -> Result<Vec<(u32, u16)>, AudioError> {
        let device = self
            .input_device
            .as_ref()
            .ok_or(AudioError::DeviceNotFound)?;

        let configs = device
            .supported_input_configs()
            .map_err(|e| AudioError::StreamError(e.to_string()))?;

        let mut result = Vec::new();
        for config in configs {
            let min_rate = config.min_sample_rate().0;
            let max_rate = config.max_sample_rate().0;
            let channels = config.channels();

            // Add common sample rates within the supported range
            for &rate in &[8000, 16000, 22050, 44100, 48000, 96000] {
                if rate >= min_rate && rate <= max_rate {
                    result.push((rate, channels));
                }
            }
        }

        result.sort();
        result.dedup();
        Ok(result)
    }
}

impl Drop for AudioRecorder {
    fn drop(&mut self) {
        if self.is_recording() {
            let _ = self.stop_recording();
        }
    }
}

/// Helper function to create audio file paths
pub fn create_audio_file_path(participant_id: &str, session_id: &str) -> PathBuf {
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!(
        "recording_{}_{}__{}.wav",
        participant_id, session_id, timestamp
    );

    // Create audio directory if it doesn't exist
    let audio_dir = PathBuf::from("audio_recordings");
    std::fs::create_dir_all(&audio_dir).unwrap_or_else(|e| {
        eprintln!("Failed to create audio directory: {}", e);
    });

    audio_dir.join(filename)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_audio_recorder_creation() {
        let recorder = AudioRecorder::new();
        assert!(recorder.is_ok());
    }

    #[test]
    fn test_audio_config_default() {
        let config = AudioConfig::default();
        assert_eq!(config.sample_rate, 44100);
        assert_eq!(config.channels, 1);
    }

    #[test]
    fn test_create_audio_file_path() {
        let path = create_audio_file_path("test_participant", "test_session");
        assert!(path
            .to_string_lossy()
            .contains("recording_test_participant_test_session"));
        assert!(path.extension() == Some(std::ffi::OsStr::new("wav")));
    }
}
