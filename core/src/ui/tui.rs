use anyhow::Result;
use chrono::Local;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{
        Block, BorderType, Borders, Clear, Gauge, LineGauge, List, ListItem, Paragraph, Sparkline,
        Wrap,
    },
    Frame, Terminal,
};
use std::{io, time::Instant};

use crate::{
    core::{AdaptiveScheduler, LearnerMetrics, LearnerModel, Topology},
    backend::{BackendClient, BackendConfig, SessionToken},
    data::export::LearnerDataExport,
    tasks::{Task, TaskGenerator, TaskResponse, TaskSession},
};

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Welcome,
    ParticipantSetup,
    TopologySelection,
    ConfigureSession,
    Training,
    TaskFeedback,
    Metrics,
    Export,
    Help,
    Exiting,
}

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub participant_id: String,
    pub group: ExperimentGroup,
    pub trials_per_session: usize,
    pub topology: Topology,
    pub task_mix: TaskMix,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExperimentGroup {
    Adaptive,
    Linear,
    Yoked,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskMix {
    Adaptive,
    Fixed,
    Custom(Vec<String>),
}

pub struct App {
    state: AppState,
    config: SessionConfig,
    scheduler: Option<AdaptiveScheduler>,
    session: Option<TaskSession>,
    current_task: Option<Task>,

    // Backend connection
    backend_client: Option<BackendClient>,
    session_token: Option<SessionToken>,
    _backend_enabled: bool,
    sync_status: SyncStatus,

    // UI State
    input: String,
    messages: Vec<String>,
    selected_menu_item: usize,

    // Performance tracking
    trial_accuracies: Vec<f64>,
    trial_rts: Vec<f64>,
    current_streak: usize,
    best_streak: usize,

    // Task timing
    task_start_time: Option<Instant>,
    last_response: Option<TaskResponse>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyncStatus {
    Offline,
    Connected,
    Syncing,
    Error(String),
}

impl App {
    pub fn new() -> Self {
        // Try to initialize backend from environment or config file
        let backend_client = Self::init_backend();
        let backend_enabled = backend_client.is_some();
        let sync_status = if backend_enabled {
            SyncStatus::Connected
        } else {
            SyncStatus::Offline
        };

        App {
            state: AppState::Welcome,
            config: SessionConfig {
                participant_id: String::new(),
                group: ExperimentGroup::Adaptive,
                trials_per_session: 100,
                topology: Topology::alphabet(),
                task_mix: TaskMix::Adaptive,
            },
            scheduler: None,
            session: None,
            current_task: None,
            backend_client,
            session_token: None,
            _backend_enabled: backend_enabled,
            sync_status,
            input: String::new(),
            messages: vec!["Welcome to the Adaptive Learning System!".to_string()],
            selected_menu_item: 0,
            trial_accuracies: Vec::new(),
            trial_rts: Vec::new(),
            current_streak: 0,
            best_streak: 0,
            task_start_time: None,
            last_response: None,
        }
    }

    fn init_backend() -> Option<BackendClient> {
        // Try config file first
        if let Ok(config) = BackendConfig::from_file("backend.toml")
            .or_else(|_| BackendConfig::from_file("backend.json"))
        {
            if let Ok(client) = BackendClient::new(config) {
                // Test connection
                if client.test_connection().unwrap_or(false) {
                    return Some(client);
                }
            }
        }

        // Try environment variables
        if std::env::var("LEARNING_API_URL").is_ok() {
            let config = BackendConfig::from_env();
            if let Ok(client) = BackendClient::new(config) {
                if client.test_connection().unwrap_or(false) {
                    return Some(client);
                }
            }
        }

        None
    }

    pub fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| self.draw(f))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match self.state {
                        AppState::Exiting => return Ok(()),
                        _ => self.handle_input(key.code)?,
                    }
                }
            }
        }
    }

    fn draw(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(10),   // Main content
                Constraint::Length(3), // Footer
            ])
            .split(f.size());

        // Header
        self.draw_header(f, chunks[0]);

        // Main content based on state
        match self.state {
            AppState::Welcome => self.draw_welcome(f, chunks[1]),
            AppState::ParticipantSetup => self.draw_participant_setup(f, chunks[1]),
            AppState::TopologySelection => self.draw_topology_selection(f, chunks[1]),
            AppState::ConfigureSession => self.draw_session_config(f, chunks[1]),
            AppState::Training => self.draw_training(f, chunks[1]),
            AppState::TaskFeedback => self.draw_feedback(f, chunks[1]),
            AppState::Metrics => self.draw_metrics(f, chunks[1]),
            AppState::Export => self.draw_export(f, chunks[1]),
            AppState::Help => self.draw_help(f, chunks[1]),
            AppState::Exiting => self.draw_exit(f, chunks[1]),
        }

        // Footer
        self.draw_footer(f, chunks[2]);
    }

    fn draw_header(&self, f: &mut Frame, area: Rect) {
        // Split header into title and status
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(40), Constraint::Length(20)])
            .split(area);

        let header = Paragraph::new("🧠 Adaptive Graph-Coded Learning System")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        f.render_widget(header, chunks[0]);

        // Sync status indicator
        let (status_text, status_color) = match &self.sync_status {
            SyncStatus::Offline => ("⚪ Offline", Color::Gray),
            SyncStatus::Connected => ("🟢 Connected", Color::Green),
            SyncStatus::Syncing => ("🔄 Syncing...", Color::Yellow),
            SyncStatus::Error(_) => ("🔴 Error", Color::Red),
        };

        let status = Paragraph::new(status_text)
            .style(Style::default().fg(status_color))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(status_color)),
            );
        f.render_widget(status, chunks[1]);
    }

    fn draw_footer(&self, f: &mut Frame, area: Rect) {
        let footer_text = match self.state {
            AppState::Training => "Type your answer and press Enter | ESC to pause",
            AppState::Welcome => "Press Enter to continue | Q to quit",
            _ => "Navigate with ↑↓ | Enter to select | ESC to go back | H for help",
        };

        let footer = Paragraph::new(footer_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::TOP));
        f.render_widget(footer, area);
    }

    fn draw_welcome(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(50),
                Constraint::Percentage(20),
            ])
            .split(area);

        // Logo/Title
        let title = vec![
            "╔═══════════════════════════════════════╗",
            "║   ADAPTIVE LEARNING SYSTEM v1.0      ║",
            "║   Build Flexible Mental Models       ║",
            "╚═══════════════════════════════════════╝",
        ];

        let title_widget = Paragraph::new(title.join("\n"))
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        f.render_widget(title_widget, chunks[0]);

        // Description
        let description = vec![
            "",
            "This system uses adaptive training algorithms to help you",
            "build flexible mental representations of structured knowledge.",
            "",
            "Based on Bayesian cognitive modeling and Expected Information Gain,",
            "the system personalizes your learning experience in real-time.",
            "",
            "You'll complete various tasks that test your knowledge of sequences,",
            "relationships, and patterns. The system adapts to your performance",
            "to optimize your learning efficiency.",
        ];

        let desc_widget = Paragraph::new(description.join("\n"))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        f.render_widget(desc_widget, chunks[1]);

        // Start prompt
        let prompt = Paragraph::new("Press ENTER to begin setup")
            .style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK),
            )
            .alignment(Alignment::Center);
        f.render_widget(prompt, chunks[2]);
    }

    fn draw_participant_setup(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Min(5),
            ])
            .split(area);

        // Title
        let title = Paragraph::new("Participant Setup")
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        f.render_widget(title, chunks[0]);

        // Participant ID input
        let input_block = Block::default()
            .title("Participant ID")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        let input = Paragraph::new(self.input.as_str())
            .style(Style::default().fg(Color::White))
            .block(input_block);
        f.render_widget(input, chunks[1]);

        // Experiment group selection
        let groups = vec![
            ListItem::new("1. Adaptive Training").style(if self.selected_menu_item == 0 {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            }),
            ListItem::new("2. Linear Training").style(if self.selected_menu_item == 1 {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            }),
            ListItem::new("3. Yoked Control").style(if self.selected_menu_item == 2 {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            }),
        ];

        let group_list = List::new(groups)
            .block(
                Block::default()
                    .title("Experiment Group")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");
        f.render_widget(group_list, chunks[2]);

        // Instructions
        let instructions = Paragraph::new(
            "Enter your participant ID and select experiment group.\nPress TAB to switch between fields."
        )
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
        f.render_widget(instructions, chunks[3]);
    }

    fn draw_training(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(area);

        // Left side - Task display
        self.draw_task_area(f, chunks[0]);

        // Right side - Progress and stats
        self.draw_progress_panel(f, chunks[1]);
    }

    fn draw_task_area(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),    // Task content
                Constraint::Length(3), // Input field
                Constraint::Length(2), // Timer
            ])
            .split(area);

        // Task display
        if let Some(task) = &self.current_task {
            let task_text = format!("\nTask Type: {:?}\n\n{}", task.task_type, task.prompt);

            let task_widget = Paragraph::new(task_text)
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .title(format!("Task #{}", self.trial_accuracies.len() + 1))
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Cyan)),
                )
                .wrap(Wrap { trim: true });
            f.render_widget(task_widget, chunks[0]);
        }

        // Input field
        let input_block = Block::default()
            .title("Your Answer")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        let input = Paragraph::new(self.input.as_str())
            .style(Style::default().fg(Color::White))
            .block(input_block);
        f.render_widget(input, chunks[1]);

        // Timer
        if let Some(start_time) = self.task_start_time {
            let elapsed = start_time.elapsed().as_secs_f64();
            let timer_text = format!("Time: {:.1}s", elapsed);
            let timer = Paragraph::new(timer_text)
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            f.render_widget(timer, chunks[2]);
        }
    }

    fn draw_progress_panel(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6), // Overall progress
                Constraint::Length(4), // Current streak
                Constraint::Length(8), // Recent accuracy
                Constraint::Min(5),    // Recent RTs
            ])
            .split(area);

        // Overall progress
        let progress = self.trial_accuracies.len() as f64 / self.config.trials_per_session as f64;
        let progress_widget = Gauge::default()
            .block(
                Block::default()
                    .title("Session Progress")
                    .borders(Borders::ALL),
            )
            .gauge_style(Style::default().fg(Color::Green))
            .percent((progress * 100.0) as u16)
            .label(format!(
                "{}/{}",
                self.trial_accuracies.len(),
                self.config.trials_per_session
            ));
        f.render_widget(progress_widget, chunks[0]);

        // Streak counter
        let streak_text = vec![
            format!("Current: {}", self.current_streak),
            format!("Best: {}", self.best_streak),
        ];
        let streak_widget = Paragraph::new(streak_text.join("\n"))
            .block(Block::default().title("Streak").borders(Borders::ALL))
            .style(Style::default().fg(if self.current_streak > 0 {
                Color::Green
            } else {
                Color::White
            }));
        f.render_widget(streak_widget, chunks[1]);

        // Recent accuracy sparkline
        if !self.trial_accuracies.is_empty() {
            let recent_acc: Vec<u64> = self
                .trial_accuracies
                .iter()
                .rev()
                .take(20)
                .rev()
                .map(|&a| (a * 100.0) as u64)
                .collect();

            let avg_accuracy =
                self.trial_accuracies.iter().sum::<f64>() / self.trial_accuracies.len() as f64;

            let sparkline = Sparkline::default()
                .block(
                    Block::default()
                        .title(format!("Accuracy (Avg: {:.1}%)", avg_accuracy * 100.0))
                        .borders(Borders::ALL),
                )
                .data(&recent_acc)
                .style(Style::default().fg(Color::Cyan));
            f.render_widget(sparkline, chunks[2]);
        }

        // Recent RTs
        if !self.trial_rts.is_empty() {
            let recent_rts: Vec<u64> = self
                .trial_rts
                .iter()
                .rev()
                .take(20)
                .rev()
                .map(|&rt| rt as u64)
                .collect();

            let avg_rt = self.trial_rts.iter().sum::<f64>() / self.trial_rts.len() as f64;

            let rt_sparkline = Sparkline::default()
                .block(
                    Block::default()
                        .title(format!("Response Time (Avg: {:.0}ms)", avg_rt))
                        .borders(Borders::ALL),
                )
                .data(&recent_rts)
                .style(Style::default().fg(Color::Magenta));
            f.render_widget(rt_sparkline, chunks[3]);
        }
    }

    fn draw_feedback(&self, f: &mut Frame, area: Rect) {
        if let Some(response) = &self.last_response {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                ])
                .split(area);

            // Result indicator
            let result_text = if response.correct {
                "✓ CORRECT!"
            } else {
                "✗ INCORRECT"
            };

            let result_color = if response.correct {
                Color::Green
            } else {
                Color::Red
            };

            let result = Paragraph::new(result_text)
                .style(
                    Style::default()
                        .fg(result_color)
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(result_color)),
                );
            f.render_widget(result, chunks[0]);

            // Feedback details
            let feedback_text = vec![
                format!("Your answer: {}", response.user_answer),
                format!("Correct answer: {}", response.task.correct_answer),
                format!("Response time: {}ms", response.response_time_ms),
            ];

            let feedback = Paragraph::new(feedback_text.join("\n"))
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::NONE));
            f.render_widget(feedback, chunks[1]);

            // Continue prompt
            let prompt = Paragraph::new("Press any key to continue")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            f.render_widget(prompt, chunks[2]);
        }
    }

    fn draw_metrics(&self, f: &mut Frame, area: Rect) {
        if let Some(scheduler) = &self.scheduler {
            let model = scheduler.get_learner_model();
            let metrics = LearnerMetrics::from_model(model);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ])
                .split(area);

            // Title
            let title = Paragraph::new("Performance Metrics")
                .style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Center);
            f.render_widget(title, chunks[0]);

            // Key metrics
            let metrics_text = vec![
                format!(
                    "Bidirectionality Index: {:.3}",
                    metrics.bidirectionality_index
                ),
                format!(
                    "Symbolic Distance Slope: {:.3}",
                    metrics.symbolic_distance_slope
                ),
                format!(
                    "Chunk Boundary Penalty: {:.3}",
                    metrics.chunk_boundary_penalty
                ),
                format!(
                    "Average Memory Strength: {:.2}%",
                    metrics.avg_memory_strength * 100.0
                ),
                String::new(),
                format!("Total Trials: {}", self.trial_accuracies.len()),
                format!(
                    "Overall Accuracy: {:.1}%",
                    if self.trial_accuracies.is_empty() {
                        0.0
                    } else {
                        self.trial_accuracies.iter().sum::<f64>()
                            / self.trial_accuracies.len() as f64
                            * 100.0
                    }
                ),
            ];

            let metrics_widget = Paragraph::new(metrics_text.join("\n")).block(
                Block::default()
                    .title("Performance Indicators")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            );
            f.render_widget(metrics_widget, chunks[1]);

            // Operation proficiencies
            let mut proficiency_text = Vec::new();
            for (op, prof) in metrics.operation_proficiencies.iter().take(5) {
                let bar_length = (prof * 20.0) as usize;
                let bar = "█".repeat(bar_length) + &"░".repeat(20 - bar_length);
                proficiency_text.push(format!("{:15} {} {:.0}%", op, bar, prof * 100.0));
            }

            let prof_widget = Paragraph::new(proficiency_text.join("\n")).block(
                Block::default()
                    .title("Operation Proficiencies")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            );
            f.render_widget(prof_widget, chunks[2]);
        } else {
            let no_data =
                Paragraph::new("No training data available.\nComplete a training session first.")
                    .style(Style::default().fg(Color::DarkGray))
                    .alignment(Alignment::Center);
            f.render_widget(no_data, area);
        }
    }

    fn draw_export(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(10),
                Constraint::Min(5),
            ])
            .split(area);

        // Title
        let title = Paragraph::new("Export Data")
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        f.render_widget(title, chunks[0]);

        // Export options
        let export_options = vec![
            ListItem::new("1. Export to CSV (R-compatible)"),
            ListItem::new("2. Export to JSON (Complete data)"),
            ListItem::new("3. Export Session Summary"),
            ListItem::new("4. Export Model Parameters"),
            ListItem::new("5. Export All Formats"),
        ];

        let options = List::new(export_options)
            .block(
                Block::default()
                    .title("Export Options")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");
        f.render_widget(options, chunks[1]);

        // Status messages
        let messages_text = self
            .messages
            .iter()
            .rev()
            .take(5)
            .rev()
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");
        let messages = Paragraph::new(messages_text).block(
            Block::default()
                .title("Export Status")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        );
        f.render_widget(messages, chunks[2]);
    }

    fn draw_help(&self, f: &mut Frame, area: Rect) {
        let help_text = vec![
            "KEYBOARD SHORTCUTS",
            "═══════════════════════════════════════",
            "",
            "General:",
            "  ESC      - Go back / Cancel",
            "  Q        - Quit application",
            "  H        - Show this help",
            "  Tab      - Switch between fields",
            "  Enter    - Confirm selection",
            "",
            "During Training:",
            "  Type answer and press Enter to submit",
            "  ESC      - Pause training",
            "  M        - View metrics",
            "",
            "Navigation:",
            "  ↑/↓      - Navigate menus",
            "  ←/→      - Adjust values",
            "  Space    - Toggle options",
            "",
            "Export:",
            "  E        - Open export menu",
            "  S        - Quick save session",
        ];

        let help_widget = Paragraph::new(help_text.join("\n")).block(
            Block::default()
                .title("Help")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        );
        f.render_widget(help_widget, area);
    }

    fn draw_exit(&self, f: &mut Frame, area: Rect) {
        let popup_area = centered_rect(50, 20, area);

        f.render_widget(Clear, popup_area);

        let exit_text = vec![
            "",
            "Are you sure you want to exit?",
            "",
            "Press Y to confirm or N to cancel",
        ];

        let exit_widget = Paragraph::new(exit_text.join("\n"))
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title("Exit Confirmation")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().fg(Color::Red)),
            );
        f.render_widget(exit_widget, popup_area);
    }

    fn draw_topology_selection(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(5),
            ])
            .split(area);

        let title = Paragraph::new("Select Knowledge Domain")
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        f.render_widget(title, chunks[0]);

        let topology_options = vec![
            ListItem::new("1. Alphabet (A-Z)").style(if self.selected_menu_item == 0 {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            }),
            ListItem::new("2. Days of Week").style(if self.selected_menu_item == 1 {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            }),
            ListItem::new("3. Months of Year").style(if self.selected_menu_item == 2 {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            }),
            ListItem::new("4. Numbers (1-20)").style(if self.selected_menu_item == 3 {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            }),
        ];

        let list = List::new(topology_options).block(
            Block::default()
                .title("Available Domains")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );
        f.render_widget(list, chunks[1]);

        let description = match self.selected_menu_item {
            0 => "Learn the English alphabet sequence (26 items)",
            1 => "Learn the days of the week (7 items)",
            2 => "Learn the months of the year (12 items)",
            3 => "Learn number sequence 1-20 (20 items)",
            _ => "",
        };

        let desc_widget = Paragraph::new(description)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::TOP));
        f.render_widget(desc_widget, chunks[2]);
    }

    fn draw_session_config(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(4),
                Constraint::Length(4),
                Constraint::Length(4),
                Constraint::Min(5),
            ])
            .split(area);

        let title = Paragraph::new("Session Configuration")
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        f.render_widget(title, chunks[0]);

        // Trials per session
        let trials_gauge = LineGauge::default()
            .block(
                Block::default()
                    .title(format!(
                        "Trials per Session: {}",
                        self.config.trials_per_session
                    ))
                    .borders(Borders::ALL),
            )
            .gauge_style(Style::default().fg(Color::Green))
            .ratio(self.config.trials_per_session as f64 / 200.0);
        f.render_widget(trials_gauge, chunks[1]);

        // Task mix
        let task_mix = Paragraph::new(format!("Task Mix: {:?}", self.config.task_mix)).block(
            Block::default()
                .title("Task Selection Strategy")
                .borders(Borders::ALL),
        );
        f.render_widget(task_mix, chunks[2]);

        // Summary
        let summary = vec![
            format!("Participant: {}", self.config.participant_id),
            format!("Group: {:?}", self.config.group),
            format!("Domain: {:?}", self.config.topology.topology_type),
            format!("Trials: {}", self.config.trials_per_session),
        ];

        let summary_widget = Paragraph::new(summary.join("\n")).block(
            Block::default()
                .title("Configuration Summary")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );
        f.render_widget(summary_widget, chunks[3]);

        let instructions =
            Paragraph::new("Use ←/→ to adjust values. Press Enter to start training.")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
        f.render_widget(instructions, chunks[4]);
    }

    fn handle_input(&mut self, key: KeyCode) -> Result<()> {
        match self.state {
            AppState::Welcome => match key {
                KeyCode::Enter => self.state = AppState::ParticipantSetup,
                KeyCode::Char('q') | KeyCode::Char('Q') => self.state = AppState::Exiting,
                _ => {}
            },
            AppState::ParticipantSetup => match key {
                KeyCode::Char(c) => self.input.push(c),
                KeyCode::Backspace => {
                    self.input.pop();
                }
                KeyCode::Tab => {
                    self.selected_menu_item = (self.selected_menu_item + 1) % 3;
                }
                KeyCode::Up => {
                    self.selected_menu_item = self.selected_menu_item.saturating_sub(1);
                }
                KeyCode::Down => {
                    self.selected_menu_item = (self.selected_menu_item + 1).min(2);
                }
                KeyCode::Enter => {
                    if !self.input.is_empty() {
                        self.config.participant_id = self.input.clone();
                        self.config.group = match self.selected_menu_item {
                            0 => ExperimentGroup::Adaptive,
                            1 => ExperimentGroup::Linear,
                            2 => ExperimentGroup::Yoked,
                            _ => ExperimentGroup::Adaptive,
                        };
                        self.input.clear();
                        self.selected_menu_item = 0;
                        self.state = AppState::TopologySelection;
                    }
                }
                KeyCode::Esc => self.state = AppState::Welcome,
                _ => {}
            },
            AppState::TopologySelection => match key {
                KeyCode::Up => {
                    self.selected_menu_item = self.selected_menu_item.saturating_sub(1);
                }
                KeyCode::Down => {
                    self.selected_menu_item = (self.selected_menu_item + 1).min(3);
                }
                KeyCode::Enter => {
                    self.config.topology = match self.selected_menu_item {
                        0 => Topology::alphabet(),
                        1 => Topology::days_of_week(),
                        2 => Topology::alphabet(), // TODO: Add months_of_year
                        3 => Topology::alphabet(), // TODO: Add number_sequence
                        _ => Topology::alphabet(),
                    };
                    self.state = AppState::ConfigureSession;
                }
                KeyCode::Esc => self.state = AppState::ParticipantSetup,
                _ => {}
            },
            AppState::ConfigureSession => match key {
                KeyCode::Left => {
                    self.config.trials_per_session =
                        (self.config.trials_per_session.saturating_sub(10)).max(10);
                }
                KeyCode::Right => {
                    self.config.trials_per_session = (self.config.trials_per_session + 10).min(200);
                }
                KeyCode::Enter => {
                    self.start_training();
                    self.state = AppState::Training;
                }
                KeyCode::Esc => self.state = AppState::TopologySelection,
                _ => {}
            },
            AppState::Training => match key {
                KeyCode::Char(c) => {
                    self.input.push(c);
                }
                KeyCode::Backspace => {
                    self.input.pop();
                }
                KeyCode::Enter => {
                    if !self.input.is_empty() {
                        self.submit_answer();
                        self.state = AppState::TaskFeedback;
                    }
                }
                KeyCode::Esc => self.state = AppState::Metrics,
                _ => {}
            },
            AppState::TaskFeedback => {
                // Any key continues to next task
                if self.trial_accuracies.len() < self.config.trials_per_session {
                    self.next_task();
                    self.state = AppState::Training;
                } else {
                    self.complete_session();
                    self.state = AppState::Metrics;
                }
            }
            AppState::Metrics => match key {
                KeyCode::Char('e') | KeyCode::Char('E') => self.state = AppState::Export,
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    if self.trial_accuracies.len() < self.config.trials_per_session {
                        self.state = AppState::Training;
                    }
                }
                KeyCode::Esc => self.state = AppState::Welcome,
                _ => {}
            },
            AppState::Export => match key {
                KeyCode::Char('1') => {
                    self.export_csv()?;
                }
                KeyCode::Char('2') => {
                    self.export_json()?;
                }
                KeyCode::Char('5') => {
                    self.export_all()?;
                }
                KeyCode::Esc => self.state = AppState::Metrics,
                _ => {}
            },
            AppState::Help => match key {
                KeyCode::Esc => self.state = AppState::Welcome,
                _ => {}
            },
            AppState::Exiting => match key {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    // Exit confirmed - handled in main loop
                }
                KeyCode::Char('n') | KeyCode::Char('N') => self.state = AppState::Welcome,
                _ => {}
            },
        }
        Ok(())
    }

    fn start_training(&mut self) {
        // Register with backend if available
        if let Some(client) = &self.backend_client {
            let group_str = format!("{:?}", self.config.group);
            match client.register_participant(&self.config.participant_id, &group_str) {
                Ok(token) => {
                    self.session_token = Some(token);
                    self.sync_status = SyncStatus::Connected;
                    self.messages.push("✓ Connected to backend".to_string());

                    // Start session
                    if let Some(token) = &self.session_token {
                        if let Err(e) = client.start_session(token) {
                            self.messages
                                .push(format!("Warning: Failed to start remote session: {}", e));
                        }
                    }
                }
                Err(e) => {
                    self.sync_status = SyncStatus::Error(e.to_string());
                    self.messages
                        .push(format!("⚠ Backend registration failed: {}", e));
                    self.messages.push("Continuing in offline mode".to_string());
                }
            }
        }

        let learner_model =
            LearnerModel::new(self.config.participant_id.clone(), &self.config.topology);

        self.scheduler = Some(AdaptiveScheduler::new(
            learner_model,
            self.config.topology.clone(),
        ));
        self.session = Some(TaskSession::new(self.config.topology.clone()));

        self.trial_accuracies.clear();
        self.trial_rts.clear();
        self.current_streak = 0;
        self.best_streak = 0;

        self.next_task();
    }

    fn next_task(&mut self) {
        if let Some(scheduler) = &mut self.scheduler {
            let task = scheduler.select_next_task();
            self.current_task = Some(task);
            self.task_start_time = Some(Instant::now());
            self.input.clear();
        }
    }

    fn submit_answer(&mut self) {
        if let (Some(task), Some(start_time)) = (&self.current_task, self.task_start_time) {
            let rt_ms = start_time.elapsed().as_millis() as u128;
            let correct = self.input.trim() == task.correct_answer;

            let response = TaskResponse {
                task: task.clone(),
                user_answer: self.input.trim().to_string(),
                correct,
                response_time_ms: rt_ms,
                timestamp: chrono::Utc::now(),
            };

            // Update metrics
            self.trial_accuracies.push(if correct { 1.0 } else { 0.0 });
            self.trial_rts.push(rt_ms as f64);

            if correct {
                self.current_streak += 1;
                self.best_streak = self.best_streak.max(self.current_streak);
            } else {
                self.current_streak = 0;
            }

            // Update model
            if let Some(scheduler) = &mut self.scheduler {
                scheduler.update_model(&response.task, response.correct, response.response_time_ms);
            }

            // Store session
            if let Some(session) = &mut self.session {
                session.history.push(response.clone());
            }

            // Send to backend
            if let Some(client) = &mut self.backend_client {
                client.buffer_response(response.clone());

                // Check if we should sync
                if client.should_sync() {
                    self.sync_status = SyncStatus::Syncing;
                    match client.sync_responses() {
                        Ok(_) => {
                            self.sync_status = SyncStatus::Connected;
                        }
                        Err(e) => {
                            self.sync_status = SyncStatus::Error(e.to_string());
                            self.messages.push(format!("Sync failed: {}", e));
                        }
                    }
                }

                // Send metrics update
                if let Some(scheduler) = &self.scheduler {
                    let metrics = LearnerMetrics::from_model(scheduler.get_learner_model());
                    let _ = client.send_metrics(&self.config.participant_id, &metrics);
                }
            }

            self.last_response = Some(response);
        }
    }

    fn complete_session(&mut self) {
        self.messages.push(format!(
            "Session completed! Accuracy: {:.1}%, Avg RT: {:.0}ms",
            self.trial_accuracies.iter().sum::<f64>() / self.trial_accuracies.len() as f64 * 100.0,
            self.trial_rts.iter().sum::<f64>() / self.trial_rts.len() as f64
        ));

        // Final sync with backend
        if let Some(client) = &mut self.backend_client {
            // Sync any remaining responses
            self.sync_status = SyncStatus::Syncing;
            if let Err(e) = client.sync_responses() {
                self.messages
                    .push(format!("Warning: Final sync failed: {}", e));
            }

            // Upload complete session data
            if let (Some(scheduler), Some(session)) = (&self.scheduler, &self.session) {
                // Create a new TaskSession with just the history
                let export_session = TaskSession {
                    generator: TaskGenerator::new(self.config.topology.clone()),
                    current_task: None,
                    task_start_time: None,
                    history: session.history.clone(),
                };
                let export = LearnerDataExport::from_learner_model(
                    scheduler.get_learner_model(),
                    vec![export_session],
                    Some(format!("{:?}_experiment", self.config.group)),
                );

                if let Err(e) = client.upload_session(&export) {
                    self.messages
                        .push(format!("Warning: Session upload failed: {}", e));
                } else {
                    self.sync_status = SyncStatus::Connected;
                    self.messages
                        .push("✓ Session data uploaded to backend".to_string());
                }
            }
        }
    }

    fn export_csv(&mut self) -> Result<()> {
        if let (Some(scheduler), Some(session)) = (&self.scheduler, &self.session) {
            // Create a new TaskSession with just the history
            let export_session = TaskSession {
                generator: TaskGenerator::new(self.config.topology.clone()),
                current_task: None,
                task_start_time: None,
                history: session.history.clone(),
            };
            let export = LearnerDataExport::from_learner_model(
                scheduler.get_learner_model(),
                vec![export_session],
                Some(format!("{:?}_experiment", self.config.group)),
            );

            let path = std::path::Path::new("data").join(format!(
                "{}_{}_responses.csv",
                self.config.participant_id,
                Local::now().format("%Y%m%d_%H%M%S")
            ));

            std::fs::create_dir_all("data")?;
            export.save_csv(&path)?;

            self.messages
                .push(format!("Exported CSV to: {}", path.display()));
        }
        Ok(())
    }

    fn export_json(&mut self) -> Result<()> {
        if let (Some(scheduler), Some(session)) = (&self.scheduler, &self.session) {
            // Create a new TaskSession with just the history
            let export_session = TaskSession {
                generator: TaskGenerator::new(self.config.topology.clone()),
                current_task: None,
                task_start_time: None,
                history: session.history.clone(),
            };
            let export = LearnerDataExport::from_learner_model(
                scheduler.get_learner_model(),
                vec![export_session],
                Some(format!("{:?}_experiment", self.config.group)),
            );

            let path = std::path::Path::new("data").join(format!(
                "{}_{}_full.json",
                self.config.participant_id,
                Local::now().format("%Y%m%d_%H%M%S")
            ));

            std::fs::create_dir_all("data")?;
            export.save_to_file(&path)?;

            self.messages
                .push(format!("Exported JSON to: {}", path.display()));
        }
        Ok(())
    }

    fn export_all(&mut self) -> Result<()> {
        if let (Some(scheduler), Some(session)) = (&self.scheduler, &self.session) {
            // Create a new TaskSession with just the history
            let export_session = TaskSession {
                generator: TaskGenerator::new(self.config.topology.clone()),
                current_task: None,
                task_start_time: None,
                history: session.history.clone(),
            };
            let export = LearnerDataExport::from_learner_model(
                scheduler.get_learner_model(),
                vec![export_session],
                Some(format!("{:?}_experiment", self.config.group)),
            );

            let base_path = std::path::Path::new("data").join(format!(
                "{}_{}",
                self.config.participant_id,
                Local::now().format("%Y%m%d_%H%M%S")
            ));

            export.export_all_formats(&base_path)?;

            self.messages
                .push(format!("Exported all formats to: {}", base_path.display()));
        }
        Ok(())
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn run() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = app.run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }

    Ok(())
}
