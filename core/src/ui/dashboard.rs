//! Research Dashboard with Pre-Registration Interface
//!
//! This module provides a comprehensive research dashboard for managing
//! pre-registrations, viewing existing registrations, and ensuring
//! scientific integrity in experimental studies.

use chrono::{DateTime, Local, Utc};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};
use std::collections::HashMap;
use std::io::{self, Write};
use std::time::Duration;

use crate::interaction::audio::{AudioRecorder, AudioSession, ThinkAloudAnalyzer};
use crate::research::compliance::IRBComplianceGenerator;
use crate::statistics::mixed_effects::MixedEffectsAnalyzer;
use crate::statistics::power_analysis::{PowerAnalyzer, StatisticalTestType};
use crate::research::preregistration::{
    AnalysisValidator, PowerAnalysisSpec, PreRegistration, RegistrationStatus, StudyMetadata,
};
use crate::interaction::sensors::{
    EEGChannel, EEGReference, EyeTrackingMode, GSRPlacement, SensorManager, SensorSession,
    SensorType,
};

/// Dashboard view states
#[derive(Debug, Clone, PartialEq)]
pub enum DashboardView {
    Overview,
    PreRegistrationList,
    CreatePreRegistration,
    EditPreRegistration { id: String },
    ViewPreRegistration { id: String },
    PowerAnalysisCalculator,
    AnalysisValidation,
    TransparencyReport { id: String },
    // Research data collection views
    ExperimentManagement,
    AudioRecording { session_id: Option<String> },
    SensorIntegration,
    DataCollection { experiment_id: String },
    IRBCompliance,
    StatisticalAnalysis,
}

/// Sub-views for the pre-registration creation form
#[derive(Debug, Clone, PartialEq)]
pub enum FormSection {
    StudyMetadata,
    Hypotheses,
    AnalysisPlan,
    PowerAnalysis,
    DataCollection,
    ExclusionCriteria,
    DecisionRules,
    Review,
}

/// Research Dashboard state and UI management
pub struct ResearchDashboard {
    current_view: DashboardView,
    current_form_section: FormSection,
    preregistrations: Vec<PreRegistration>,
    draft_registration: PreRegistration,
    form_inputs: FormInputs,
    selected_index: usize,
    scroll_offset: usize,
    message: Option<(String, MessageType)>,
    show_finalization_dialog: bool,
    analysis_validator: Option<AnalysisValidator>,
    power_calculator: PowerCalculatorState,
    // Research data collection components
    audio_recorder: Option<AudioRecorder>,
    current_audio_session: Option<AudioSession>,
    sensor_manager: Option<SensorManager>,
    current_sensor_session: Option<SensorSession>,
    irb_generator: Option<IRBComplianceGenerator>,
    mixed_effects_analyzer: Option<MixedEffectsAnalyzer>,
    think_aloud_analyzer: Option<ThinkAloudAnalyzer>,
    // UI state for research components
    recording_state: AudioRecordingState,
    sensor_configs: Vec<SensorConfiguration>,
}

/// Form input state management
#[derive(Debug, Clone)]
struct FormInputs {
    // Study metadata
    title: String,
    description: String,
    researchers: Vec<String>,
    current_researcher: String,
    institution: String,
    ethical_approval: String,
    funding_source: String,
    conflicts: Vec<String>,
    current_conflict: String,

    // Hypotheses
    hypothesis_id: String,
    hypothesis_description: String,
    hypothesis_operationalization: String,
    hypothesis_test: String,
    hypothesis_alpha: String,
    hypothesis_effect_type: String,
    hypothesis_effect_value: String,
    is_primary_hypothesis: bool,

    // Analysis plan
    analysis_name: String,
    analysis_description: String,
    dependent_variable: String,
    independent_variables: Vec<String>,
    current_independent: String,
    covariates: Vec<String>,
    current_covariate: String,
    statistical_model: String,
    assumptions: Vec<String>,
    current_assumption: String,
    fallback_method: String,
    correction_method: String,

    // Power analysis
    target_power: String,
    alpha_level: String,
    effect_size: String,

    // Data collection
    sample_size: String,
    sampling_method: String,
    inclusion_criteria: Vec<String>,
    current_inclusion: String,
    randomization: String,
    blinding_level: String,
    stopping_rule_type: String,
    stopping_n: String,
    quality_checks: Vec<String>,
    current_quality_check: String,

    // Exclusion criteria
    participant_exclusions: Vec<String>,
    current_participant_exclusion: String,
    trial_exclusions: Vec<String>,
    current_trial_exclusion: String,
    data_quality_exclusions: Vec<String>,
    current_data_quality: String,
    outlier_strategy: String,
    outlier_threshold: String,

    // Decision rules
    success_criteria: Vec<String>,
    current_success: String,
    failure_criteria: Vec<String>,
    current_failure: String,
    interpretation_guidelines: HashMap<String, String>,
    current_guideline_key: String,
    current_guideline_value: String,
}

/// Power calculator state
#[derive(Debug, Clone)]
struct PowerCalculatorState {
    test_type: String,
    effect_size: String,
    alpha: String,
    power: String,
    sample_size: String,
    calculated_result: Option<PowerCalculationResult>,
}

#[derive(Debug, Clone)]
struct PowerCalculationResult {
    required_n: usize,
    actual_power: f64,
    effect_size: f64,
    alpha: f64,
}

#[derive(Debug, Clone, PartialEq)]
enum MessageType {
    Success,
    Error,
    Warning,
    Info,
}

/// Audio recording UI state
#[derive(Debug, Clone, PartialEq)]
pub enum AudioRecordingState {
    Idle,
    Recording {
        start_time: DateTime<Utc>,
        duration: Duration,
    },
    Paused {
        total_duration: Duration,
    },
    Processing,
    Completed {
        file_path: String,
    },
    Error(String),
}

