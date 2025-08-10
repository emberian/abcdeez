// demo.rs - Enhanced guided demo system with UI highlighting and interactive walkthroughs

use crate::{AppData, Screen};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a single step in the demo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoStep {
    pub id: String,
    pub title: String,
    pub description: String,
    pub highlight_elements: Vec<String>, // UI element IDs to highlight
    pub action: Option<DemoAction>,
    pub navigation: Option<Screen>,
    pub wait_for_user: bool,
    pub auto_advance_ms: Option<u64>, // Auto-advance after milliseconds
}

/// Actions that can be performed during a demo step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DemoAction {
    ClickButton(String),
    EnterText { field: String, text: String },
    SelectDomain(String),
    SubmitAnswer(String),
    RequestHint,
    NavigateTo(Screen),
    ShowTooltip { element: String, text: String },
    RunMiniDemo, // Run a mini automated sequence
}

/// Highlight style for UI elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightStyle {
    pub color: String,
    pub animation: HighlightAnimation,
    pub opacity: f32,
    pub border_width: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HighlightAnimation {
    None,
    Pulse,
    Glow,
    Bounce,
    Arrow,
}

impl Default for HighlightStyle {
    fn default() -> Self {
        Self {
            color: "#4CAF50".to_string(),
            animation: HighlightAnimation::Pulse,
            opacity: 0.3,
            border_width: 3.0,
        }
    }
}

/// Demo scenario - a collection of steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoScenario {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<DemoStep>,
    pub completion_message: String,
}

/// Main demo controller
pub struct DemoController {
    pub scenarios: HashMap<String, DemoScenario>,
    pub current_scenario: Option<String>,
    pub current_step_index: usize,
    pub highlights: HashMap<String, HighlightStyle>,
    pub is_active: bool,
    pub is_paused: bool,
    pub step_history: Vec<String>,
    pub tooltips: HashMap<String, String>,
}

impl DemoController {
    pub fn new() -> Self {
        let mut controller = Self {
            scenarios: HashMap::new(),
            current_scenario: None,
            current_step_index: 0,
            highlights: HashMap::new(),
            is_active: false,
            is_paused: false,
            step_history: Vec::new(),
            tooltips: HashMap::new(),
        };

        // Initialize with default scenarios
        controller.init_default_scenarios();
        controller
    }

