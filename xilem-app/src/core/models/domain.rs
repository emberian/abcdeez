use abcdeez_core::{Task as CoreTask, TopologyType};
use serde::{Deserialize, Serialize};

// Domain types for UI
#[derive(Debug, Clone, PartialEq)]
pub enum Domain {
    Alphabet,
    DaysOfWeek,
    Music,
    Mathematics,
    Custom(String),
}

impl Domain {
    pub fn as_str(&self) -> &str {
        match self {
            Domain::Alphabet => "alphabet",
            Domain::DaysOfWeek => "days_of_week",
            Domain::Music => "music",
            Domain::Mathematics => "mathematics",
            Domain::Custom(s) => s,
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Domain::Alphabet => "Alphabet (A-Z)",
            Domain::DaysOfWeek => "Days of the Week",
            Domain::Music => "Music Theory",
            Domain::Mathematics => "Mathematics",
            Domain::Custom(s) => s,
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Domain::Alphabet => "Learn letter positions, sequences, and relationships",
            Domain::DaysOfWeek => "Master the order and relationships between days",
            Domain::Music => "Understand intervals, scales, and chord progressions",
            Domain::Mathematics => "Practice arithmetic operations and number patterns",
            Domain::Custom(_) => "Custom learning domain",
        }
    }

    pub fn to_topology_type(&self) -> TopologyType {
        match self {
            Domain::Alphabet => TopologyType::Linear,
            Domain::DaysOfWeek => TopologyType::Cyclic,
            Domain::Music => TopologyType::PartialOrder,
            Domain::Mathematics => TopologyType::GeneralGraph,
            Domain::Custom(_) => TopologyType::Linear,
        }
    }
}

// UI-specific task wrapper that includes the core task
#[derive(Debug, Clone)]
pub struct UITask {
    pub core_task: CoreTask,
    pub display_prompt: String,
    pub display_options: Vec<String>,
    pub hint: Option<String>,
    pub feedback_message: Option<String>,
}

impl UITask {
    pub fn from_core_task(task: CoreTask) -> Self {
        let (display_prompt, display_options) = Self::format_task_for_ui(&task);

        UITask {
            core_task: task,
            display_prompt,
            display_options,
            hint: None,
            feedback_message: None,
        }
    }

    fn format_task_for_ui(task: &CoreTask) -> (String, Vec<String>) {
        // Format the task prompt and options based on task type
        let prompt = task.prompt.clone();
        let options = task.options.clone();

        (prompt, options)
    }
}