/// Sensor configuration for UI display
#[derive(Debug, Clone)]
pub struct SensorConfiguration {
    pub sensor_type: SensorType,
    pub enabled: bool,
    pub sample_rate: u32,
    pub buffer_size: usize,
    pub status: SensorStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SensorStatus {
    Disconnected,
    Connected,
    Recording,
    Error(String),
}

impl FormInputs {
    fn new() -> Self {
        FormInputs {
            title: String::new(),
            description: String::new(),
            researchers: Vec::new(),
            current_researcher: String::new(),
            institution: String::new(),
            ethical_approval: String::new(),
            funding_source: String::new(),
            conflicts: Vec::new(),
            current_conflict: String::new(),
            hypothesis_id: String::new(),
            hypothesis_description: String::new(),
            hypothesis_operationalization: String::new(),
            hypothesis_test: String::new(),
            hypothesis_alpha: "0.05".to_string(),
            hypothesis_effect_type: "greater_than".to_string(),
            hypothesis_effect_value: String::new(),
            is_primary_hypothesis: true,
            analysis_name: String::new(),
            analysis_description: String::new(),
            dependent_variable: String::new(),
            independent_variables: Vec::new(),
            current_independent: String::new(),
            covariates: Vec::new(),
            current_covariate: String::new(),
            statistical_model: String::new(),
            assumptions: Vec::new(),
            current_assumption: String::new(),
            fallback_method: String::new(),
            correction_method: "Benjamini-Hochberg".to_string(),
            target_power: "0.80".to_string(),
            alpha_level: "0.05".to_string(),
            effect_size: "0.5".to_string(),
            sample_size: String::new(),
            sampling_method: String::new(),
            inclusion_criteria: Vec::new(),
            current_inclusion: String::new(),
            randomization: String::new(),
            blinding_level: "none".to_string(),
            stopping_rule_type: "fixed".to_string(),
            stopping_n: String::new(),
            quality_checks: Vec::new(),
            current_quality_check: String::new(),
            participant_exclusions: Vec::new(),
            current_participant_exclusion: String::new(),
            trial_exclusions: Vec::new(),
            current_trial_exclusion: String::new(),
            data_quality_exclusions: Vec::new(),
            current_data_quality: String::new(),
            outlier_strategy: "none".to_string(),
            outlier_threshold: String::new(),
            success_criteria: Vec::new(),
            current_success: String::new(),
            failure_criteria: Vec::new(),
            current_failure: String::new(),
            interpretation_guidelines: HashMap::new(),
            current_guideline_key: String::new(),
            current_guideline_value: String::new(),
        }
    }
}

impl ResearchDashboard {
    pub fn new() -> Self {
        ResearchDashboard {
            current_view: DashboardView::Overview,
            current_form_section: FormSection::StudyMetadata,
            preregistrations: Vec::new(),
            draft_registration: PreRegistration::new(String::new(), String::new(), Vec::new()),
            form_inputs: FormInputs::new(),
            selected_index: 0,
            scroll_offset: 0,
            message: None,
            show_finalization_dialog: false,
            analysis_validator: None,
            power_calculator: PowerCalculatorState {
                test_type: "t-test".to_string(),
                effect_size: "0.5".to_string(),
                alpha: "0.05".to_string(),
                power: "0.80".to_string(),
                sample_size: String::new(),
                calculated_result: None,
            },
            // Initialize research components
            audio_recorder: None,
            current_audio_session: None,
            sensor_manager: None,
            current_sensor_session: None,
            irb_generator: None,
            mixed_effects_analyzer: None,
            think_aloud_analyzer: None,
            recording_state: AudioRecordingState::Idle,
            sensor_configs: vec![
                SensorConfiguration {
                    sensor_type: SensorType::EEG {
                        channels: vec![EEGChannel::Fz, EEGChannel::Cz, EEGChannel::Pz],
                        reference_type: EEGReference::CommonAverage,
                        impedance_threshold: 5.0,
                    },
                    enabled: false,
                    sample_rate: 250,
                    buffer_size: 1024,
                    status: SensorStatus::Disconnected,
                },
                SensorConfiguration {
                    sensor_type: SensorType::GSR {
                        measurement_range: (0.0, 100.0),
                        electrode_placement: GSRPlacement::Fingers,
                    },
                    enabled: false,
                    sample_rate: 100,
                    buffer_size: 512,
                    status: SensorStatus::Disconnected,
                },
                SensorConfiguration {
                    sensor_type: SensorType::EyeTracking {
                        tracking_mode: EyeTrackingMode::Binocular,
                        accuracy: 0.5,
                        sampling_frequency: 60.0,
                    },
                    enabled: false,
                    sample_rate: 60,
                    buffer_size: 256,
                    status: SensorStatus::Disconnected,
                },
            ],
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, terminal::EnterAlternateScreen)?;

        loop {
            execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;

            // Render header
            self.render_header(&mut stdout)?;

            // Render current view
            let current_view = self.current_view.clone();
            match current_view {
                DashboardView::Overview => self.render_overview(&mut stdout)?,
                DashboardView::PreRegistrationList => self.render_registration_list(&mut stdout)?,
                DashboardView::CreatePreRegistration => self.render_create_form(&mut stdout)?,
                DashboardView::EditPreRegistration { id } => {
                    self.render_edit_form(&mut stdout, &id)?
                }
                DashboardView::ViewPreRegistration { id } => {
                    self.render_registration_viewer(&mut stdout, &id)?
                }
                DashboardView::PowerAnalysisCalculator => {
                    self.render_power_calculator(&mut stdout)?
                }
                DashboardView::AnalysisValidation => {
                    self.render_analysis_validation(&mut stdout)?
                }
                DashboardView::TransparencyReport { id } => {
                    self.render_transparency_report(&mut stdout, &id)?
                }
                // Research data collection views
                DashboardView::ExperimentManagement => {
                    self.render_experiment_management(&mut stdout)?
                }
                DashboardView::AudioRecording { session_id } => {
                    self.render_audio_recording(&mut stdout, session_id.as_deref())?
                }
                DashboardView::SensorIntegration => self.render_sensor_integration(&mut stdout)?,
                DashboardView::DataCollection { experiment_id } => {
                    self.render_data_collection(&mut stdout, &experiment_id)?
                }
                DashboardView::IRBCompliance => self.render_irb_compliance(&mut stdout)?,
                DashboardView::StatisticalAnalysis => {
                    self.render_statistical_analysis(&mut stdout)?
                }
            }

            // Render message if any
            if let Some((msg, msg_type)) = &self.message {
                self.render_message(&mut stdout, msg, msg_type)?;
            }

            // Render finalization dialog if shown
            if self.show_finalization_dialog {
                self.render_finalization_dialog(&mut stdout)?;
            }

            stdout.flush()?;

            // Handle input
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if !self.handle_input(key)? {
                        break;
                    }
                }
            }
        }