    fn init_default_scenarios(&mut self) {
        // Quick Tour scenario
        let quick_tour = DemoScenario {
            id: "quick_tour".to_string(),
            name: "Quick Tour".to_string(),
            description: "A brief introduction to the main features".to_string(),
            steps: vec![
                DemoStep {
                    id: "welcome".to_string(),
                    title: "Welcome to Adaptive Learning!".to_string(),
                    description: "This guided tour will show you the key features of our adaptive learning system. You can click Next to continue or End Demo at any time.".to_string(),
                    highlight_elements: vec![],
                    action: None,
                    navigation: Some(Screen::Welcome),
                    wait_for_user: true,
                    auto_advance_ms: None,
                },
                DemoStep {
                    id: "navigation".to_string(),
                    title: "Navigation Bar".to_string(),
                    description: "Use the navigation bar at the top to move between different sections of the application.".to_string(),
                    highlight_elements: vec!["nav_bar".to_string()],
                    action: Some(DemoAction::ShowTooltip {
                        element: "nav_bar".to_string(),
                        text: "Click any tab to navigate".to_string(),
                    }),
                    navigation: None,
                    wait_for_user: true,
                    auto_advance_ms: None,
                },
                DemoStep {
                    id: "domain_selection".to_string(),
                    title: "Choose Your Learning Domain".to_string(),
                    description: "Select a domain to start learning. Each domain has different content and difficulty levels.".to_string(),
                    highlight_elements: vec!["domain_cards".to_string()],
                    action: None,
                    navigation: Some(Screen::DomainSelection),
                    wait_for_user: true,
                    auto_advance_ms: None,
                },
                DemoStep {
                    id: "training_intro".to_string(),
                    title: "Training Interface".to_string(),
                    description: "This is where the learning happens! You'll see tasks, submit answers, and get instant feedback.".to_string(),
                    highlight_elements: vec!["training_area".to_string(), "task_display".to_string()],
                    action: None,
                    navigation: Some(Screen::Training),
                    wait_for_user: true,
                    auto_advance_ms: None,
                },
                DemoStep {
                    id: "hints_system".to_string(),
                    title: "Hint System".to_string(),
                    description: "Stuck? Click 'Get Hint' for assistance. The system adapts hints based on your progress.".to_string(),
                    highlight_elements: vec!["hint_button".to_string(), "hint_display".to_string()],
                    action: Some(DemoAction::ShowTooltip {
                        element: "hint_button".to_string(),
                        text: "Hints are context-aware!".to_string(),
                    }),
                    navigation: None,
                    wait_for_user: true,
                    auto_advance_ms: None,
                },
                DemoStep {
                    id: "dashboard_overview".to_string(),
                    title: "Your Progress Dashboard".to_string(),
                    description: "Track your learning progress with detailed metrics, charts, and activity history.".to_string(),
                    highlight_elements: vec!["dashboard_metrics".to_string(), "charts_area".to_string()],
                    action: None,
                    navigation: Some(Screen::Dashboard),
                    wait_for_user: true,
                    auto_advance_ms: None,
                },
                DemoStep {
                    id: "settings_customization".to_string(),
                    title: "Customize Your Experience".to_string(),
                    description: "Adjust difficulty, enable/disable features, and export your data in Settings.".to_string(),
                    highlight_elements: vec!["settings_panel".to_string()],
                    action: None,
                    navigation: Some(Screen::Settings),
                    wait_for_user: true,
                    auto_advance_ms: None,
                },
                DemoStep {
                    id: "tour_complete".to_string(),
                    title: "Tour Complete!".to_string(),
                    description: "You're ready to start learning! Try creating a session or explore on your own.".to_string(),
                    highlight_elements: vec![],
                    action: None,
                    navigation: Some(Screen::Welcome),
                    wait_for_user: true,
                    auto_advance_ms: None,
                },
            ],
            completion_message: "Great job completing the tour! You're ready to start your adaptive learning journey.".to_string(),
        };

        // Full Training Demo scenario
        let training_demo = DemoScenario {
            id: "training_demo".to_string(),
            name: "Complete Training Session".to_string(),
            description: "Experience a full training session with automated responses".to_string(),
            steps: vec![
                DemoStep {
                    id: "start_session".to_string(),
                    title: "Starting a Training Session".to_string(),
                    description: "Let's begin a training session. We'll select the Alphabet domain for this demo.".to_string(),
                    highlight_elements: vec!["start_button".to_string()],
                    action: Some(DemoAction::NavigateTo(Screen::DomainSelection)),
                    navigation: None,
                    wait_for_user: false,
                    auto_advance_ms: Some(2000),
                },
                DemoStep {
                    id: "select_domain".to_string(),
                    title: "Selecting Alphabet Domain".to_string(),
                    description: "We'll learn the alphabet sequence. Watch as we select this domain.".to_string(),
                    highlight_elements: vec!["domain_alphabet".to_string()],
                    action: Some(DemoAction::SelectDomain("Alphabet".to_string())),
                    navigation: None,
                    wait_for_user: false,
                    auto_advance_ms: Some(2000),
                },
                DemoStep {
                    id: "first_task".to_string(),
                    title: "Your First Task".to_string(),
                    description: "Here's a task! The system will present 'What comes after B?'".to_string(),
                    highlight_elements: vec!["task_display".to_string(), "answer_buttons".to_string()],
                    action: None,
                    navigation: Some(Screen::Training),
                    wait_for_user: false,
                    auto_advance_ms: Some(3000),
                },
                DemoStep {
                    id: "submit_answer".to_string(),
                    title: "Submitting an Answer".to_string(),
                    description: "Watch as we select the correct answer 'C'.".to_string(),
                    highlight_elements: vec!["answer_C".to_string()],
                    action: Some(DemoAction::SubmitAnswer("C".to_string())),
                    navigation: None,
                    wait_for_user: false,
                    auto_advance_ms: Some(2000),
                },
                DemoStep {
                    id: "feedback".to_string(),
                    title: "Instant Feedback".to_string(),
                    description: "Great! The system shows feedback immediately. Notice the performance metrics updating.".to_string(),
                    highlight_elements: vec!["feedback_display".to_string(), "metrics_display".to_string()],
                    action: None,
                    navigation: None,
                    wait_for_user: false,
                    auto_advance_ms: Some(3000),
                },
                DemoStep {
                    id: "hint_demo".to_string(),
                    title: "Using Hints".to_string(),
                    description: "For harder questions, you can request hints. Let's try it!".to_string(),
                    highlight_elements: vec!["hint_button".to_string()],
                    action: Some(DemoAction::RequestHint),
                    navigation: None,
                    wait_for_user: false,
                    auto_advance_ms: Some(2000),
                },
                DemoStep {
                    id: "mini_session".to_string(),
                    title: "Completing More Tasks".to_string(),
                    description: "The system will now complete several more tasks automatically...".to_string(),
                    highlight_elements: vec!["training_area".to_string()],
                    action: Some(DemoAction::RunMiniDemo),
                    navigation: None,
                    wait_for_user: false,
                    auto_advance_ms: Some(5000),
                },
                DemoStep {
                    id: "view_progress".to_string(),
                    title: "Viewing Your Progress".to_string(),
                    description: "Let's check the dashboard to see your learning progress!".to_string(),
                    highlight_elements: vec![],
                    action: Some(DemoAction::NavigateTo(Screen::Dashboard)),
                    navigation: None,
                    wait_for_user: false,
                    auto_advance_ms: Some(2000),
                },
                DemoStep {
                    id: "demo_complete".to_string(),
                    title: "Demo Complete!".to_string(),
                    description: "Excellent! You've seen how a full training session works. The dashboard shows your progress with charts and metrics.".to_string(),
                    highlight_elements: vec!["dashboard_metrics".to_string()],
                    action: None,
                    navigation: None,
                    wait_for_user: true,
                    auto_advance_ms: None,
                },
            ],
            completion_message: "Training demo complete! You've experienced the full learning cycle.".to_string(),
        };

        self.scenarios.insert(quick_tour.id.clone(), quick_tour);
        self.scenarios
            .insert(training_demo.id.clone(), training_demo);
    }

