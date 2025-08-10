use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// External Physiological Sensor Integration System
/// Provides integration for EEG, GSR, eye-tracking, and other biometric sensors

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorSession {
    pub session_id: String,
    pub participant_id: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub active_sensors: Vec<SensorConfig>,
    pub sensor_data: HashMap<String, Vec<SensorReading>>,
    pub sync_events: Vec<SyncEvent>,
    pub calibration_data: HashMap<String, CalibrationData>,
    pub data_quality: HashMap<String, DataQuality>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorConfig {
    pub sensor_id: String,
    pub sensor_type: SensorType,
    pub connection_type: ConnectionType,
    pub sampling_rate: f64, // Hz
    pub enabled: bool,
    pub calibrated: bool,
    pub device_info: DeviceInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SensorType {
    EEG {
        channels: Vec<EEGChannel>,
        reference_type: EEGReference,
        impedance_threshold: f64,
    },
    GSR {
        measurement_range: (f64, f64), // microsiemens
        electrode_placement: GSRPlacement,
    },
    EyeTracking {
        tracking_mode: EyeTrackingMode,
        accuracy: f64, // degrees visual angle
        sampling_frequency: f64,
    },
    HeartRate {
        measurement_type: HRMeasurementType,
    },
    BloodPressure {
        measurement_interval: Duration,
    },
    Accelerometer {
        range: f64, // g-force
        axes: usize,
    },
    Temperature {
        sensor_location: TemperatureLocation,
        accuracy: f64, // degrees Celsius
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EEGChannel {
    Fp1,
    Fp2,
    F3,
    F4,
    C3,
    C4,
    P3,
    P4,
    O1,
    O2,
    F7,
    F8,
    T3,
    T4,
    T5,
    T6,
    Fz,
    Cz,
    Pz,
    AF3,
    AF4,
    FC1,
    FC2,
    CP1,
    CP2,
    PO3,
    PO4,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EEGReference {
    CommonAverage,
    LinkedMastoids,
    Cz,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GSRPlacement {
    Fingers,
    Palm,
    Wrist,
    Foot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EyeTrackingMode {
    Binocular,
    Monocular,
    HeadMounted,
    Remote,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HRMeasurementType {
    ECG,
    PPG, // Photoplethysmography
    Chest,
    Wrist,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemperatureLocation {
    Skin,
    Core,
    Ambient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionType {
    Bluetooth { device_address: String },
    USB { device_path: String },
    TCP { ip: String, port: u16 },
    Serial { port: String, baud_rate: u32 },
    LSL { stream_name: String }, // Lab Streaming Layer
    WebSocket { url: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub manufacturer: String,
    pub model: String,
    pub firmware_version: Option<String>,
    pub serial_number: Option<String>,
    pub battery_level: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReading {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub sensor_id: String,
    pub data: SensorData,
    pub quality_indicator: Option<f64>,
    pub sync_marker: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SensorData {
    EEG {
        channels: HashMap<EEGChannel, f64>,   // microvolts
        impedances: HashMap<EEGChannel, f64>, // ohms
        artifacts_detected: Vec<ArtifactType>,
    },
    GSR {
        conductance: f64,         // microsiemens
        resistance: f64,          // ohms
        temperature: Option<f64>, // skin temperature
    },
    EyeTracking {
        gaze_point: GazePoint,
        pupil_diameter: Option<f64>, // millimeters
        fixation_duration: Option<Duration>,
        saccade_velocity: Option<f64>, // degrees per second
        blink_detected: bool,
    },
    HeartRate {
        bpm: f64,
        hrv: Option<f64>,               // heart rate variability (RMSSD)
        rr_intervals: Option<Vec<f64>>, // milliseconds
    },
    BloodPressure {
        systolic: f64,
        diastolic: f64,
        pulse: f64,
    },
    Accelerometer {
        x: f64,
        y: f64,
        z: f64,
        magnitude: f64,
    },
    Temperature {
        celsius: f64,
        humidity: Option<f64>, // relative humidity percentage
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GazePoint {
    pub x: f64, // screen coordinates (pixels or normalized)
    pub y: f64,
    pub z: Option<f64>,  // depth if available
    pub confidence: f64, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactType {
    Blink,
    EyeMovement,
    Muscle,
    Movement,
    ElectricalNoise,
    Saturation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: SyncEventType,
    pub task_marker: Option<String>,
    pub sensors_involved: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncEventType {
    TaskStart,
    TaskEnd,
    TrialStart,
    TrialEnd,
    StimulusPresentation,
    ResponseGiven,
    CalibrationMarker,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationData {
    pub sensor_id: String,
    pub calibration_type: CalibrationType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub parameters: HashMap<String, f64>,
    pub success: bool,
    pub error_metrics: Option<CalibrationError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CalibrationType {
    EEGImpedanceCheck,
    EyeTrackingGaze,
    GSRBaseline,
    HeartRateBaseline,
    AccelerometerZero,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationError {
    pub mean_error: f64,
    pub max_error: f64,
    pub accuracy: f64,
    pub precision: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQuality {
    pub sensor_id: String,
    pub overall_quality: QualityLevel,
    pub signal_to_noise_ratio: f64,
    pub data_loss_percentage: f64,
    pub artifact_percentage: f64,
    pub quality_issues: Vec<QualityIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityLevel {
    Excellent,
    Good,
    Fair,
    Poor,
    Unusable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssue {
    pub issue_type: QualityIssueType,
    pub severity: IssueSeverity,
    pub description: String,
    pub suggested_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityIssueType {
    HighImpedance,
    SignalDrift,
    ExcessiveNoise,
    DataLoss,
    CalibrationDrift,
    BatteryLow,
    ConnectionUnstable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// Main sensor integration system
pub struct SensorManager {
    session: Option<SensorSession>,
    sensors: HashMap<String, Box<dyn SensorInterface>>,
    data_buffer: Arc<Mutex<HashMap<String, VecDeque<SensorReading>>>>,
    recording_active: Arc<Mutex<bool>>,
    sync_clock: Arc<Mutex<chrono::DateTime<chrono::Utc>>>,
}

pub trait SensorInterface: Send + Sync {
    fn connect(&mut self) -> Result<(), String>;
    fn disconnect(&mut self) -> Result<(), String>;
    fn is_connected(&self) -> bool;
    fn start_recording(&mut self) -> Result<(), String>;
    fn stop_recording(&mut self) -> Result<(), String>;
    fn calibrate(&mut self) -> Result<CalibrationData, String>;
    fn get_latest_reading(&self) -> Option<SensorReading>;
    fn get_device_info(&self) -> DeviceInfo;
    fn check_quality(&self) -> DataQuality;
}

impl SensorManager {
    pub fn new() -> Self {
        Self {
            session: None,
            sensors: HashMap::new(),
            data_buffer: Arc::new(Mutex::new(HashMap::new())),
            recording_active: Arc::new(Mutex::new(false)),
            sync_clock: Arc::new(Mutex::new(chrono::Utc::now())),
        }
    }

    /// Add a sensor to the system
    pub fn add_sensor(
        &mut self,
        config: SensorConfig,
        sensor: Box<dyn SensorInterface>,
    ) -> Result<(), String> {
        self.sensors.insert(config.sensor_id.clone(), sensor);

        // Initialize data buffer for this sensor
        {
            let mut buffer = self.data_buffer.lock().unwrap();
            buffer.insert(config.sensor_id.clone(), VecDeque::new());
        }

        Ok(())
    }

    /// Start a new sensor session
    pub fn start_session(
        &mut self,
        session_id: String,
        participant_id: String,
    ) -> Result<(), String> {
        if self.session.is_some() {
            return Err("Session already active".to_string());
        }

        // Connect all sensors
        let mut active_sensors = Vec::new();
        for (sensor_id, sensor) in &mut self.sensors {
            if let Err(e) = sensor.connect() {
                eprintln!("Failed to connect sensor {}: {}", sensor_id, e);
                continue;
            }

            // Get sensor configuration
            let device_info = sensor.get_device_info();
            active_sensors.push(SensorConfig {
                sensor_id: sensor_id.clone(),
                sensor_type: SensorType::EEG {
                    channels: vec![EEGChannel::Cz], // Default
                    reference_type: EEGReference::CommonAverage,
                    impedance_threshold: 5000.0,
                },
                connection_type: ConnectionType::Bluetooth {
                    device_address: "Unknown".to_string(),
                },
                sampling_rate: 250.0,
                enabled: true,
                calibrated: false,
                device_info,
            });
        }

        self.session = Some(SensorSession {
            session_id,
            participant_id,
            start_time: chrono::Utc::now(),
            end_time: None,
            active_sensors,
            sensor_data: HashMap::new(),
            sync_events: Vec::new(),
            calibration_data: HashMap::new(),
            data_quality: HashMap::new(),
        });

        // Update sync clock
        {
            let mut sync_clock = self.sync_clock.lock().unwrap();
            *sync_clock = chrono::Utc::now();
        }

        Ok(())
    }

    /// Calibrate all sensors
    pub fn calibrate_sensors(&mut self) -> Result<(), String> {
        if self.session.is_none() {
            return Err("No active session".to_string());
        }

        for (sensor_id, sensor) in &mut self.sensors {
            match sensor.calibrate() {
                Ok(calibration_data) => {
                    if let Some(ref mut session) = self.session {
                        session
                            .calibration_data
                            .insert(sensor_id.clone(), calibration_data);
                    }
                }
                Err(e) => {
                    eprintln!("Calibration failed for sensor {}: {}", sensor_id, e);
                }
            }
        }

        Ok(())
    }

    /// Start recording from all sensors
    pub fn start_recording(&mut self) -> Result<(), String> {
        if self.session.is_none() {
            return Err("No active session".to_string());
        }

        let mut recording_active = self.recording_active.lock().unwrap();
        if *recording_active {
            return Err("Already recording".to_string());
        }

        // Start all sensors
        for (sensor_id, sensor) in &mut self.sensors {
            if let Err(e) = sensor.start_recording() {
                eprintln!("Failed to start recording for sensor {}: {}", sensor_id, e);
            }
        }

        *recording_active = true;
        drop(recording_active);

        // Start data collection threads for each sensor
        self.start_data_collection_threads();

        // Add sync event
        self.add_sync_event(SyncEventType::TaskStart, None);

        Ok(())
    }

    /// Stop recording from all sensors
    pub fn stop_recording(&mut self) -> Result<(), String> {
        {
            let mut recording_active = self.recording_active.lock().unwrap();
            if !*recording_active {
                return Err("Not currently recording".to_string());
            }

            // Stop all sensors
            for (sensor_id, sensor) in &mut self.sensors {
                if let Err(e) = sensor.stop_recording() {
                    eprintln!("Failed to stop recording for sensor {}: {}", sensor_id, e);
                }
            }

            *recording_active = false;
        } // Release the lock before calling add_sync_event

        // Add sync event
        self.add_sync_event(SyncEventType::TaskEnd, None);

        Ok(())
    }

    /// Add synchronization event
    pub fn add_sync_event(&mut self, event_type: SyncEventType, task_marker: Option<String>) {
        if let Some(ref mut session) = self.session {
            let sync_event = SyncEvent {
                timestamp: chrono::Utc::now(),
                event_type,
                task_marker,
                sensors_involved: self.sensors.keys().cloned().collect(),
            };
            session.sync_events.push(sync_event);
        }
    }

    /// Get real-time sensor data summary
    pub fn get_real_time_summary(&self) -> HashMap<String, SensorSummary> {
        let mut summaries = HashMap::new();

        for (sensor_id, sensor) in &self.sensors {
            let latest_reading = sensor.get_latest_reading();
            let quality = sensor.check_quality();
            let is_connected = sensor.is_connected();

            summaries.insert(
                sensor_id.clone(),
                SensorSummary {
                    sensor_id: sensor_id.clone(),
                    is_connected,
                    latest_reading,
                    quality,
                    buffer_size: {
                        let buffer = self.data_buffer.lock().unwrap();
                        buffer.get(sensor_id).map(|b| b.len()).unwrap_or(0)
                    },
                },
            );
        }

        summaries
    }

    /// Finalize session and return all sensor data
    pub fn finalize_session(mut self) -> Result<SensorSession, String> {
        if let Some(mut session) = self.session.take() {
            session.end_time = Some(chrono::Utc::now());

            // Stop recording if still active
            if *self.recording_active.lock().unwrap() {
                self.stop_recording()?;
            }

            // Collect all buffered data
            let buffer = self.data_buffer.lock().unwrap();
            for (sensor_id, readings) in buffer.iter() {
                session
                    .sensor_data
                    .insert(sensor_id.clone(), readings.iter().cloned().collect());
            }
            drop(buffer);

            // Final quality assessment
            for (sensor_id, sensor) in &self.sensors {
                let quality = sensor.check_quality();
                session.data_quality.insert(sensor_id.clone(), quality);
            }

            // Disconnect all sensors
            for (sensor_id, sensor) in &mut self.sensors {
                if let Err(e) = sensor.disconnect() {
                    eprintln!("Failed to disconnect sensor {}: {}", sensor_id, e);
                }
            }

            Ok(session)
        } else {
            Err("No active session to finalize".to_string())
        }
    }

    fn start_data_collection_threads(&mut self) {
        let data_buffer = Arc::clone(&self.data_buffer);
        let recording_active = Arc::clone(&self.recording_active);

        // In a real implementation, each sensor would have its own thread
        // Here we simulate with a single thread that polls all sensors
        thread::spawn(move || {
            while *recording_active.lock().unwrap() {
                // Simulate data collection
                let mock_reading = SensorReading {
                    timestamp: chrono::Utc::now(),
                    sensor_id: "mock_sensor".to_string(),
                    data: SensorData::EEG {
                        channels: {
                            let mut channels = HashMap::new();
                            channels.insert(EEGChannel::Cz, 10.0); // 10 microvolts
                            channels
                        },
                        impedances: HashMap::new(),
                        artifacts_detected: Vec::new(),
                    },
                    quality_indicator: Some(0.9),
                    sync_marker: None,
                };

                // Add to buffer
                {
                    let mut buffer = data_buffer.lock().unwrap();
                    if let Some(sensor_buffer) = buffer.get_mut("mock_sensor") {
                        sensor_buffer.push_back(mock_reading);

                        // Keep buffer size manageable
                        while sensor_buffer.len() > 10000 {
                            sensor_buffer.pop_front();
                        }
                    }
                }

                thread::sleep(Duration::from_millis(4)); // 250 Hz sampling
            }
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorSummary {
    pub sensor_id: String,
    pub is_connected: bool,
    pub latest_reading: Option<SensorReading>,
    pub quality: DataQuality,
    pub buffer_size: usize,
}

/// Mock EEG sensor implementation for testing/demonstration
pub struct MockEEGSensor {
    connected: bool,
    recording: bool,
    device_info: DeviceInfo,
    last_reading: Option<SensorReading>,
}

impl MockEEGSensor {
    pub fn new() -> Self {
        Self {
            connected: false,
            recording: false,
            device_info: DeviceInfo {
                manufacturer: "OpenBCI".to_string(),
                model: "Cyton".to_string(),
                firmware_version: Some("3.1.2".to_string()),
                serial_number: Some("MOCK001".to_string()),
                battery_level: Some(0.85),
            },
            last_reading: None,
        }
    }
}

impl SensorInterface for MockEEGSensor {
    fn connect(&mut self) -> Result<(), String> {
        // Simulate connection process
        thread::sleep(Duration::from_millis(100));
        self.connected = true;
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), String> {
        self.connected = false;
        self.recording = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn start_recording(&mut self) -> Result<(), String> {
        if !self.connected {
            return Err("Sensor not connected".to_string());
        }
        self.recording = true;
        Ok(())
    }

    fn stop_recording(&mut self) -> Result<(), String> {
        self.recording = false;
        Ok(())
    }

    fn calibrate(&mut self) -> Result<CalibrationData, String> {
        if !self.connected {
            return Err("Sensor not connected".to_string());
        }

        // Simulate calibration
        thread::sleep(Duration::from_millis(500));

        Ok(CalibrationData {
            sensor_id: "mock_eeg".to_string(),
            calibration_type: CalibrationType::EEGImpedanceCheck,
            timestamp: chrono::Utc::now(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("impedance_threshold".to_string(), 5000.0);
                params
            },
            success: true,
            error_metrics: Some(CalibrationError {
                mean_error: 0.1,
                max_error: 0.5,
                accuracy: 0.95,
                precision: 0.92,
            }),
        })
    }

    fn get_latest_reading(&self) -> Option<SensorReading> {
        if self.recording {
            // Generate mock EEG data
            let mut channels = HashMap::new();
            channels.insert(EEGChannel::Fp1, rand::random::<f64>() * 20.0 - 10.0);
            channels.insert(EEGChannel::Fp2, rand::random::<f64>() * 20.0 - 10.0);
            channels.insert(EEGChannel::Cz, rand::random::<f64>() * 20.0 - 10.0);

            Some(SensorReading {
                timestamp: chrono::Utc::now(),
                sensor_id: "mock_eeg".to_string(),
                data: SensorData::EEG {
                    channels,
                    impedances: HashMap::new(),
                    artifacts_detected: Vec::new(),
                },
                quality_indicator: Some(0.9),
                sync_marker: None,
            })
        } else {
            None
        }
    }

    fn get_device_info(&self) -> DeviceInfo {
        self.device_info.clone()
    }

    fn check_quality(&self) -> DataQuality {
        DataQuality {
            sensor_id: "mock_eeg".to_string(),
            overall_quality: QualityLevel::Good,
            signal_to_noise_ratio: 15.0,
            data_loss_percentage: 0.5,
            artifact_percentage: 2.0,
            quality_issues: Vec::new(),
        }
    }
}

/// Mock GSR sensor implementation
pub struct MockGSRSensor {
    connected: bool,
    recording: bool,
    device_info: DeviceInfo,
}

impl MockGSRSensor {
    pub fn new() -> Self {
        Self {
            connected: false,
            recording: false,
            device_info: DeviceInfo {
                manufacturer: "Shimmer".to_string(),
                model: "GSR3+".to_string(),
                firmware_version: Some("0.15.0".to_string()),
                serial_number: Some("GSR001".to_string()),
                battery_level: Some(0.92),
            },
        }
    }
}

impl SensorInterface for MockGSRSensor {
    fn connect(&mut self) -> Result<(), String> {
        thread::sleep(Duration::from_millis(200));
        self.connected = true;
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), String> {
        self.connected = false;
        self.recording = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn start_recording(&mut self) -> Result<(), String> {
        if !self.connected {
            return Err("Sensor not connected".to_string());
        }
        self.recording = true;
        Ok(())
    }

    fn stop_recording(&mut self) -> Result<(), String> {
        self.recording = false;
        Ok(())
    }

    fn calibrate(&mut self) -> Result<CalibrationData, String> {
        if !self.connected {
            return Err("Sensor not connected".to_string());
        }

        Ok(CalibrationData {
            sensor_id: "mock_gsr".to_string(),
            calibration_type: CalibrationType::GSRBaseline,
            timestamp: chrono::Utc::now(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("baseline_conductance".to_string(), 2.5);
                params
            },
            success: true,
            error_metrics: None,
        })
    }

    fn get_latest_reading(&self) -> Option<SensorReading> {
        if self.recording {
            // Generate mock GSR data (skin conductance varies with arousal/stress)
            let base_conductance = 2.5; // microsiemens
            let variation = rand::random::<f64>() * 1.0 - 0.5; // ±0.5 μS
            let conductance = base_conductance + variation;
            let resistance = 1.0 / conductance * 1_000_000.0; // convert to ohms

            Some(SensorReading {
                timestamp: chrono::Utc::now(),
                sensor_id: "mock_gsr".to_string(),
                data: SensorData::GSR {
                    conductance,
                    resistance,
                    temperature: Some(32.0 + rand::random::<f64>() * 3.0), // 32-35°C
                },
                quality_indicator: Some(0.95),
                sync_marker: None,
            })
        } else {
            None
        }
    }

    fn get_device_info(&self) -> DeviceInfo {
        self.device_info.clone()
    }

    fn check_quality(&self) -> DataQuality {
        DataQuality {
            sensor_id: "mock_gsr".to_string(),
            overall_quality: QualityLevel::Excellent,
            signal_to_noise_ratio: 25.0,
            data_loss_percentage: 0.1,
            artifact_percentage: 0.5,
            quality_issues: Vec::new(),
        }
    }
}

/// Mock eye tracker implementation using computer vision
pub struct MockEyeTracker {
    connected: bool,
    recording: bool,
    device_info: DeviceInfo,
    calibrated: bool,
}

impl MockEyeTracker {
    pub fn new() -> Self {
        Self {
            connected: false,
            recording: false,
            device_info: DeviceInfo {
                manufacturer: "Tobii".to_string(),
                model: "Eye Tracker 5".to_string(),
                firmware_version: Some("1.7.1-441deb0".to_string()),
                serial_number: Some("ET001".to_string()),
                battery_level: None, // USB powered
            },
            calibrated: false,
        }
    }
}

impl SensorInterface for MockEyeTracker {
    fn connect(&mut self) -> Result<(), String> {
        // Simulate USB/camera detection
        thread::sleep(Duration::from_millis(300));
        self.connected = true;
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), String> {
        self.connected = false;
        self.recording = false;
        self.calibrated = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn start_recording(&mut self) -> Result<(), String> {
        if !self.connected {
            return Err("Eye tracker not connected".to_string());
        }
        if !self.calibrated {
            return Err("Eye tracker not calibrated".to_string());
        }
        self.recording = true;
        Ok(())
    }

    fn stop_recording(&mut self) -> Result<(), String> {
        self.recording = false;
        Ok(())
    }

    fn calibrate(&mut self) -> Result<CalibrationData, String> {
        if !self.connected {
            return Err("Eye tracker not connected".to_string());
        }

        // Simulate calibration process
        thread::sleep(Duration::from_millis(2000)); // Calibration takes time
        self.calibrated = true;

        Ok(CalibrationData {
            sensor_id: "mock_eyetracker".to_string(),
            calibration_type: CalibrationType::EyeTrackingGaze,
            timestamp: chrono::Utc::now(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("accuracy_degrees".to_string(), 0.4);
                params.insert("precision_degrees".to_string(), 0.08);
                params
            },
            success: true,
            error_metrics: Some(CalibrationError {
                mean_error: 0.3,
                max_error: 0.8,
                accuracy: 0.96,
                precision: 0.92,
            }),
        })
    }

    fn get_latest_reading(&self) -> Option<SensorReading> {
        if self.recording && self.calibrated {
            // Generate mock eye tracking data
            let screen_width = 1920.0;
            let screen_height = 1080.0;

            // Simulate gaze wandering around screen center
            let center_x = screen_width / 2.0;
            let center_y = screen_height / 2.0;
            let noise_x = (rand::random::<f64>() - 0.5) * 200.0; // ±100 pixels
            let noise_y = (rand::random::<f64>() - 0.5) * 200.0;

            Some(SensorReading {
                timestamp: chrono::Utc::now(),
                sensor_id: "mock_eyetracker".to_string(),
                data: SensorData::EyeTracking {
                    gaze_point: GazePoint {
                        x: center_x + noise_x,
                        y: center_y + noise_y,
                        z: Some(650.0), // mm from screen
                        confidence: 0.9 + rand::random::<f64>() * 0.1,
                    },
                    pupil_diameter: Some(3.0 + rand::random::<f64>() * 2.0), // 3-5mm
                    fixation_duration: Some(Duration::from_millis(
                        150 + (rand::random::<u64>() % 300),
                    )),
                    saccade_velocity: None, // Only set during saccades
                    blink_detected: rand::random::<f64>() < 0.02, // 2% chance of blink
                },
                quality_indicator: Some(0.95),
                sync_marker: None,
            })
        } else {
            None
        }
    }

    fn get_device_info(&self) -> DeviceInfo {
        self.device_info.clone()
    }

    fn check_quality(&self) -> DataQuality {
        let mut quality_issues = Vec::new();

        if !self.calibrated {
            quality_issues.push(QualityIssue {
                issue_type: QualityIssueType::CalibrationDrift,
                severity: IssueSeverity::High,
                description: "Eye tracker not calibrated".to_string(),
                suggested_action: "Run calibration procedure".to_string(),
            });
        }

        DataQuality {
            sensor_id: "mock_eyetracker".to_string(),
            overall_quality: if self.calibrated {
                QualityLevel::Excellent
            } else {
                QualityLevel::Poor
            },
            signal_to_noise_ratio: if self.calibrated { 30.0 } else { 5.0 },
            data_loss_percentage: if self.calibrated { 0.2 } else { 15.0 },
            artifact_percentage: 1.0,
            quality_issues,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensor_manager_creation() {
        let manager = SensorManager::new();
        assert!(manager.session.is_none());
        assert!(manager.sensors.is_empty());
    }

    #[test]
    fn test_mock_eeg_sensor() {
        let mut sensor = MockEEGSensor::new();

        // Test connection
        assert!(!sensor.is_connected());
        sensor.connect().unwrap();
        assert!(sensor.is_connected());

        // Test calibration
        let calibration = sensor.calibrate().unwrap();
        assert!(calibration.success);

        // Test recording
        sensor.start_recording().unwrap();
        let reading = sensor.get_latest_reading();
        assert!(reading.is_some());

        let quality = sensor.check_quality();
        assert!(matches!(quality.overall_quality, QualityLevel::Good));
    }

    #[test]
    fn test_mock_gsr_sensor() {
        let mut sensor = MockGSRSensor::new();

        sensor.connect().unwrap();
        sensor.start_recording().unwrap();

        let reading = sensor.get_latest_reading().unwrap();
        if let SensorData::GSR {
            conductance,
            resistance,
            temperature,
        } = reading.data
        {
            assert!(conductance > 0.0);
            assert!(resistance > 0.0);
            assert!(temperature.is_some());
        } else {
            panic!("Expected GSR sensor data");
        }
    }

    #[test]
    fn test_mock_eye_tracker() {
        let mut tracker = MockEyeTracker::new();

        tracker.connect().unwrap();

        // Should fail before calibration
        assert!(tracker.start_recording().is_err());

        // Calibrate and try again
        tracker.calibrate().unwrap();
        tracker.start_recording().unwrap();

        let reading = tracker.get_latest_reading().unwrap();
        if let SensorData::EyeTracking { gaze_point, .. } = reading.data {
            assert!(gaze_point.confidence > 0.8);
            assert!(gaze_point.x >= 0.0);
            assert!(gaze_point.y >= 0.0);
        } else {
            panic!("Expected eye tracking data");
        }
    }

    #[test]
    fn test_sensor_session_workflow() {
        let mut manager = SensorManager::new();

        // Add mock sensors
        let eeg_config = SensorConfig {
            sensor_id: "eeg".to_string(),
            sensor_type: SensorType::EEG {
                channels: vec![EEGChannel::Cz],
                reference_type: EEGReference::CommonAverage,
                impedance_threshold: 5000.0,
            },
            connection_type: ConnectionType::Bluetooth {
                device_address: "00:11:22:33:44:55".to_string(),
            },
            sampling_rate: 250.0,
            enabled: true,
            calibrated: false,
            device_info: DeviceInfo {
                manufacturer: "Test".to_string(),
                model: "Mock".to_string(),
                firmware_version: None,
                serial_number: None,
                battery_level: None,
            },
        };

        manager
            .add_sensor(eeg_config, Box::new(MockEEGSensor::new()))
            .unwrap();

        // Start session
        manager
            .start_session("test_session".to_string(), "participant_1".to_string())
            .unwrap();
        assert!(manager.session.is_some());

        // Check that we can get real-time summary
        let summary = manager.get_real_time_summary();
        assert!(!summary.is_empty());
    }
}