        execute!(stdout, terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    fn render_header(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        execute!(
            stdout,
            SetForegroundColor(Color::Cyan),
            Print("╔══════════════════════════════════════════════════════════════════════════╗\n"),
            Print("║                         RESEARCH DASHBOARD                              ║\n"),
            Print("║                    Pre-Registration & Analysis System                   ║\n"),
            Print("╚══════════════════════════════════════════════════════════════════════════╝\n"),
            ResetColor
        )?;

        // Render navigation tabs
        let tabs = vec![
            ("Overview", DashboardView::Overview),
            ("Pre-Registrations", DashboardView::PreRegistrationList),
            ("New Registration", DashboardView::CreatePreRegistration),
            ("Power Analysis", DashboardView::PowerAnalysisCalculator),
            ("Validation", DashboardView::AnalysisValidation),
        ];

        execute!(stdout, Print("\n"))?;
        for (i, (label, view)) in tabs.iter().enumerate() {
            if std::mem::discriminant(&self.current_view) == std::mem::discriminant(view) {
                execute!(
                    stdout,
                    SetBackgroundColor(Color::DarkBlue),
                    SetForegroundColor(Color::White),
                    Print(format!(" [{}] {} ", i + 1, label)),
                    ResetColor,
                    Print("  ")
                )?;
            } else {
                execute!(
                    stdout,
                    SetForegroundColor(Color::Grey),
                    Print(format!(" [{}] {} ", i + 1, label)),
                    ResetColor,
                    Print("  ")
                )?;
            }
        }
        execute!(stdout, Print("\n\n"))?;

        Ok(())
    }

    fn render_overview(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        execute!(
            stdout,
            SetForegroundColor(Color::Yellow),
            Print("Dashboard Overview\n"),
            Print("─────────────────────────────────────────────────────────────────────\n\n"),
            ResetColor
        )?;

        // Statistics
        let total = self.preregistrations.len();
        let drafts = self
            .preregistrations
            .iter()
            .filter(|p| matches!(p.status, RegistrationStatus::Draft))
            .count();
        let registered = self
            .preregistrations
            .iter()
            .filter(|p| matches!(p.status, RegistrationStatus::Registered))
            .count();
        let in_progress = self
            .preregistrations
            .iter()
            .filter(|p| matches!(p.status, RegistrationStatus::DataCollectionStarted))
            .count();
        let completed = self
            .preregistrations
            .iter()
            .filter(|p| matches!(p.status, RegistrationStatus::AnalysisComplete))
            .count();

        execute!(
            stdout,
            SetForegroundColor(Color::Green),
            Print("📊 Statistics:\n"),
            ResetColor,
            Print(format!("  Total Pre-Registrations: {}\n", total)),
            Print(format!("  • Drafts: {} ", drafts)),
            SetForegroundColor(Color::Yellow),
            Print("●\n"),
            ResetColor,
            Print(format!("  • Registered: {} ", registered)),
            SetForegroundColor(Color::Green),
            Print("●\n"),
            ResetColor,
            Print(format!("  • Data Collection: {} ", in_progress)),
            SetForegroundColor(Color::Cyan),
            Print("●\n"),
            ResetColor,
            Print(format!("  • Analysis Complete: {} ", completed)),
            SetForegroundColor(Color::Blue),
            Print("●\n\n"),
            ResetColor
        )?;

        // Recent activity
        execute!(
            stdout,
            SetForegroundColor(Color::Green),
            Print("📝 Recent Activity:\n"),
            ResetColor
        )?;

        let mut recent: Vec<_> = self.preregistrations.iter().take(5).collect();
        recent.sort_by_key(|p| p.registered_at);
        recent.reverse();

        if recent.is_empty() {
            execute!(stdout, Print("  No recent pre-registrations\n"))?;
        } else {
            for prereg in recent.iter().take(5) {
                let status_color = match prereg.status {
                    RegistrationStatus::Draft => Color::Yellow,
                    RegistrationStatus::Registered => Color::Green,
                    RegistrationStatus::DataCollectionStarted => Color::Cyan,
                    RegistrationStatus::AnalysisComplete => Color::Blue,
                    _ => Color::Grey,
                };

                execute!(
                    stdout,
                    Print(format!(
                        "  • {} - ",
                        prereg
                            .registered_at
                            .with_timezone(&Local)
                            .format("%Y-%m-%d")
                    )),
                    SetForegroundColor(status_color),
                    Print(format!("{:?}", prereg.status)),
                    ResetColor,
                    Print(format!(" - {}\n", prereg.study.title))
                )?;
            }
        }

        execute!(
            stdout,
            Print("\n"),
            SetForegroundColor(Color::DarkGrey),
            Print("Press [1-5] to navigate tabs | [Q] to quit\n"),
            ResetColor
        )?;

        Ok(())
    }

    fn render_registration_list(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        execute!(
            stdout,
            SetForegroundColor(Color::Yellow),
            Print("Pre-Registration List\n"),
            Print("─────────────────────────────────────────────────────────────────────\n\n"),
            ResetColor
        )?;

        if self.preregistrations.is_empty() {
            execute!(
                stdout,
                SetForegroundColor(Color::Grey),
                Print("No pre-registrations found.\n"),
                Print("Press [3] to create a new pre-registration.\n"),
                ResetColor
            )?;
        } else {
            // Table header
            execute!(
                stdout,
                SetForegroundColor(Color::Cyan),
                Print(format!(
                    "{:<30} {:<15} {:<20} {:<10}\n",
                    "Title", "Status", "Date", "Actions"
                )),
                Print("─".repeat(75)),
                Print("\n"),
                ResetColor
            )?;

            // List items with pagination
            let items_per_page = 10;
            let start = self.scroll_offset;
            let end = (start + items_per_page).min(self.preregistrations.len());

            for (i, prereg) in self.preregistrations[start..end].iter().enumerate() {
                let is_selected = i + start == self.selected_index;

                let status_color = match prereg.status {
                    RegistrationStatus::Draft => Color::Yellow,
                    RegistrationStatus::Registered => Color::Green,
                    RegistrationStatus::DataCollectionStarted => Color::Cyan,
                    RegistrationStatus::DataCollectionComplete => Color::Blue,
                    RegistrationStatus::AnalysisComplete => Color::Magenta,
                    RegistrationStatus::Published => Color::White,
                };

                if is_selected {
                    execute!(stdout, SetBackgroundColor(Color::DarkGrey))?;
                }

                let title = if prereg.study.title.len() > 28 {
                    format!("{}...", &prereg.study.title[..25])
                } else {
                    prereg.study.title.clone()
                };

                execute!(
                    stdout,
                    Print(format!("{:<30} ", title)),
                    SetForegroundColor(status_color),
                    Print(format!("{:<15} ", format!("{:?}", prereg.status))),
                    ResetColor,
                    Print(format!(
                        "{:<20} ",
                        prereg
                            .registered_at
                            .with_timezone(&Local)
                            .format("%Y-%m-%d %H:%M")
                    ))
                )?;

                // Actions based on status
                match prereg.status {
                    RegistrationStatus::Draft => {
                        execute!(stdout, Print("[E]dit [F]inalize"))?;
                    }
                    _ => {
                        execute!(stdout, Print("[V]iew [R]eport"))?;
                    }
                }

                if is_selected {
                    execute!(stdout, ResetColor)?;
                }
                execute!(stdout, Print("\n"))?;
            }

            // Pagination info
            execute!(
                stdout,
                Print("\n"),
                SetForegroundColor(Color::DarkGrey),
                Print(format!(
                    "Showing {}-{} of {} | ",
                    start + 1,
                    end,
                    self.preregistrations.len()
                )),
                Print("↑/↓ Navigate | Enter: Select | N: New | Q: Quit\n"),
                ResetColor
            )?;
        }

        Ok(())
    }

    fn render_create_form(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        execute!(
            stdout,
            SetForegroundColor(Color::Yellow),
            Print("Create Pre-Registration\n"),
            Print("─────────────────────────────────────────────────────────────────────\n\n"),
            ResetColor
        )?;

        // Form section tabs
        let sections = vec![
            ("Study", FormSection::StudyMetadata),
            ("Hypotheses", FormSection::Hypotheses),
            ("Analysis", FormSection::AnalysisPlan),
            ("Power", FormSection::PowerAnalysis),
            ("Collection", FormSection::DataCollection),
            ("Exclusions", FormSection::ExclusionCriteria),
            ("Decisions", FormSection::DecisionRules),
            ("Review", FormSection::Review),
        ];

        for (label, section) in &sections {
            if self.current_form_section == *section {
                execute!(
                    stdout,
                    SetBackgroundColor(Color::DarkBlue),
                    SetForegroundColor(Color::White),
                    Print(format!(" {} ", label)),
                    ResetColor,
                    Print(" ")
                )?;
            } else {
                execute!(
                    stdout,
                    SetForegroundColor(Color::Grey),
                    Print(format!(" {} ", label)),
                    ResetColor,
                    Print(" ")
                )?;
            }
        }
        execute!(stdout, Print("\n\n"))?;

        // Render current section
        match self.current_form_section {
            FormSection::StudyMetadata => self.render_study_metadata_form(stdout)?,
            FormSection::Hypotheses => self.render_hypotheses_form(stdout)?,
            FormSection::AnalysisPlan => self.render_analysis_plan_form(stdout)?,
            FormSection::PowerAnalysis => self.render_power_analysis_form(stdout)?,
            FormSection::DataCollection => self.render_data_collection_form(stdout)?,
            FormSection::ExclusionCriteria => self.render_exclusion_criteria_form(stdout)?,
            FormSection::DecisionRules => self.render_decision_rules_form(stdout)?,
            FormSection::Review => self.render_review_form(stdout)?,
        }

        // Navigation help
        execute!(
            stdout,
            Print("\n"),
            SetForegroundColor(Color::DarkGrey),
            Print("Tab: Next section | Shift+Tab: Previous | Ctrl+S: Save draft | F: Finalize\n"),
            ResetColor
        )?;

        Ok(())
    }

    fn render_study_metadata_form(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        execute!(
            stdout,
            SetForegroundColor(Color::Green),
            Print("Study Metadata\n"),
            Print("─────────────────────────────────────────────────────────────────────\n\n"),
            ResetColor
        )?;

        execute!(
            stdout,
            Print("Title: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.title),
            ResetColor,
            Print("\n\n"),
            Print("Description:\n"),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.description),
            ResetColor,
            Print("\n\n"),
            Print("Researchers: "),
            SetForegroundColor(Color::Cyan)
        )?;

        for researcher in &self.form_inputs.researchers {
            execute!(stdout, Print(format!("[{}] ", researcher)))?;
        }
        execute!(
            stdout,
            ResetColor,
            Print("\nAdd researcher: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.current_researcher),
            ResetColor,
            Print("\n\n"),
            Print("Institution: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.institution),
            ResetColor,
            Print("\n\n"),
            Print("Ethical Approval #: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.ethical_approval),
            ResetColor,
            Print("\n\n"),
            Print("Funding Source: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.funding_source),
            ResetColor,
            Print("\n\n"),
            Print("Conflicts of Interest: "),
            SetForegroundColor(Color::Cyan)
        )?;

        for conflict in &self.form_inputs.conflicts {
            execute!(stdout, Print(format!("[{}] ", conflict)))?;
        }
        execute!(
            stdout,
            ResetColor,
            Print("\nAdd conflict: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.current_conflict),
            ResetColor
        )?;

        Ok(())
    }

    fn render_hypotheses_form(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        execute!(
            stdout,
            SetForegroundColor(Color::Green),
            Print("Hypotheses\n"),
            Print("─────────────────────────────────────────────────────────────────────\n\n"),
            ResetColor
        )?;

        // Show existing hypotheses
        execute!(
            stdout,
            SetForegroundColor(Color::Cyan),
            Print("Primary Hypotheses:\n"),
            ResetColor
        )?;
        for hyp in &self.draft_registration.hypotheses.primary {
            execute!(
                stdout,
                Print(format!("  • [{}] {}\n", hyp.id, hyp.description))
            )?;
        }

        execute!(
            stdout,
            Print("\n"),
            SetForegroundColor(Color::Cyan),
            Print("Secondary Hypotheses:\n"),
            ResetColor
        )?;
        for hyp in &self.draft_registration.hypotheses.secondary {
            execute!(
                stdout,
                Print(format!("  • [{}] {}\n", hyp.id, hyp.description))
            )?;
        }

        // New hypothesis form
        execute!(
            stdout,
            Print("\n"),
            SetForegroundColor(Color::Yellow),
            Print("Add New Hypothesis:\n"),
            Print("─────────────────────────\n"),
            ResetColor,
            Print("Type: "),
            SetForegroundColor(if self.form_inputs.is_primary_hypothesis {
                Color::Green
            } else {
                Color::Yellow
            }),
            Print(if self.form_inputs.is_primary_hypothesis {
                "[Primary]"
            } else {
                "[Secondary]"
            }),
            ResetColor,
            Print(" (Press P to toggle)\n\n"),
            Print("ID: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.hypothesis_id),
            ResetColor,
            Print("\n\n"),
            Print("Description:\n"),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.hypothesis_description),
            ResetColor,
            Print("\n\n"),
            Print("Operationalization:\n"),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.hypothesis_operationalization),
            ResetColor,
            Print("\n\n"),
            Print("Statistical Test: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.hypothesis_test),
            ResetColor,
            Print("\n\n"),
            Print("Alpha Level: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.hypothesis_alpha),
            ResetColor,
            Print("\n\n"),
            Print("Effect Prediction: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.hypothesis_effect_type),
            ResetColor,
            Print(" Value: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.hypothesis_effect_value),
            ResetColor,
            Print("\n\n"),
            SetForegroundColor(Color::Green),
            Print("[A] Add Hypothesis | [Tab] Next Section\n"),
            ResetColor
        )?;

        Ok(())
    }

    fn render_analysis_plan_form(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        execute!(
            stdout,
            SetForegroundColor(Color::Green),
            Print("Analysis Plan\n"),
            Print("─────────────────────────────────────────────────────────────────────\n\n"),
            ResetColor
        )?;

        // Show existing analyses
        execute!(
            stdout,
            SetForegroundColor(Color::Cyan),
            Print("Primary Analyses:\n"),
            ResetColor
        )?;
        for analysis in &self.draft_registration.analysis_plan.primary_analyses {
            execute!(
                stdout,
                Print(format!(
                    "  • {} - {}\n",
                    analysis.name, analysis.statistical_model
                ))
            )?;
        }

        // New analysis form
        execute!(
            stdout,
            Print("\n"),
            SetForegroundColor(Color::Yellow),
            Print("Add New Analysis:\n"),
            Print("─────────────────────────\n"),
            ResetColor,
            Print("Name: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.analysis_name),
            ResetColor,
            Print("\n\n"),
            Print("Description:\n"),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.analysis_description),
            ResetColor,
            Print("\n\n"),
            Print("Dependent Variable: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.dependent_variable),
            ResetColor,
            Print("\n\n"),
            Print("Independent Variables: "),
            SetForegroundColor(Color::Cyan)
        )?;

        for var in &self.form_inputs.independent_variables {
            execute!(stdout, Print(format!("[{}] ", var)))?;
        }
        execute!(
            stdout,
            ResetColor,
            Print("\nAdd variable: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.current_independent),
            ResetColor,
            Print("\n\n"),
            Print("Statistical Model: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.statistical_model),
            ResetColor,
            Print("\n\n"),
            Print("Multiple Comparison Correction: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.correction_method),
            ResetColor,
            Print("\n\n"),
            SetForegroundColor(Color::Green),
            Print("[A] Add Analysis | [Tab] Next Section\n"),
            ResetColor
        )?;

        Ok(())
    }

    fn render_power_analysis_form(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        execute!(
            stdout,
            SetForegroundColor(Color::Green),
            Print("Power Analysis\n"),
            Print("─────────────────────────────────────────────────────────────────────\n\n"),
            ResetColor,
            Print("Target Power: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.target_power),
            ResetColor,
            Print("\n\n"),
            Print("Alpha Level: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.alpha_level),
            ResetColor,
            Print("\n\n"),
            Print("Expected Effect Size (Cohen's d): "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.effect_size),
            ResetColor,
            Print("\n\n")
        )?;

        // Calculate required sample size
        if let (Ok(power), Ok(alpha), Ok(effect)) = (
            self.form_inputs.target_power.parse::<f64>(),
            self.form_inputs.alpha_level.parse::<f64>(),
            self.form_inputs.effect_size.parse::<f64>(),
        ) {
            let analyzer = PowerAnalyzer::new(0.05, 0.8);
            match analyzer.calculate_sample_size(
                StatisticalTestType::IndependentTTest,
                effect,
                power,
            ) {
                Ok(required_n) => {
                    execute!(
                        stdout,
                        SetForegroundColor(Color::Cyan),
                        Print("───────────────────────────────────────\n"),
                        Print(format!("Required Sample Size: {} per group\n", required_n)),
                        Print(format!("Total N (two groups): {}\n", required_n * 2)),
                        ResetColor
                    )?;
                }
                Err(e) => {
                    execute!(
                        stdout,
                        SetForegroundColor(Color::Red),
                        Print(format!("Error calculating sample size: {}\n", e)),
                        ResetColor
                    )?;
                }
            }
        }

        execute!(
            stdout,
            Print("\n"),
            SetForegroundColor(Color::DarkGrey),
            Print("Effect Size Guidelines:\n"),
            Print("  Small: 0.2 | Medium: 0.5 | Large: 0.8\n"),
            ResetColor
        )?;

        Ok(())
    }

    fn render_data_collection_form(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        execute!(
            stdout,
            SetForegroundColor(Color::Green),
            Print("Data Collection Plan\n"),
            Print("─────────────────────────────────────────────────────────────────────\n\n"),
            ResetColor,
            Print("Target Sample Size: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.sample_size),
            ResetColor,
            Print("\n\n"),
            Print("Sampling Method: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.sampling_method),
            ResetColor,
            Print("\n\n"),
            Print("Inclusion Criteria: "),
            SetForegroundColor(Color::Cyan)
        )?;

        for criterion in &self.form_inputs.inclusion_criteria {
            execute!(stdout, Print(format!("\n  • {} ", criterion)))?;
        }
        execute!(
            stdout,
            ResetColor,
            Print("\nAdd criterion: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.current_inclusion),
            ResetColor,
            Print("\n\n"),
            Print("Randomization Procedure: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.randomization),
            ResetColor,
            Print("\n\n"),
            Print("Blinding Level: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.blinding_level),
            ResetColor,
            Print(" [none/single/double/triple]\n\n"),
            Print("Stopping Rule: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.stopping_rule_type),
            ResetColor,
            Print(" N="),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.stopping_n),
            ResetColor,
            Print("\n")
        )?;

        Ok(())
    }

    fn render_exclusion_criteria_form(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        execute!(
            stdout,
            SetForegroundColor(Color::Green),
            Print("Exclusion Criteria\n"),
            Print("─────────────────────────────────────────────────────────────────────\n\n"),
            ResetColor,
            SetForegroundColor(Color::Yellow),
            Print("Participant-Level Exclusions:\n"),
            ResetColor
        )?;

        for criterion in &self.form_inputs.participant_exclusions {
            execute!(stdout, Print(format!("  • {}\n", criterion)))?;
        }
        execute!(
            stdout,
            Print("Add: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.current_participant_exclusion),
            ResetColor,
            Print("\n\n"),
            SetForegroundColor(Color::Yellow),
            Print("Trial-Level Exclusions:\n"),
            ResetColor
        )?;

        for criterion in &self.form_inputs.trial_exclusions {
            execute!(stdout, Print(format!("  • {}\n", criterion)))?;
        }
        execute!(
            stdout,
            Print("Add: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.current_trial_exclusion),
            ResetColor,
            Print("\n\n"),
            Print("Outlier Handling Strategy: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.outlier_strategy),
            ResetColor,
            Print(" Threshold: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.outlier_threshold),
            ResetColor,
            Print("\n")
        )?;

        Ok(())
    }

    fn render_decision_rules_form(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        execute!(
            stdout,
            SetForegroundColor(Color::Green),
            Print("Decision Rules\n"),
            Print("─────────────────────────────────────────────────────────────────────\n\n"),
            ResetColor,
            SetForegroundColor(Color::Green),
            Print("Success Criteria:\n"),
            ResetColor
        )?;

        for criterion in &self.form_inputs.success_criteria {
            execute!(stdout, Print(format!("  ✓ {}\n", criterion)))?;
        }
        execute!(
            stdout,
            Print("Add: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.current_success),
            ResetColor,
            Print("\n\n"),
            SetForegroundColor(Color::Red),
            Print("Failure Criteria:\n"),
            ResetColor
        )?;

        for criterion in &self.form_inputs.failure_criteria {
            execute!(stdout, Print(format!("  ✗ {}\n", criterion)))?;
        }
        execute!(
            stdout,
            Print("Add: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.current_failure),
            ResetColor,
            Print("\n\n"),
            SetForegroundColor(Color::Yellow),
            Print("Interpretation Guidelines:\n"),
            ResetColor
        )?;

        for (key, value) in &self.form_inputs.interpretation_guidelines {
            execute!(stdout, Print(format!("  {} → {}\n", key, value)))?;
        }
        execute!(
            stdout,
            Print("Key: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.current_guideline_key),
            ResetColor,
            Print(" Value: "),
            SetForegroundColor(Color::White),
            Print(&self.form_inputs.current_guideline_value),
            ResetColor,
            Print("\n")
        )?;

        Ok(())
    }

    fn render_review_form(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        execute!(
            stdout,
            SetForegroundColor(Color::Green),
            Print("Review Pre-Registration\n"),
            Print("─────────────────────────────────────────────────────────────────────\n\n"),
            ResetColor
        )?;

        // Summary of all sections
        execute!(
            stdout,
            SetForegroundColor(Color::Cyan),
            Print("Study Summary:\n"),
            ResetColor,
            Print(format!("  Title: {}\n", self.form_inputs.title)),
            Print(format!(
                "  Researchers: {}\n",
                self.form_inputs.researchers.join(", ")
            )),
            Print(format!(
                "  Institution: {}\n\n",
                self.form_inputs.institution
            )),
            SetForegroundColor(Color::Cyan),
            Print("Hypotheses:\n"),
            ResetColor,
            Print(format!(
                "  Primary: {}\n",
                self.draft_registration.hypotheses.primary.len()
            )),
            Print(format!(
                "  Secondary: {}\n\n",
                self.draft_registration.hypotheses.secondary.len()
            )),
            SetForegroundColor(Color::Cyan),
            Print("Analysis Plan:\n"),
            ResetColor,
            Print(format!(
                "  Primary Analyses: {}\n",
                self.draft_registration.analysis_plan.primary_analyses.len()
            )),
            Print(format!("  Power: {}\n", self.form_inputs.target_power)),
            Print(format!("  Alpha: {}\n", self.form_inputs.alpha_level)),
            Print(format!(
                "  Effect Size: {}\n\n",
                self.form_inputs.effect_size
            )),
            SetForegroundColor(Color::Cyan),
            Print("Data Collection:\n"),
            ResetColor,
            Print(format!("  Sample Size: {}\n", self.form_inputs.sample_size)),
            Print(format!(
                "  Blinding: {}\n\n",
                self.form_inputs.blinding_level
            ))
        )?;

        // Validation checks
        execute!(
            stdout,
            SetForegroundColor(Color::Yellow),
            Print("Validation Checks:\n"),
            ResetColor
        )?;

        let checks = vec![
            (!self.form_inputs.title.is_empty(), "Title provided"),
            (
                !self.form_inputs.researchers.is_empty(),
                "Researchers listed",
            ),
            (
                !self.draft_registration.hypotheses.primary.is_empty(),
                "Primary hypotheses defined",
            ),
            (
                !self
                    .draft_registration
                    .analysis_plan
                    .primary_analyses
                    .is_empty(),
                "Primary analyses specified",
            ),
            (
                !self.form_inputs.sample_size.is_empty(),
                "Sample size determined",
            ),
        ];

        for (passed, description) in checks {
            if passed {
                execute!(
                    stdout,
                    SetForegroundColor(Color::Green),
                    Print("  ✓ "),
                    ResetColor,
                    Print(format!("{}\n", description))
                )?;
            } else {
                execute!(
                    stdout,
                    SetForegroundColor(Color::Red),
                    Print("  ✗ "),
                    ResetColor,
                    Print(format!("{}\n", description))
                )?;
            }
        }

        execute!(
            stdout,
            Print("\n"),
            SetForegroundColor(Color::Green),
            Print("[S] Save as Draft | [F] Finalize Registration | [E] Edit\n"),
            ResetColor
        )?;

        Ok(())
    }

    fn render_edit_form(&mut self, stdout: &mut io::Stdout, id: &str) -> io::Result<()> {
        // Similar to create form but loads existing data
        self.render_create_form(stdout)
    }

    fn render_registration_viewer(&self, stdout: &mut io::Stdout, id: &str) -> io::Result<()> {
        if let Some(prereg) = self.preregistrations.iter().find(|p| p.id == id) {
            execute!(
                stdout,
                SetForegroundColor(Color::Yellow),
                Print("Pre-Registration Details\n"),
                Print("─────────────────────────────────────────────────────────────────────\n\n"),
                ResetColor,
                SetForegroundColor(Color::Cyan),
                Print("Registration Information:\n"),
                ResetColor,
                Print(format!("  ID: {}\n", prereg.id)),
                Print(format!("  Status: {:?}\n", prereg.status)),
                Print(format!(
                    "  Registered: {}\n",
                    prereg
                        .registered_at
                        .with_timezone(&Local)
                        .format("%Y-%m-%d %H:%M:%S")
                ))
            )?;

            if !prereg.registration_hash.is_empty() {
                execute!(
                    stdout,
                    SetForegroundColor(Color::Green),
                    Print(format!("  Hash: {}\n", &prereg.registration_hash[..16])),
                    ResetColor
                )?;
            }

            execute!(
                stdout,
                Print("\n"),
                SetForegroundColor(Color::Cyan),
                Print("Study Details:\n"),
                ResetColor,
                Print(format!("  Title: {}\n", prereg.study.title)),
                Print(format!("  Description: {}\n", prereg.study.description)),
                Print(format!(
                    "  Researchers: {}\n",
                    prereg.study.researchers.join(", ")
                )),
                Print(format!("  Institution: {}\n", prereg.study.institution))
            )?;

            if let Some(approval) = &prereg.study.ethical_approval {
                execute!(stdout, Print(format!("  Ethical Approval: {}\n", approval)))?;
            }

            execute!(
                stdout,
                Print("\n"),
                SetForegroundColor(Color::Cyan),
                Print("Hypotheses:\n"),
                ResetColor,
                SetForegroundColor(Color::Green),
                Print(format!(
                    "  Primary ({}):\n",
                    prereg.hypotheses.primary.len()
                )),
                ResetColor
            )?;

            for hyp in &prereg.hypotheses.primary {
                execute!(
                    stdout,
                    Print(format!("    • [{}] {}\n", hyp.id, hyp.description)),
                    Print(format!(
                        "      Test: {} (α = {})\n",
                        hyp.statistical_test, hyp.alpha_level
                    ))
                )?;
            }

            if !prereg.hypotheses.secondary.is_empty() {
                execute!(
                    stdout,
                    SetForegroundColor(Color::Yellow),
                    Print(format!(
                        "  Secondary ({}):\n",
                        prereg.hypotheses.secondary.len()
                    )),
                    ResetColor
                )?;

                for hyp in &prereg.hypotheses.secondary {
                    execute!(
                        stdout,
                        Print(format!("    • [{}] {}\n", hyp.id, hyp.description))
                    )?;
                }
            }

            // Deviations if any
            if !prereg.deviations.is_empty() {
                execute!(
                    stdout,
                    Print("\n"),
                    SetForegroundColor(Color::Red),
                    Print("Deviations from Pre-Registration:\n"),
                    ResetColor
                )?;

                for deviation in &prereg.deviations {
                    execute!(
                        stdout,
                        Print(format!(
                            "  • {} - {}\n",
                            deviation.timestamp.with_timezone(&Local).format("%Y-%m-%d"),
                            deviation.description
                        )),
                        Print(format!("    Justification: {}\n", deviation.justification))
                    )?;
                }
            }

            execute!(
                stdout,
                Print("\n"),
                SetForegroundColor(Color::DarkGrey),
                Print("[T] View Transparency Report | [B] Back to List | [Q] Quit\n"),
                ResetColor
            )?;
        } else {
            execute!(
                stdout,
                SetForegroundColor(Color::Red),
                Print("Pre-registration not found.\n"),
                ResetColor
            )?;
        }

        Ok(())
    }

    fn render_power_calculator(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        execute!(
            stdout,
            SetForegroundColor(Color::Yellow),
            Print("Power Analysis Calculator\n"),
            Print("─────────────────────────────────────────────────────────────────────\n\n"),
            ResetColor,
            Print("Test Type: "),
            SetForegroundColor(Color::White),
            Print(&self.power_calculator.test_type),
            ResetColor,
            Print(" [t-test/anova/regression]\n\n"),
            Print("Effect Size (Cohen's d): "),
            SetForegroundColor(Color::White),
            Print(&self.power_calculator.effect_size),
            ResetColor,
            Print("\n"),
            SetForegroundColor(Color::DarkGrey),
            Print("  Small: 0.2 | Medium: 0.5 | Large: 0.8\n\n"),
            ResetColor,
            Print("Alpha Level: "),
            SetForegroundColor(Color::White),
            Print(&self.power_calculator.alpha),
            ResetColor,
            Print("\n\n"),
            Print("Desired Power: "),
            SetForegroundColor(Color::White),
            Print(&self.power_calculator.power),
            ResetColor,
            Print("\n\n"),
            Print("Sample Size (optional): "),
            SetForegroundColor(Color::White),
            Print(&self.power_calculator.sample_size),
            ResetColor,
            Print("\n\n")
        )?;

        // Show calculation result
        if let Some(result) = &self.power_calculator.calculated_result {
            execute!(
                stdout,
                SetForegroundColor(Color::Green),
                Print("═══════════════════════════════════════════════\n"),
                Print("Calculation Results:\n"),
                Print("───────────────────────────────────────────────\n"),
                ResetColor,
                Print(format!("  Required N per group: {}\n", result.required_n)),
                Print(format!(
                    "  Total N (two groups): {}\n",
                    result.required_n * 2
                )),
                Print(format!("  Actual Power: {:.3}\n", result.actual_power)),
                Print(format!("  Effect Size: {:.3}\n", result.effect_size)),
                Print(format!("  Alpha: {:.3}\n", result.alpha)),
                SetForegroundColor(Color::Green),
                Print("═══════════════════════════════════════════════\n"),
                ResetColor
            )?;
        }

        execute!(
            stdout,
            Print("\n"),
            SetForegroundColor(Color::DarkGrey),
            Print("[C] Calculate | [R] Reset | [B] Back | [Q] Quit\n"),
            ResetColor
        )?;

        Ok(())
    }

    fn render_analysis_validation(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        execute!(
            stdout,
            SetForegroundColor(Color::Yellow),
            Print("Analysis Validation\n"),
            Print("─────────────────────────────────────────────────────────────────────\n\n"),
            ResetColor
        )?;

        if let Some(validator) = &self.analysis_validator {
            execute!(
                stdout,
                SetForegroundColor(Color::Green),
                Print("Pre-registered analyses are being validated.\n\n"),
                ResetColor,
                SetForegroundColor(Color::Cyan),
                Print("Validation Status:\n"),
                ResetColor,
                Print("  ✓ All primary analyses must match pre-registration\n"),
                Print("  ⚠ Secondary analyses should be marked as exploratory\n"),
                Print("  ✗ Non-registered analyses require justification\n\n"),
                SetForegroundColor(Color::Yellow),
                Print("Recent Validations:\n"),
                ResetColor,
                Print("  [No recent validations to display]\n")
            )?;
        } else {
            execute!(
                stdout,
                SetForegroundColor(Color::Grey),
                Print("No pre-registration selected for validation.\n"),
                Print("Select a finalized pre-registration to enable validation.\n"),
                ResetColor
            )?;
        }

        execute!(
            stdout,
            Print("\n"),
            SetForegroundColor(Color::DarkGrey),
            Print("[S] Select Pre-Registration | [B] Back | [Q] Quit\n"),
            ResetColor
        )?;

        Ok(())
    }

    fn render_transparency_report(&self, stdout: &mut io::Stdout, id: &str) -> io::Result<()> {
        if let Some(prereg) = self.preregistrations.iter().find(|p| p.id == id) {
            let report = prereg.generate_transparency_report();

            execute!(
                stdout,
                SetForegroundColor(Color::Yellow),
                Print("Transparency Report\n"),
                Print("─────────────────────────────────────────────────────────────────────\n\n"),
                ResetColor,
                SetForegroundColor(Color::Cyan),
                Print("Registration Summary:\n"),
                ResetColor,
                Print(format!("  ID: {}\n", report.registration_id)),
                Print(format!(
                    "  Date: {}\n",
                    report
                        .registered_at
                        .with_timezone(&Local)
                        .format("%Y-%m-%d %H:%M:%S")
                )),
                Print(format!(
                    "  Hash: {}...\n\n",
                    &report.registration_hash[..32]
                )),
                SetForegroundColor(Color::Cyan),
                Print("Pre-Registered Elements:\n"),
                ResetColor,
                Print(format!(
                    "  Primary Hypotheses: {}\n",
                    report.n_primary_hypotheses
                )),
                Print(format!(
                    "  Secondary Hypotheses: {}\n",
                    report.n_secondary_hypotheses
                )),
                Print(format!(
                    "  Primary Analyses: {}\n",
                    report.n_primary_analyses
                )),
                Print(format!(
                    "  Secondary Analyses: {}\n\n",
                    report.n_secondary_analyses
                )),
                SetForegroundColor(Color::Cyan),
                Print("Adherence:\n"),
                ResetColor
            )?;

            if report.n_deviations == 0 {
                execute!(
                    stdout,
                    SetForegroundColor(Color::Green),
                    Print("  ✓ No deviations from pre-registration\n"),
                    ResetColor
                )?;
            } else {
                execute!(
                    stdout,
                    SetForegroundColor(Color::Yellow),
                    Print(format!(
                        "  ⚠ {} deviations recorded:\n",
                        report.n_deviations
                    )),
                    ResetColor
                )?;

                for deviation in &report.deviation_descriptions {
                    execute!(stdout, Print(format!("    • {}\n", deviation)))?;
                }
            }

            execute!(
                stdout,
                Print("\n"),
                SetForegroundColor(Color::Green),
                Print("This report can be included in publications to demonstrate\n"),
                Print("transparency and adherence to pre-registered protocols.\n\n"),
                ResetColor,
                SetForegroundColor(Color::DarkGrey),
                Print("[E] Export Report | [B] Back | [Q] Quit\n"),
                ResetColor
            )?;
        }

        Ok(())
    }

    fn render_finalization_dialog(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        execute!(
            stdout,
            cursor::MoveTo(10, 10),
            SetBackgroundColor(Color::DarkRed),
            SetForegroundColor(Color::White),
            Print("╔══════════════════════════════════════════════════════╗\n"),
            cursor::MoveTo(10, 11),
            Print("║            FINALIZE PRE-REGISTRATION?               ║\n"),
            cursor::MoveTo(10, 12),
            Print("╠══════════════════════════════════════════════════════╣\n"),
            cursor::MoveTo(10, 13),
            Print("║                                                      ║\n"),
            cursor::MoveTo(10, 14),
            Print("║  ⚠ WARNING: This action cannot be undone!          ║\n"),
            cursor::MoveTo(10, 15),
            Print("║                                                      ║\n"),
            cursor::MoveTo(10, 16),
            Print("║  Finalizing will:                                   ║\n"),
            cursor::MoveTo(10, 17),
            Print("║  • Lock all hypotheses and analyses                 ║\n"),
            cursor::MoveTo(10, 18),
            Print("║  • Generate a cryptographic hash                    ║\n"),
            cursor::MoveTo(10, 19),
            Print("║  • Timestamp the registration                       ║\n"),
            cursor::MoveTo(10, 20),
            Print("║  • Prevent further edits                            ║\n"),
            cursor::MoveTo(10, 21),
            Print("║                                                      ║\n"),
            cursor::MoveTo(10, 22),
            Print("║  [Y] Yes, Finalize  |  [N] No, Keep as Draft       ║\n"),
            cursor::MoveTo(10, 23),
            Print("╚══════════════════════════════════════════════════════╝\n"),
            ResetColor
        )?;

        Ok(())
    }

    fn render_message(
        &self,
        stdout: &mut io::Stdout,
        msg: &str,
        msg_type: &MessageType,
    ) -> io::Result<()> {
        let color = match msg_type {
            MessageType::Success => Color::Green,
            MessageType::Error => Color::Red,
            MessageType::Warning => Color::Yellow,
            MessageType::Info => Color::Cyan,
        };

        let symbol = match msg_type {
            MessageType::Success => "✓",
            MessageType::Error => "✗",
            MessageType::Warning => "⚠",
            MessageType::Info => "ℹ",
        };

        execute!(
            stdout,
            cursor::MoveTo(0, 25),
            SetForegroundColor(color),
            Print(format!(" {} {} ", symbol, msg)),
            ResetColor
        )?;

        Ok(())
    }

    fn handle_input(&mut self, key: KeyEvent) -> io::Result<bool> {
        // Clear message on any input
        self.message = None;

        // Handle finalization dialog
        if self.show_finalization_dialog {
            return self.handle_finalization_input(key);
        }

        // Global navigation
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q')
                if key.modifiers.contains(KeyModifiers::NONE) =>
            {
                return Ok(false);
            }
            KeyCode::Char('1') => {
                self.current_view = DashboardView::Overview;
            }
            KeyCode::Char('2') => {
                self.current_view = DashboardView::PreRegistrationList;
            }
            KeyCode::Char('3') => {
                self.current_view = DashboardView::CreatePreRegistration;
            }
            KeyCode::Char('4') => {
                self.current_view = DashboardView::PowerAnalysisCalculator;
            }
            KeyCode::Char('5') => {
                self.current_view = DashboardView::AnalysisValidation;
            }
            _ => {
                // View-specific input handling
                match &self.current_view {
                    DashboardView::PreRegistrationList => self.handle_list_input(key)?,
                    DashboardView::CreatePreRegistration => self.handle_form_input(key)?,
                    DashboardView::PowerAnalysisCalculator => self.handle_calculator_input(key)?,
                    _ => {}
                }
            }
        }

        Ok(true)
    }

    fn handle_finalization_input(&mut self, key: KeyEvent) -> io::Result<bool> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.finalize_registration();
                self.show_finalization_dialog = false;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.show_finalization_dialog = false;
                self.message = Some(("Registration kept as draft".to_string(), MessageType::Info));
            }
            _ => {}
        }
        Ok(true)
    }

    fn handle_list_input(&mut self, key: KeyEvent) -> io::Result<()> {
        match key.code {
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    if self.selected_index < self.scroll_offset {
                        self.scroll_offset = self.selected_index;
                    }
                }
            }
            KeyCode::Down => {
                if self.selected_index < self.preregistrations.len().saturating_sub(1) {
                    self.selected_index += 1;
                    if self.selected_index >= self.scroll_offset + 10 {
                        self.scroll_offset = self.selected_index - 9;
                    }
                }
            }
            KeyCode::Enter | KeyCode::Char('v') | KeyCode::Char('V') => {
                if let Some(prereg) = self.preregistrations.get(self.selected_index) {
                    self.current_view = DashboardView::ViewPreRegistration {
                        id: prereg.id.clone(),
                    };
                }
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                if let Some(prereg) = self.preregistrations.get(self.selected_index) {
                    if matches!(prereg.status, RegistrationStatus::Draft) {
                        self.current_view = DashboardView::EditPreRegistration {
                            id: prereg.id.clone(),
                        };
                    }
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                self.current_view = DashboardView::CreatePreRegistration;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_form_input(&mut self, key: KeyEvent) -> io::Result<()> {
        match key.code {
            KeyCode::Tab if key.modifiers == KeyModifiers::NONE => {
                self.next_form_section();
            }
            KeyCode::BackTab | KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.previous_form_section();
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.save_draft();
            }
            KeyCode::Char('f') | KeyCode::Char('F')
                if self.current_form_section == FormSection::Review =>
            {
                self.show_finalization_dialog = true;
            }
            _ => {
                // Handle text input based on current section
                // This would involve updating the appropriate form_inputs field
            }
        }
        Ok(())
    }

    fn handle_calculator_input(&mut self, key: KeyEvent) -> io::Result<()> {
        match key.code {
            KeyCode::Char('c') | KeyCode::Char('C') => {
                self.calculate_power();
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.power_calculator = PowerCalculatorState {
                    test_type: "t-test".to_string(),
                    effect_size: "0.5".to_string(),
                    alpha: "0.05".to_string(),
                    power: "0.80".to_string(),
                    sample_size: String::new(),
                    calculated_result: None,
                };
            }
            _ => {}
        }
        Ok(())
    }

    fn next_form_section(&mut self) {
        self.current_form_section = match self.current_form_section {
            FormSection::StudyMetadata => FormSection::Hypotheses,
            FormSection::Hypotheses => FormSection::AnalysisPlan,
            FormSection::AnalysisPlan => FormSection::PowerAnalysis,
            FormSection::PowerAnalysis => FormSection::DataCollection,
            FormSection::DataCollection => FormSection::ExclusionCriteria,
            FormSection::ExclusionCriteria => FormSection::DecisionRules,
            FormSection::DecisionRules => FormSection::Review,
            FormSection::Review => FormSection::Review,
        };
    }

    fn previous_form_section(&mut self) {
        self.current_form_section = match self.current_form_section {
            FormSection::StudyMetadata => FormSection::StudyMetadata,
            FormSection::Hypotheses => FormSection::StudyMetadata,
            FormSection::AnalysisPlan => FormSection::Hypotheses,
            FormSection::PowerAnalysis => FormSection::AnalysisPlan,
            FormSection::DataCollection => FormSection::PowerAnalysis,
            FormSection::ExclusionCriteria => FormSection::DataCollection,
            FormSection::DecisionRules => FormSection::ExclusionCriteria,
            FormSection::Review => FormSection::DecisionRules,
        };
    }

    fn save_draft(&mut self) {
        // Update draft_registration from form_inputs
        self.update_draft_from_form();

        // Add to list if new
        if !self
            .preregistrations
            .iter()
            .any(|p| p.id == self.draft_registration.id)
        {
            self.preregistrations.push(self.draft_registration.clone());
        }

        self.message = Some(("Draft saved successfully".to_string(), MessageType::Success));
    }

    fn finalize_registration(&mut self) {
        self.update_draft_from_form();

        match self.draft_registration.finalize() {
            Ok(hash) => {
                // Update in list
                if let Some(prereg) = self
                    .preregistrations
                    .iter_mut()
                    .find(|p| p.id == self.draft_registration.id)
                {
                    *prereg = self.draft_registration.clone();
                } else {
                    self.preregistrations.push(self.draft_registration.clone());
                }

                self.message = Some((
                    format!("Registration finalized! Hash: {}...", &hash[..16]),
                    MessageType::Success,
                ));

                // Create validator for this registration
                self.analysis_validator =
                    Some(AnalysisValidator::new(self.draft_registration.clone()));

                // Switch to viewer
                self.current_view = DashboardView::ViewPreRegistration {
                    id: self.draft_registration.id.clone(),
                };
            }
            Err(e) => {
                self.message = Some((format!("Failed to finalize: {}", e), MessageType::Error));
            }
        }
    }

    fn calculate_power(&mut self) {
        if let (Ok(effect), Ok(alpha), Ok(power)) = (
            self.power_calculator.effect_size.parse::<f64>(),
            self.power_calculator.alpha.parse::<f64>(),
            self.power_calculator.power.parse::<f64>(),
        ) {
            let analyzer = PowerAnalyzer::new(0.05, 0.8);
            match analyzer.calculate_sample_size(
                StatisticalTestType::IndependentTTest,
                effect,
                power,
            ) {
                Ok(required_n) => {
                    self.power_calculator.calculated_result = Some(PowerCalculationResult {
                        required_n,
                        actual_power: power,
                        effect_size: effect,
                        alpha,
                    });
                }
                Err(e) => {
                    self.message = Some((
                        format!("Power calculation error: {}", e),
                        MessageType::Error,
                    ));
                }
            }
        } else {
            self.message = Some(("Invalid input values".to_string(), MessageType::Error));
        }
    }

    fn update_draft_from_form(&mut self) {
        // Update study metadata
        self.draft_registration.study = StudyMetadata {
            title: self.form_inputs.title.clone(),
            description: self.form_inputs.description.clone(),
            researchers: self.form_inputs.researchers.clone(),
            institution: self.form_inputs.institution.clone(),
            ethical_approval: if self.form_inputs.ethical_approval.is_empty() {
                None
            } else {
                Some(self.form_inputs.ethical_approval.clone())
            },
            funding_source: if self.form_inputs.funding_source.is_empty() {
                None
            } else {
                Some(self.form_inputs.funding_source.clone())
            },
            conflicts_of_interest: self.form_inputs.conflicts.clone(),
        };

        // Update power analysis
        if let (Ok(power), Ok(alpha), Ok(effect)) = (
            self.form_inputs.target_power.parse::<f64>(),
            self.form_inputs.alpha_level.parse::<f64>(),
            self.form_inputs.effect_size.parse::<f64>(),
        ) {
            self.draft_registration.analysis_plan.power_analysis = PowerAnalysisSpec {
                target_power: power,
                alpha_level: alpha,
                effect_size: effect,
                sample_size_calculation: format!(
                    "Power = {}, Alpha = {}, Effect = {}",
                    power, alpha, effect
                ),
                achieved_sample_size: None,
            };
        }

        // Update sample size
        if let Ok(n) = self.form_inputs.sample_size.parse::<usize>() {
            self.draft_registration.data_collection.target_sample_size = n;
        }
    }

    fn render_experiment_management(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        execute!(
            stdout,
            Clear(ClearType::All),
            cursor::MoveTo(0, 0),
            SetForegroundColor(Color::Cyan),
            Print("=== EXPERIMENT MANAGEMENT ===\n\n"),
            ResetColor,
            Print("Experiment management functionality coming soon...\n\n"),
            SetForegroundColor(Color::DarkGrey),
            Print("[B] Back | [Q] Quit\n"),
            ResetColor
        )
    }

    fn render_audio_recording(
        &mut self,
        stdout: &mut io::Stdout,
        _session_id: Option<&str>,
    ) -> io::Result<()> {
        execute!(
            stdout,
            Clear(ClearType::All),
            cursor::MoveTo(0, 0),
            SetForegroundColor(Color::Cyan),
            Print("=== AUDIO RECORDING ===\n\n"),
            ResetColor,
            Print("Audio recording functionality coming soon...\n\n"),
            SetForegroundColor(Color::DarkGrey),
            Print("[B] Back | [Q] Quit\n"),
            ResetColor
        )
    }

    fn render_sensor_integration(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        execute!(
            stdout,
            Clear(ClearType::All),
            cursor::MoveTo(0, 0),
            SetForegroundColor(Color::Cyan),
            Print("=== SENSOR INTEGRATION ===\n\n"),
            ResetColor,
            Print("Sensor integration functionality coming soon...\n\n"),
            SetForegroundColor(Color::DarkGrey),
            Print("[B] Back | [Q] Quit\n"),
            ResetColor
        )
    }

    fn render_data_collection(
        &mut self,
        stdout: &mut io::Stdout,
        _experiment_id: &str,
    ) -> io::Result<()> {
        execute!(
            stdout,
            Clear(ClearType::All),
            cursor::MoveTo(0, 0),
            SetForegroundColor(Color::Cyan),
            Print("=== DATA COLLECTION ===\n\n"),
            ResetColor,
            Print("Data collection functionality coming soon...\n\n"),
            SetForegroundColor(Color::DarkGrey),
            Print("[B] Back | [Q] Quit\n"),
            ResetColor
        )
    }

    fn render_irb_compliance(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        execute!(
            stdout,
            Clear(ClearType::All),
            cursor::MoveTo(0, 0),
            SetForegroundColor(Color::Cyan),
            Print("=== IRB COMPLIANCE ===\n\n"),
            ResetColor,
            Print("IRB compliance functionality coming soon...\n\n"),
            SetForegroundColor(Color::DarkGrey),
            Print("[B] Back | [Q] Quit\n"),
            ResetColor
        )
    }

    fn render_statistical_analysis(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        execute!(
            stdout,
            Clear(ClearType::All),
            cursor::MoveTo(0, 0),
            SetForegroundColor(Color::Cyan),
            Print("=== STATISTICAL ANALYSIS ===\n\n"),
            ResetColor,
            Print("Statistical analysis functionality coming soon...\n\n"),
            SetForegroundColor(Color::DarkGrey),
            Print("[B] Back | [Q] Quit\n"),
            ResetColor
        )
    }
}

/// Entry point for the research dashboard
pub fn run_research_dashboard() -> io::Result<()> {
    let mut dashboard = ResearchDashboard::new();
    dashboard.run()
}