    pub fn start_scenario(&mut self, scenario_id: &str) -> Result<(), String> {
        if !self.scenarios.contains_key(scenario_id) {
            return Err(format!("Scenario '{}' not found", scenario_id));
        }

        self.current_scenario = Some(scenario_id.to_string());
        self.current_step_index = 0;
        self.is_active = true;
        self.is_paused = false;
        self.step_history.clear();
        self.highlights.clear();
        self.tooltips.clear();

        // Apply first step
        self.apply_current_step();
        Ok(())
    }

    pub fn next_step(&mut self) -> bool {
        if !self.is_active || self.is_paused {
            return false;
        }

        if let Some(scenario_id) = &self.current_scenario {
            if let Some(scenario) = self.scenarios.get(scenario_id) {
                if self.current_step_index < scenario.steps.len() - 1 {
                    // Record history
                    if let Some(current_step) = scenario.steps.get(self.current_step_index) {
                        self.step_history.push(current_step.id.clone());
                    }

                    self.current_step_index += 1;
                    self.apply_current_step();
                    return true;
                } else {
                    // Demo complete
                    self.complete_demo();
                }
            }
        }
        false
    }

    pub fn previous_step(&mut self) -> bool {
        if !self.is_active || self.current_step_index == 0 {
            return false;
        }

        self.current_step_index -= 1;
        self.apply_current_step();
        true
    }

    pub fn skip_to_step(&mut self, step_id: &str) -> bool {
        if let Some(scenario_id) = &self.current_scenario {
            if let Some(scenario) = self.scenarios.get(scenario_id) {
                if let Some(index) = scenario.steps.iter().position(|s| s.id == step_id) {
                    self.current_step_index = index;
                    self.apply_current_step();
                    return true;
                }
            }
        }
        false
    }

    fn apply_current_step(&mut self) {
        self.highlights.clear();
        self.tooltips.clear();

        if let Some(scenario_id) = &self.current_scenario {
            if let Some(scenario) = self.scenarios.get(scenario_id) {
                if let Some(step) = scenario.steps.get(self.current_step_index) {
                    // Apply highlights
                    for element_id in &step.highlight_elements {
                        self.highlights
                            .insert(element_id.clone(), HighlightStyle::default());
                    }

                    // Apply tooltips from action
                    if let Some(DemoAction::ShowTooltip { element, text }) = &step.action {
                        self.tooltips.insert(element.clone(), text.clone());
                    }
                }
            }
        }
    }

    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    pub fn resume(&mut self) {
        self.is_paused = false;
    }

    pub fn end_demo(&mut self) {
        self.is_active = false;
        self.is_paused = false;
        self.current_scenario = None;
        self.current_step_index = 0;
        self.highlights.clear();
        self.tooltips.clear();
    }

    fn complete_demo(&mut self) {
        // Mark demo as complete but keep highlights for a moment
        if let Some(scenario_id) = &self.current_scenario {
            if let Some(scenario) = self.scenarios.get(scenario_id) {
                // Could trigger completion callback here
                println!("Demo complete: {}", scenario.completion_message);
            }
        }
        // Clear after a delay (in real app, this would be async)
        self.is_active = false;
    }

    pub fn get_current_step(&self) -> Option<&DemoStep> {
        if let Some(scenario_id) = &self.current_scenario {
            if let Some(scenario) = self.scenarios.get(scenario_id) {
                return scenario.steps.get(self.current_step_index);
            }
        }
        None
    }

    pub fn get_progress(&self) -> (usize, usize) {
        if let Some(scenario_id) = &self.current_scenario {
            if let Some(scenario) = self.scenarios.get(scenario_id) {
                return (self.current_step_index + 1, scenario.steps.len());
            }
        }
        (0, 0)
    }

