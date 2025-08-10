use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};
use std::io::{self, Write};
use std::time::Duration;

use crate::core::adaptive::AdaptiveScheduler;
use crate::core::learner::{LearnerMetrics, LearnerModel};
use crate::tasks::types::TaskSession;
use crate::core::topology::Topology;

pub enum AppState {
    MainMenu,
    SelectTopology,
    Training,
    ViewMetrics,
    TaskFeedback {
        correct: bool,
        correct_answer: String,
    },
}

pub struct TerminalApp {
    state: AppState,
    topology: Option<Topology>,
    session: Option<TaskSession>,
    scheduler: Option<AdaptiveScheduler>,
    learner_name: String,
    current_input: String,
}

impl TerminalApp {
    pub fn new() -> Self {
        TerminalApp {
            state: AppState::MainMenu,
            topology: None,
            session: None,
            scheduler: None,
            learner_name: String::new(),
            current_input: String::new(),
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, terminal::EnterAlternateScreen)?;

        loop {
            execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;

            match &self.state {
                AppState::MainMenu => self.render_main_menu(&mut stdout)?,
                AppState::SelectTopology => self.render_topology_selection(&mut stdout)?,
                AppState::Training => self.render_training(&mut stdout)?,
                AppState::ViewMetrics => self.render_metrics(&mut stdout)?,
                AppState::TaskFeedback {
                    correct,
                    correct_answer,
                } => self.render_feedback(&mut stdout, *correct, correct_answer)?,
            }

            stdout.flush()?;

            if event::poll(Duration::from_millis(10000))? {
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

    fn render_main_menu(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        execute!(
            stdout,
            SetForegroundColor(Color::Cyan),
            Print("═══════════════════════════════════════════════════\n"),
            Print("    ADAPTIVE GRAPH-CODED LEARNING SYSTEM\n"),
            Print("═══════════════════════════════════════════════════\n\n"),
            ResetColor,
            Print("Welcome to the Adaptive Training System!\n\n"),
            Print("This system helps you build flexible mental models\n"),
            Print("of structured knowledge using graph-based learning.\n\n"),
            SetForegroundColor(Color::Yellow),
            Print("Main Menu:\n"),
            ResetColor,
            Print("──────────────────────────────────────────────────\n"),
            Print("  [1] Start New Training Session\n"),
            Print("  [2] View Current Metrics\n"),
            Print("  [3] Research Dashboard (Pre-Registration)\n"),
            Print("  [4] About This System\n"),
            Print("  [Q] Quit\n\n"),
            Print("Select an option: ")
        )
    }

    fn render_topology_selection(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        execute!(
            stdout,
            SetForegroundColor(Color::Cyan),
            Print("Select Training Domain\n"),
            Print("═══════════════════════════════════════════════════\n\n"),
            ResetColor,
            SetForegroundColor(Color::Yellow),
            Print("Available Topologies:\n"),
            ResetColor,
            Print("──────────────────────────────────────────────────\n"),
            Print("  [1] English Alphabet (Linear, 26 items)\n"),
            Print("  [2] Days of Week (Cyclic, 7 items)\n"),
            Print("  [3] Custom Linear Sequence\n"),
            Print("  [4] Custom Cyclic Sequence\n"),
            Print("  [B] Back to Main Menu\n\n"),
            Print("Select topology: ")
        )
    }

    fn render_training(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        if let Some(session) = &self.session {
            if let Some(task) = &session.current_task {
                let stats = session.get_statistics();

                execute!(
                    stdout,
                    SetForegroundColor(Color::Cyan),
                    Print("Training Session\n"),
                    Print("═══════════════════════════════════════════════════\n\n"),
                    ResetColor,
                    SetForegroundColor(Color::Green),
                    Print(format!(
                        "Progress: {} tasks completed | Accuracy: {:.1}%\n\n",
                        stats.total_tasks,
                        stats.accuracy * 100.0
                    )),
                    ResetColor,
                    SetForegroundColor(Color::Yellow),
                    Print("Current Task:\n"),
                    ResetColor,
                    Print("──────────────────────────────────────────────────\n"),
                    Print(format!("{}\n\n", task.prompt))
                )?;

                if !task.options.is_empty() {
                    execute!(stdout, Print("Options:\n"))?;
                    for (i, option) in task.options.iter().enumerate() {
                        execute!(stdout, Print(format!("  [{}] {}\n", i + 1, option)))?;
                    }
                    execute!(stdout, Print("\n"))?;
                }

                execute!(
                    stdout,
                    Print(format!("Your answer: {}", self.current_input)),
                    Print("\n\n"),
                    SetForegroundColor(Color::DarkGrey),
                    Print("[Enter to submit | ESC for menu | Q to quit]\n"),
                    ResetColor
                )?;
            } else {
                execute!(
                    stdout,
                    Print("Preparing next task...\n"),
                    Print("Press any key to continue.")
                )?;
            }
        }
        Ok(())
    }

    fn render_feedback(
        &self,
        stdout: &mut io::Stdout,
        correct: bool,
        correct_answer: &str,
    ) -> io::Result<()> {
        execute!(
            stdout,
            SetForegroundColor(Color::Cyan),
            Print("Task Result\n"),
            Print("═══════════════════════════════════════════════════\n\n"),
            ResetColor
        )?;

        if correct {
            execute!(
                stdout,
                SetForegroundColor(Color::Green),
                Print("✓ CORRECT!\n\n"),
                ResetColor
            )?;
        } else {
            execute!(
                stdout,
                SetForegroundColor(Color::Red),
                Print("✗ INCORRECT\n\n"),
                ResetColor,
                Print(format!("Correct answer: {}\n\n", correct_answer))
            )?;
        }

        execute!(stdout, Print("Press any key to continue..."))
    }

    fn render_metrics(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        execute!(
            stdout,
            SetForegroundColor(Color::Cyan),
            Print("Learner Metrics\n"),
            Print("═══════════════════════════════════════════════════\n\n"),
            ResetColor
        )?;

        if let Some(scheduler) = &self.scheduler {
            let model = scheduler.get_learner_model();
            let metrics = LearnerMetrics::from_model(model);

            execute!(
                stdout,
                SetForegroundColor(Color::Yellow),
                Print("Performance Indicators:\n"),
                ResetColor,
                Print("──────────────────────────────────────────────────\n"),
                Print(format!(
                    "Bidirectionality Index: {:.3}\n",
                    metrics.bidirectionality_index
                )),
                Print(format!(
                    "Symbolic Distance Slope: {:.3}\n",
                    metrics.symbolic_distance_slope
                )),
                Print(format!(
                    "Chunk Boundary Penalty: {:.3}\n",
                    metrics.chunk_boundary_penalty
                )),
                Print(format!(
                    "Average Memory Strength: {:.2}%\n\n",
                    metrics.avg_memory_strength * 100.0
                )),
                SetForegroundColor(Color::Yellow),
                Print("Operation Proficiencies:\n"),
                ResetColor,
                Print("──────────────────────────────────────────────────\n")
            )?;

            for (op, prof) in metrics.operation_proficiencies {
                let bar_length = (prof * 20.0) as usize;
                let bar = "█".repeat(bar_length) + &"░".repeat(20 - bar_length);
                execute!(
                    stdout,
                    Print(format!("{:20} [{}] {:.0}%\n", op, bar, prof * 100.0))
                )?;
            }
        } else {
            execute!(
                stdout,
                Print("No active training session.\n"),
                Print("Start a training session to view metrics.\n")
            )?;
        }

        execute!(stdout, Print("\n[B] Back to Main Menu\n"))
    }

    fn handle_input(&mut self, key: KeyEvent) -> io::Result<bool> {
        match &self.state {
            AppState::MainMenu => self.handle_main_menu_input(key),
            AppState::SelectTopology => self.handle_topology_input(key),
            AppState::Training => self.handle_training_input(key),
            AppState::ViewMetrics => self.handle_metrics_input(key),
            AppState::TaskFeedback { .. } => {
                self.start_next_task();
                Ok(true)
            }
        }
    }

    fn handle_main_menu_input(&mut self, key: KeyEvent) -> io::Result<bool> {
        match key.code {
            KeyCode::Char('1') => {
                self.state = AppState::SelectTopology;
                Ok(true)
            }
            KeyCode::Char('2') => {
                self.state = AppState::ViewMetrics;
                Ok(true)
            }
            KeyCode::Char('3') => {
                // Launch the research dashboard in a separate session
                terminal::disable_raw_mode()?;
                execute!(io::stdout(), terminal::LeaveAlternateScreen)?;

                // Run the research dashboard
                crate::ui::dashboard::run_research_dashboard()?;

                // Re-enable raw mode and alternate screen when returning
                terminal::enable_raw_mode()?;
                execute!(io::stdout(), terminal::EnterAlternateScreen)?;
                Ok(true)
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => Ok(false),
            _ => Ok(true),
        }
    }

    fn handle_topology_input(&mut self, key: KeyEvent) -> io::Result<bool> {
        match key.code {
            KeyCode::Char('1') => {
                self.initialize_training(Topology::alphabet());
                Ok(true)
            }
            KeyCode::Char('2') => {
                self.initialize_training(Topology::days_of_week());
                Ok(true)
            }
            KeyCode::Char('b') | KeyCode::Char('B') => {
                self.state = AppState::MainMenu;
                Ok(true)
            }
            _ => Ok(true),
        }
    }

    fn handle_training_input(&mut self, key: KeyEvent) -> io::Result<bool> {
        match key.code {
            KeyCode::Char(c) => {
                self.current_input.push(c);
                Ok(true)
            }
            KeyCode::Backspace => {
                self.current_input.pop();
                Ok(true)
            }
            KeyCode::Enter => {
                if !self.current_input.is_empty() {
                    self.submit_answer();
                }
                Ok(true)
            }
            KeyCode::Esc => {
                self.state = AppState::MainMenu;
                Ok(true)
            }
            _ => Ok(true),
        }
    }

    fn handle_metrics_input(&mut self, key: KeyEvent) -> io::Result<bool> {
        match key.code {
            KeyCode::Char('b') | KeyCode::Char('B') => {
                self.state = AppState::MainMenu;
                Ok(true)
            }
            _ => Ok(true),
        }
    }

    fn initialize_training(&mut self, topology: Topology) {
        let learner_model = LearnerModel::new(self.learner_name.clone(), &topology);
        let scheduler = AdaptiveScheduler::new(learner_model, topology.clone());
        let session = TaskSession::new(topology.clone());

        self.topology = Some(topology);
        self.scheduler = Some(scheduler);
        self.session = Some(session);

        self.start_next_task();
    }

    fn start_next_task(&mut self) {
        if let (Some(scheduler), Some(session)) = (&mut self.scheduler, &mut self.session) {
            let task = scheduler.select_next_task();
            session.start_task(Some(task.task_type.clone()));
            self.current_input.clear();
            self.state = AppState::Training;
        }
    }

    fn submit_answer(&mut self) {
        if let (Some(scheduler), Some(session)) = (&mut self.scheduler, &mut self.session) {
            let response = session.submit_answer(self.current_input.clone());
            scheduler.update_model(&response.task, response.correct, response.response_time_ms);

            self.state = AppState::TaskFeedback {
                correct: response.correct,
                correct_answer: response.task.correct_answer.clone(),
            };
        }
    }
}