    pub fn should_highlight(&self, element_id: &str) -> Option<&HighlightStyle> {
        self.highlights.get(element_id)
    }

    pub fn get_tooltip(&self, element_id: &str) -> Option<&String> {
        self.tooltips.get(element_id)
    }

    /// Execute the action for the current step (if any)
    pub fn execute_current_action(&self, app_data: &mut AppData) {
        if let Some(step) = self.get_current_step() {
            if let Some(action) = &step.action {
                match action {
                    DemoAction::ClickButton(button_id) => {
                        println!("Demo: Clicking button {}", button_id);
                        // In real implementation, trigger the button's callback
                    }
                    DemoAction::EnterText { field, text } => {
                        println!("Demo: Entering '{}' into field {}", text, field);
                        // In real implementation, update the field's value
                    }
                    DemoAction::SelectDomain(domain) => {
                        println!("Demo: Selecting domain {}", domain);
                        // app_data.select_domain(domain);
                    }
                    DemoAction::SubmitAnswer(answer) => {
                        println!("Demo: Submitting answer {}", answer);
                        app_data.selected_answer = Some(answer.clone());
                        // Trigger async response submission
                        app_data.submit_response_in_flight = true;
                    }
                    DemoAction::RequestHint => {
                        println!("Demo: Requesting hint");
                        app_data.request_hint();
                    }
                    DemoAction::NavigateTo(screen) => {
                        println!("Demo: Navigating to {:?}", screen);
                        app_data.current_screen = screen.clone();
                    }
                    DemoAction::ShowTooltip { element, text } => {
                        println!("Demo: Showing tooltip '{}' for {}", text, element);
                        // Tooltip is already handled in apply_current_step
                    }
                    DemoAction::RunMiniDemo => {
                        println!("Demo: Running mini demo sequence");
                        // Run a quick automated sequence
                        for i in 0..3 {
                            app_data.generate_next_task();
                            app_data.selected_answer = Some(format!("Answer{}", i));
                            // Trigger async response submission
                            app_data.submit_response_in_flight = true;
                        }
                    }
                }
            }
        }
    }
}

impl Default for DemoController {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create a demo overlay view
pub fn demo_overlay_view(controller: &DemoController) -> String {
    if !controller.is_active {
        return String::new();
    }

    if let Some(step) = controller.get_current_step() {
        let (current, total) = controller.get_progress();
        format!(
            "📚 {} ({}/{})\n\n{}\n\n{}",
            step.title,
            current,
            total,
            step.description,
            if controller.is_paused {
                "⏸ Paused"
            } else {
                ""
            }
        )
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_controller_initialization() {
        let controller = DemoController::new();
        assert!(!controller.is_active);
        assert_eq!(controller.scenarios.len(), 2); // quick_tour and training_demo
        assert!(controller.scenarios.contains_key("quick_tour"));
        assert!(controller.scenarios.contains_key("training_demo"));
    }

    #[test]
    fn test_start_scenario() {
        let mut controller = DemoController::new();
        let result = controller.start_scenario("quick_tour");
        assert!(result.is_ok());
        assert!(controller.is_active);
        assert_eq!(controller.current_step_index, 0);
        assert_eq!(controller.current_scenario, Some("quick_tour".to_string()));
    }

    #[test]
    fn test_demo_navigation() {
        let mut controller = DemoController::new();
        controller.start_scenario("quick_tour").unwrap();

        // Test next step
        let advanced = controller.next_step();
        assert!(advanced);
        assert_eq!(controller.current_step_index, 1);

        // Test previous step
        let went_back = controller.previous_step();
        assert!(went_back);
        assert_eq!(controller.current_step_index, 0);

        // Test skip to step
        let skipped = controller.skip_to_step("dashboard_overview");
        assert!(skipped);
        let current_step = controller.get_current_step().unwrap();
        assert_eq!(current_step.id, "dashboard_overview");
    }

    #[test]
    fn test_highlights_and_tooltips() {
        let mut controller = DemoController::new();
        controller.start_scenario("quick_tour").unwrap();

        // Navigate to step with highlights
        controller.skip_to_step("navigation");

        // Check highlights are applied
        assert!(controller.should_highlight("nav_bar").is_some());

        // Check tooltip is present
        assert!(controller.get_tooltip("nav_bar").is_some());
    }

    #[test]
    fn test_demo_completion() {
        let mut controller = DemoController::new();
        controller.start_scenario("quick_tour").unwrap();

        // Navigate to last step
        while controller.next_step() {
            // Keep advancing
        }

        // Should no longer be active after completing all steps
        assert!(!controller.is_active);
    }
}
