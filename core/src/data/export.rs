use crate::core::learner::LearnerModel;
use crate::statistics::StrategyType;
use crate::tasks::types::{TaskResponse, TaskSession};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerDataExport {
    pub learner_id: String,
    pub export_timestamp: DateTime<Utc>,
    pub sessions: Vec<SessionData>,
    pub performance_trajectories: Vec<PerformancePoint>,
    pub error_patterns: ErrorAnalysis,
    pub model_parameters: ModelSnapshot,
    pub metadata: ExportMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub session_id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub topology_type: String,
    pub responses: Vec<TaskResponse>,
    pub summary: SessionSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub total_tasks: usize,
    pub correct_count: usize,
    pub accuracy: f64,
    pub mean_rt_ms: f64,
    pub median_rt_ms: f64,
    pub strategy_detected: Option<StrategyType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformancePoint {
    pub trial_number: usize,
    pub timestamp: DateTime<Utc>,
    pub accuracy: f64,
    pub mean_rt: f64,
    pub task_type: String,
    pub difficulty: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorAnalysis {
    pub total_errors: usize,
    pub error_rate: f64,
    pub common_confusions: Vec<(String, String, usize)>,
    pub error_by_task_type: HashMap<String, f64>,
    pub error_by_difficulty: Vec<(f64, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSnapshot {
    pub timestamp: DateTime<Utc>,
    pub node_embeddings: HashMap<String, NodeEmbeddingExport>,
    pub operation_proficiencies: HashMap<String, f64>,
    pub memory_strengths: HashMap<String, f64>,
    pub chunk_boundaries: Vec<ChunkBoundaryExport>,
    pub total_practice_time_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeEmbeddingExport {
    pub position: f64,
    pub uncertainty: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkBoundaryExport {
    pub position: usize,
    pub strength: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub export_version: String,
    pub software_version: String,
    pub platform: String,
    pub experiment_id: Option<String>,
    pub notes: Option<String>,
}

impl LearnerDataExport {
    pub fn from_learner_model(
        model: &LearnerModel,
        sessions: Vec<TaskSession>,
        experiment_id: Option<String>,
    ) -> Self {
        let learner_id = model.learner_id.clone();
        let export_timestamp = Utc::now();

        // Convert sessions
        let session_data: Vec<SessionData> = sessions
            .iter()
            .map(|session| {
                let responses = session.history.clone();
                let summary = Self::calculate_session_summary(&responses);

                SessionData {
                    session_id: format!("session_{}", uuid::Uuid::new_v4()),
                    start_time: responses
                        .first()
                        .map(|r| r.timestamp)
                        .unwrap_or_else(Utc::now),
                    end_time: responses.last().map(|r| r.timestamp),
                    topology_type: format!("{:?}", session.generator.topology.topology_type),
                    responses,
                    summary,
                }
            })
            .collect();

        // Calculate performance trajectories
        let mut performance_trajectories = Vec::new();
        let mut running_correct = 0;
        let mut running_total = 0;

        for session in &session_data {
            for (_i, response) in session.responses.iter().enumerate() {
                running_total += 1;
                if response.correct {
                    running_correct += 1;
                }

                performance_trajectories.push(PerformancePoint {
                    trial_number: running_total,
                    timestamp: response.timestamp,
                    accuracy: running_correct as f64 / running_total as f64,
                    mean_rt: response.response_time_ms as f64,
                    task_type: format!("{:?}", response.task.task_type),
                    difficulty: response.task.difficulty,
                });
            }
        }

        // Analyze errors
        let error_patterns = Self::analyze_errors(&session_data);

        // Create model snapshot
        let model_snapshot = ModelSnapshot {
            timestamp: export_timestamp,
            node_embeddings: model
                .node_embeddings
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        NodeEmbeddingExport {
                            position: v.position,
                            uncertainty: v.uncertainty,
                        },
                    )
                })
                .collect(),
            operation_proficiencies: model
                .operation_proficiencies
                .iter()
                .map(|(k, v)| (k.clone(), sigmoid(v.theta)))
                .collect(),
            memory_strengths: model
                .memory_strengths
                .iter()
                .map(|(k, v)| (k.clone(), v.strength))
                .collect(),
            chunk_boundaries: model
                .chunk_boundaries
                .iter()
                .map(|b| ChunkBoundaryExport {
                    position: b.position,
                    strength: b.strength,
                })
                .collect(),
            total_practice_time_seconds: model.total_practice_time.as_secs(),
        };

        let metadata = ExportMetadata {
            export_version: "1.0.0".to_string(),
            software_version: env!("CARGO_PKG_VERSION").to_string(),
            platform: std::env::consts::OS.to_string(),
            experiment_id,
            notes: None,
        };

        LearnerDataExport {
            learner_id,
            export_timestamp,
            sessions: session_data,
            performance_trajectories,
            error_patterns,
            model_parameters: model_snapshot,
            metadata,
        }
    }

    fn calculate_session_summary(responses: &[TaskResponse]) -> SessionSummary {
        let total_tasks = responses.len();
        let correct_count = responses.iter().filter(|r| r.correct).count();
        let accuracy = if total_tasks > 0 {
            correct_count as f64 / total_tasks as f64
        } else {
            0.0
        };

        let rts: Vec<f64> = responses
            .iter()
            .map(|r| r.response_time_ms as f64)
            .collect();

        let mean_rt_ms = if !rts.is_empty() {
            rts.iter().sum::<f64>() / rts.len() as f64
        } else {
            0.0
        };

        let median_rt_ms = if !rts.is_empty() {
            let mut sorted_rts = rts.clone();
            sorted_rts.sort_by(|a, b| a.partial_cmp(b).unwrap());
            sorted_rts[sorted_rts.len() / 2]
        } else {
            0.0
        };

        // Detect strategies based on response patterns
        let strategy_detected = Self::detect_strategy(responses);

        SessionSummary {
            total_tasks,
            correct_count,
            accuracy,
            mean_rt_ms,
            median_rt_ms,
            strategy_detected,
        }
    }

    fn detect_strategy(responses: &[TaskResponse]) -> Option<crate::statistics::StrategyType> {
        if responses.is_empty() {
            return None;
        }

        // Collect response times and distances for correlation analysis
        let mut rts = Vec::new();
        let mut distances = Vec::new();

        for response in responses {
            rts.push(response.response_time_ms as f64);

            // Extract distance information from task type
            let distance = match &response.task.task_type {
                crate::tasks::TaskType::KJump { k, .. } => *k as usize,
                crate::tasks::TaskType::Segment { count, .. } => *count,
                _ => 1, // Default distance for other tasks
            };
            distances.push(distance);
        }

        // Use the existing strategy analysis logic
        let correlation = if rts.len() >= 2 && distances.len() >= 2 {
            // Calculate correlation between RT and distance
            let n = rts.len() as f64;
            let sum_x: f64 = distances.iter().map(|&d| d as f64).sum();
            let sum_y: f64 = rts.iter().sum();
            let sum_xy: f64 = distances
                .iter()
                .zip(rts.iter())
                .map(|(d, rt)| *d as f64 * rt)
                .sum();
            let sum_x2: f64 = distances.iter().map(|&d| (d * d) as f64).sum();
            let sum_y2: f64 = rts.iter().map(|rt| rt * rt).sum();

            let numerator = n * sum_xy - sum_x * sum_y;
            let denominator = ((n * sum_x2 - sum_x * sum_x) * (n * sum_y2 - sum_y * sum_y)).sqrt();

            if denominator > 0.0 {
                numerator / denominator
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Classify strategy based on correlation
        // Using Cohen's effect size conventions:
        // r > 0.7: Strong correlation (r² > 0.49) - serial scanning
        // r < 0.3: Weak correlation (r² < 0.09) - direct access
        if correlation > 0.7 {
            Some(crate::statistics::StrategyType::SerialScan)
        } else if correlation < 0.3 {
            Some(crate::statistics::StrategyType::DirectIndex)
        } else {
            Some(crate::statistics::StrategyType::Hybrid)
        }
    }

    fn analyze_errors(sessions: &[SessionData]) -> ErrorAnalysis {
        let mut total_errors = 0;
        let mut total_tasks = 0;
        let mut confusion_counts: HashMap<(String, String), usize> = HashMap::new();
        let mut error_by_task_type: HashMap<String, (usize, usize)> = HashMap::new();
        let mut error_by_difficulty: HashMap<String, Vec<bool>> = HashMap::new();

        for session in sessions {
            for response in &session.responses {
                total_tasks += 1;

                let task_type = format!("{:?}", response.task.task_type);
                let difficulty_bucket = format!("{:.1}", response.task.difficulty);

                error_by_task_type
                    .entry(task_type.clone())
                    .or_insert((0, 0))
                    .1 += 1;

                error_by_difficulty
                    .entry(difficulty_bucket)
                    .or_insert_with(Vec::new)
                    .push(response.correct);

                if !response.correct {
                    total_errors += 1;

                    error_by_task_type.entry(task_type).or_insert((0, 0)).0 += 1;

                    let confusion = (
                        response.task.correct_answer.clone(),
                        response.user_answer.clone(),
                    );
                    *confusion_counts.entry(confusion).or_insert(0) += 1;
                }
            }
        }

        let error_rate = if total_tasks > 0 {
            total_errors as f64 / total_tasks as f64
        } else {
            0.0
        };

        let mut common_confusions: Vec<(String, String, usize)> = confusion_counts
            .into_iter()
            .map(|((expected, actual), count)| (expected, actual, count))
            .collect();
        common_confusions.sort_by_key(|c| std::cmp::Reverse(c.2));
        common_confusions.truncate(10);

        let error_by_task_type_rates = error_by_task_type
            .into_iter()
            .map(|(k, (errors, total))| {
                (
                    k,
                    if total > 0 {
                        errors as f64 / total as f64
                    } else {
                        0.0
                    },
                )
            })
            .collect();

        let error_by_difficulty_rates: Vec<(f64, f64)> = error_by_difficulty
            .into_iter()
            .map(|(bucket, results)| {
                let errors = results.iter().filter(|&&c| !c).count();
                let rate = if !results.is_empty() {
                    errors as f64 / results.len() as f64
                } else {
                    0.0
                };
                (bucket.parse::<f64>().unwrap_or(0.0), rate)
            })
            .collect();

        ErrorAnalysis {
            total_errors,
            error_rate,
            common_confusions,
            error_by_task_type: error_by_task_type_rates,
            error_by_difficulty: error_by_difficulty_rates,
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn to_csv(&self) -> String {
        let mut csv = String::new();

        // Enhanced header with all needed columns for R analysis
        csv.push_str("learner_id,session_id,trial_number,timestamp,group,phase,task_type,task_json,correct,rt_ms,user_answer,correct_answer,difficulty\n");

        // Data rows with complete information
        let mut global_trial = 0;
        for session in &self.sessions {
            for response in session.responses.iter() {
                global_trial += 1;

                // Serialize task details as JSON for flexible parsing in R
                let task_json = serde_json::to_string(&response.task.task_type)
                    .unwrap_or_else(|_| "{}".to_string());

                // Determine experimental phase based on trial number
                let phase = if global_trial <= 500 {
                    "training"
                } else if global_trial <= 600 {
                    "test"
                } else {
                    "transfer"
                };

                // Infer group from metadata (would be set during experiment)
                let group = self
                    .metadata
                    .experiment_id
                    .as_ref()
                    .map(|id| {
                        if id.contains("adaptive") {
                            "Adaptive"
                        } else if id.contains("linear") {
                            "Linear"
                        } else {
                            "Yoked"
                        }
                    })
                    .unwrap_or("Unknown");

                csv.push_str(&format!(
                    "{},{},{},{},{},{},{:?},\"{}\",{},{},{},{},{}\n",
                    self.learner_id,
                    session.session_id,
                    global_trial,
                    response.timestamp.to_rfc3339(),
                    group,
                    phase,
                    response.task.task_type,
                    task_json.replace("\"", "\"\""), // Escape quotes for CSV
                    response.correct,
                    response.response_time_ms,
                    response.user_answer,
                    response.task.correct_answer,
                    response.task.difficulty
                ));
            }
        }

        csv
    }

    pub fn save_to_file(&self, path: &Path) -> std::io::Result<()> {
        let json = self
            .to_json()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;

        Ok(())
    }

    pub fn save_csv(&self, path: &Path) -> std::io::Result<()> {
        let csv = self.to_csv();
        let mut file = File::create(path)?;
        file.write_all(csv.as_bytes())?;
        Ok(())
    }

    /// Enhanced export functionality for R/Python/SPSS integration
    pub fn export_for_r(&self, base_path: &Path, include_script: bool) -> std::io::Result<()> {
        // Export CSV data
        let csv_path = base_path.with_extension("csv");
        self.save_csv(&csv_path)?;

        if include_script {
            // Generate R analysis script
            let r_script = self.generate_r_script()?;
            let script_path = base_path.with_extension("R");
            let mut file = File::create(&script_path)?;
            file.write_all(r_script.as_bytes())?;

            // Generate codebook
            let codebook = self.generate_codebook()?;
            let codebook_path = base_path.with_file_name(format!(
                "{}_codebook.txt",
                base_path.file_stem().unwrap().to_string_lossy()
            ));
            let mut file = File::create(&codebook_path)?;
            file.write_all(codebook.as_bytes())?;
        }

        Ok(())
    }

    pub fn export_for_python(&self, base_path: &Path, notebook: bool) -> std::io::Result<()> {
        // Export CSV data
        let csv_path = base_path.with_extension("csv");
        self.save_csv(&csv_path)?;

        if notebook {
            // Generate Jupyter notebook
            let notebook_content = self.generate_jupyter_notebook()?;
            let notebook_path = base_path.with_extension("ipynb");
            let mut file = File::create(&notebook_path)?;
            file.write_all(notebook_content.as_bytes())?;
        } else {
            // Generate Python script
            let python_script = self.generate_python_script()?;
            let script_path = base_path.with_extension("py");
            let mut file = File::create(&script_path)?;
            file.write_all(python_script.as_bytes())?;
        }

        Ok(())
    }

    pub fn export_for_spss(&self, base_path: &Path) -> std::io::Result<()> {
        // Export CSV data
        let csv_path = base_path.with_extension("csv");
        self.save_csv(&csv_path)?;

        // Generate SPSS syntax
        let spss_syntax = self.generate_spss_syntax()?;
        let syntax_path = base_path.with_extension("sps");
        let mut file = File::create(&syntax_path)?;
        file.write_all(spss_syntax.as_bytes())?;

        Ok(())
    }

    fn generate_r_script(&self) -> std::io::Result<String> {
        let script = format!(
            r#"
# R Analysis Script for Learning Data
# Generated by Alphabet Terminal Prototype
# Participant: {}

library(tidyverse)
library(lme4)
library(lmerTest)
library(psych)

# Load data
data <- read.csv("{}.csv", stringsAsFactors = TRUE)

# Basic descriptive statistics
cat("=== DESCRIPTIVE STATISTICS ===\n")
summary(data)
describe(data[sapply(data, is.numeric)])

# Learning curve analysis
if(require(ggplot2)) {{
    # Plot accuracy over trials
    p1 <- ggplot(data, aes(x = trial_number, y = as.numeric(correct))) +
        geom_smooth(method = "loess", span = 0.3) +
        geom_point(alpha = 0.3) +
        labs(title = "Learning Curve",
             x = "Trial Number",
             y = "Accuracy") +
        theme_minimal()
    
    print(p1)
    
    # Plot response time over trials
    p2 <- ggplot(data, aes(x = trial_number, y = rt_ms)) +
        geom_smooth(method = "loess", span = 0.3) +
        geom_point(alpha = 0.3) +
        labs(title = "Response Time Over Trials",
             x = "Trial Number", 
             y = "Response Time (ms)") +
        theme_minimal()
    
    print(p2)
    
    # RT by accuracy
    p3 <- ggplot(data, aes(x = factor(correct), y = rt_ms)) +
        geom_boxplot() +
        labs(title = "Response Time by Accuracy",
             x = "Correct", 
             y = "Response Time (ms)") +
        theme_minimal()
    
    print(p3)
}}

# Mixed-effects model for learning
if("phase" %in% colnames(data)) {{
    cat("\n=== MIXED-EFFECTS ANALYSIS ===\n")
    
    # Learning model
    model_learning <- glmer(correct ~ trial_number + (1|session_id), 
                           data = data, family = binomial)
    summary(model_learning)
    
    # Phase comparison model
    model_phase <- glmer(correct ~ phase + (1|session_id), 
                        data = data, family = binomial)
    summary(model_phase)
    
    # ANOVA comparison
    anova(model_learning, model_phase)
}}

# Error analysis
cat("\n=== ERROR ANALYSIS ===\n")
error_rate <- 1 - mean(as.numeric(data$correct), na.rm = TRUE)
cat("Overall error rate:", round(error_rate * 100, 2), "%\n")

# Task type analysis
if("task_type" %in% colnames(data)) {{
    task_performance <- data %>%
        group_by(task_type) %>%
        summarise(
            n = n(),
            accuracy = mean(as.numeric(correct), na.rm = TRUE),
            mean_rt = mean(rt_ms, na.rm = TRUE),
            .groups = "drop"
        )
    
    print(task_performance)
}}

cat("\nAnalysis complete. Check plots and results above.\n")
"#,
            self.learner_id, self.learner_id
        );

        Ok(script)
    }

    fn generate_python_script(&self) -> std::io::Result<String> {
        let script = format!(
            r#"
"""
Python Analysis Script for Learning Data
Generated by Alphabet Terminal Prototype
Participant: {}
"""

import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import seaborn as sns
from scipy import stats
import warnings
warnings.filterwarnings('ignore')

# Load data
data = pd.read_csv('{}.csv')
print("Data loaded successfully!")
print(f"Shape: {{data.shape}}")

# Basic information
print("\n=== DATA OVERVIEW ===")
print(data.info())
print("\n=== DESCRIPTIVE STATISTICS ===")
print(data.describe())

# Convert correct to numeric for analysis
data['accuracy'] = data['correct'].astype(int)

# Learning curve analysis
print("\n=== LEARNING CURVE ANALYSIS ===")
accuracy_by_trial = data.groupby('trial_number')['accuracy'].mean()

plt.figure(figsize=(15, 5))

# Plot 1: Learning curve
plt.subplot(1, 3, 1)
plt.plot(accuracy_by_trial.index, accuracy_by_trial.values)
plt.xlabel('Trial Number')
plt.ylabel('Accuracy')
plt.title('Learning Curve')
plt.grid(True, alpha=0.3)

# Plot 2: Response time distribution
plt.subplot(1, 3, 2)
plt.hist(data['rt_ms'], bins=50, alpha=0.7)
plt.xlabel('Response Time (ms)')
plt.ylabel('Frequency')
plt.title('Response Time Distribution')

# Plot 3: RT by accuracy
plt.subplot(1, 3, 3)
correct_rt = data[data['correct'] == True]['rt_ms']
incorrect_rt = data[data['correct'] == False]['rt_ms']

plt.boxplot([correct_rt, incorrect_rt], labels=['Correct', 'Incorrect'])
plt.ylabel('Response Time (ms)')
plt.title('RT by Accuracy')

plt.tight_layout()
plt.show()

# Statistical tests
print("\n=== STATISTICAL ANALYSIS ===")
overall_accuracy = data['accuracy'].mean()
print(f"Overall accuracy: {{overall_accuracy:.3f}} ({{overall_accuracy*100:.1f}}%)")

# Learning effect (correlation between trial and accuracy)
learning_corr, learning_p = stats.pearsonr(data['trial_number'], data['accuracy'])
print(f"Learning correlation (trial vs accuracy): r = {{learning_corr:.3f}}, p = {{learning_p:.3f}}")

# Speed-accuracy tradeoff
rt_acc_corr, rt_acc_p = stats.pearsonr(data['rt_ms'], data['accuracy'])
print(f"Speed-accuracy correlation: r = {{rt_acc_corr:.3f}}, p = {{rt_acc_p:.3f}}")

# Task type analysis (if available)
if 'task_type' in data.columns:
    print("\n=== TASK TYPE ANALYSIS ===")
    task_stats = data.groupby('task_type').agg({{
        'accuracy': ['count', 'mean', 'std'],
        'rt_ms': ['mean', 'std']
    }}).round(3)
    print(task_stats)

# Phase analysis (if available)
if 'phase' in data.columns:
    print("\n=== PHASE ANALYSIS ===")
    phase_stats = data.groupby('phase').agg({{
        'accuracy': ['count', 'mean', 'std'],
        'rt_ms': ['mean', 'std']
    }}).round(3)
    print(phase_stats)
    
    # ANOVA for phase differences
    from scipy.stats import f_oneway
    phases = data['phase'].unique()
    phase_accuracies = [data[data['phase'] == phase]['accuracy'] for phase in phases]
    f_stat, f_p = f_oneway(*phase_accuracies)
    print(f"\nPhase ANOVA: F = {{f_stat:.3f}}, p = {{f_p:.3f}}")

print("\nAnalysis complete!")
"#,
            self.learner_id, self.learner_id
        );

        Ok(script)
    }

    fn generate_jupyter_notebook(&self) -> std::io::Result<String> {
        let notebook = serde_json::json!({
            "cells": [
                {
                    "cell_type": "markdown",
                    "metadata": {},
                    "source": [
                        "# Learning Data Analysis\n",
                        format!("**Participant:** {}\n", self.learner_id),
                        format!("**Generated:** {}\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")),
                        format!("**Sessions:** {}\n", self.sessions.len()),
                        format!("**Total Trials:** {}", self.performance_trajectories.len())
                    ]
                },
                {
                    "cell_type": "code",
                    "execution_count": null,
                    "metadata": {},
                    "outputs": [],
                    "source": [
                        "import pandas as pd\n",
                        "import numpy as np\n",
                        "import matplotlib.pyplot as plt\n",
                        "import seaborn as sns\n",
                        "from scipy import stats\n",
                        "\n",
                        format!("# Load data\ndata = pd.read_csv('{}.csv')", self.learner_id),
                        "\nprint(f'Data shape: {{data.shape}}')\ndata.head()"
                    ]
                },
                {
                    "cell_type": "markdown",
                    "metadata": {},
                    "source": ["## Learning Curve Analysis"]
                },
                {
                    "cell_type": "code",
                    "execution_count": null,
                    "metadata": {},
                    "outputs": [],
                    "source": [
                        "# Learning curve\n",
                        "data['accuracy'] = data['correct'].astype(int)\n",
                        "accuracy_by_trial = data.groupby('trial_number')['accuracy'].mean()\n",
                        "\n",
                        "plt.figure(figsize=(10, 6))\n",
                        "plt.plot(accuracy_by_trial.index, accuracy_by_trial.values)\n",
                        "plt.xlabel('Trial Number')\n",
                        "plt.ylabel('Accuracy')\n",
                        "plt.title('Learning Curve')\n",
                        "plt.grid(True, alpha=0.3)\n",
                        "plt.show()\n",
                        "\n",
                        "print(f'Overall accuracy: {{data[\"accuracy\"].mean():.3f}}')"
                    ]
                },
                {
                    "cell_type": "markdown",
                    "metadata": {},
                    "source": ["## Response Time Analysis"]
                },
                {
                    "cell_type": "code",
                    "execution_count": null,
                    "metadata": {},
                    "outputs": [],
                    "source": [
                        "# Response time analysis\n",
                        "plt.figure(figsize=(12, 4))\n",
                        "\n",
                        "plt.subplot(1, 2, 1)\n",
                        "plt.hist(data['rt_ms'], bins=50, alpha=0.7)\n",
                        "plt.xlabel('Response Time (ms)')\n",
                        "plt.ylabel('Frequency')\n",
                        "plt.title('RT Distribution')\n",
                        "\n",
                        "plt.subplot(1, 2, 2)\n",
                        "sns.boxplot(x='correct', y='rt_ms', data=data)\n",
                        "plt.title('RT by Accuracy')\n",
                        "\n",
                        "plt.tight_layout()\n",
                        "plt.show()"
                    ]
                }
            ],
            "metadata": {
                "kernelspec": {
                    "display_name": "Python 3",
                    "language": "python",
                    "name": "python3"
                },
                "language_info": {
                    "name": "python",
                    "version": "3.8.0"
                }
            },
            "nbformat": 4,
            "nbformat_minor": 4
        });

        serde_json::to_string_pretty(&notebook)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    fn generate_spss_syntax(&self) -> std::io::Result<String> {
        let syntax = format!(
            r#"
* SPSS Syntax File for Learning Data
* Generated by Alphabet Terminal Prototype
* Participant: {}

* Import CSV data
GET DATA /TYPE=TXT
  /FILE='{}.csv'
  /ARRANGEMENT=DELIMITED
  /DELCASE=LINE
  /FIRSTCASE=2
  /DELIMITERS=","
  /QUALIFIER='"'
  /VARIABLES=
    learner_id A50
    session_id A50
    trial_number F8.0
    timestamp A30
    group A20
    phase A20
    task_type A50
    task_json A255
    correct F1.0
    rt_ms F8.1
    user_answer A10
    correct_answer A10
    difficulty F8.3.

* Variable labels
VARIABLE LABELS
    learner_id 'Participant ID'
    session_id 'Session identifier'
    trial_number 'Trial number'
    timestamp 'Response timestamp'
    group 'Experimental group'
    phase 'Experimental phase'
    task_type 'Type of task'
    task_json 'Task details (JSON)'
    correct 'Response accuracy'
    rt_ms 'Response time in milliseconds'
    user_answer 'Participant response'
    correct_answer 'Correct response'
    difficulty 'Task difficulty'.

* Value labels
VALUE LABELS
    correct 0 'Incorrect' 1 'Correct'
    /phase 'training' 'Training Phase' 'test' 'Test Phase' 'transfer' 'Transfer Phase'.

* Basic descriptive statistics
DESCRIPTIVES VARIABLES=trial_number rt_ms difficulty
  /STATISTICS=MEAN STDDEV MIN MAX.

* Frequency tables
FREQUENCIES VARIABLES=correct group phase task_type.

* Learning curve analysis
* Bin trials into groups for analysis
COMPUTE trial_bin = TRUNC(trial_number / 50) + 1.
VARIABLE LABELS trial_bin 'Trial bin (50-trial groups)'.

* Mean accuracy by trial bin
MEANS TABLES=correct BY trial_bin
  /CELLS MEAN COUNT STDDEV.

* Response time analysis
MEANS TABLES=rt_ms BY correct
  /CELLS MEAN COUNT STDDEV
  /STATISTICS ANOVA.

* Task type analysis
MEANS TABLES=correct rt_ms BY task_type
  /CELLS MEAN COUNT STDDEV.

* Phase analysis (if applicable)
MEANS TABLES=correct rt_ms BY phase
  /CELLS MEAN COUNT STDDEV
  /STATISTICS ANOVA.

* Correlations
CORRELATIONS
  /VARIABLES=trial_number correct rt_ms difficulty
  /PRINT=TWOTAIL NOSIG
  /MISSING=PAIRWISE.

* Learning curve visualization
GRAPH
  /SCATTERPLOT(BIVAR)=trial_number WITH correct
  /MISSING=LISTWISE
  /TITLE='Learning Curve'.

* Response time by accuracy
GRAPH
  /BOXPLOT(GROUPED)=rt_ms BY correct
  /MISSING=LISTWISE
  /TITLE='Response Time by Accuracy'.

SAVE OUTFILE='{}_analyzed.sav'.
"#,
            self.learner_id, self.learner_id, self.learner_id
        );

        Ok(syntax)
    }

    fn generate_codebook(&self) -> std::io::Result<String> {
        let codebook = format!(
            r#"
CODEBOOK FOR LEARNING DATA
===========================

Dataset: Learning experiment data for participant {}
Generated: {}
Total trials: {}
Sessions: {}

VARIABLE DESCRIPTIONS
====================

learner_id: Unique participant identifier
    Type: String
    Example: "{}"

session_id: Session identifier within participant
    Type: String
    Description: Links trials to specific testing sessions

trial_number: Sequential trial number across all sessions
    Type: Integer
    Range: 1 to {}
    Description: Global trial counter across entire experiment

timestamp: Response timestamp
    Type: DateTime (ISO 8601 format)
    Description: When the response was recorded

group: Experimental condition
    Type: Categorical
    Values: Adaptive, Linear, Yoked, Unknown
    Description: Assigned experimental condition

phase: Experimental phase
    Type: Categorical
    Values: training, test, transfer
    Description: Stage of experiment when trial occurred

task_type: Type of task presented
    Type: Categorical
    Description: Specific task variant (see task_json for details)

task_json: Detailed task parameters
    Type: JSON string
    Description: Complete task specification in JSON format

correct: Response accuracy
    Type: Binary
    Values: 0 (Incorrect), 1 (Correct)
    Description: Whether the response was correct

rt_ms: Response time
    Type: Numeric
    Units: Milliseconds
    Description: Time from stimulus presentation to response

user_answer: Participant response
    Type: String
    Description: The actual response provided by participant

correct_answer: Correct response
    Type: String
    Description: The correct response for this trial

difficulty: Task difficulty
    Type: Numeric
    Range: 0.0 to 1.0
    Description: Estimated difficulty of the task

PERFORMANCE SUMMARY
==================

Overall accuracy: {:.3} ({:.1}%)
Total errors: {}
Average response time: {:.1} ms
Completed sessions: {}

ERROR ANALYSIS
=============

{}

NOTES
=====

- All times are in UTC
- Missing values coded as "NA" 
- Task details in task_json can be parsed for fine-grained analysis
- Response times include stimulus processing and motor execution time
"#,
            self.learner_id,
            self.export_timestamp.format("%Y-%m-%d %H:%M:%S"),
            self.performance_trajectories.len(),
            self.sessions.len(),
            self.learner_id,
            self.performance_trajectories.len(),
            self.error_patterns.error_rate,
            self.error_patterns.error_rate * 100.0,
            self.error_patterns.total_errors,
            self.sessions
                .iter()
                .map(|s| s.summary.mean_rt_ms)
                .sum::<f64>()
                / self.sessions.len() as f64,
            self.sessions.len(),
            self.format_error_summary()
        );

        Ok(codebook)
    }

    fn format_error_summary(&self) -> String {
        let mut summary = String::new();

        summary.push_str(&format!(
            "Total errors: {}\n",
            self.error_patterns.total_errors
        ));
        summary.push_str(&format!(
            "Error rate: {:.1}%\n\n",
            self.error_patterns.error_rate * 100.0
        ));

        summary.push_str("Most common confusions:\n");
        for (i, (expected, actual, count)) in self
            .error_patterns
            .common_confusions
            .iter()
            .enumerate()
            .take(5)
        {
            summary.push_str(&format!(
                "{}. Expected '{}' → Got '{}' ({} times)\n",
                i + 1,
                expected,
                actual,
                count
            ));
        }

        if !self.error_patterns.error_by_task_type.is_empty() {
            summary.push_str("\nError rates by task type:\n");
            for (task_type, error_rate) in &self.error_patterns.error_by_task_type {
                summary.push_str(&format!("- {}: {:.1}%\n", task_type, error_rate * 100.0));
            }
        }

        summary
    }

    pub fn export_all_formats(&self, base_path: &Path) -> std::io::Result<()> {
        // Create directory if it doesn't exist
        std::fs::create_dir_all(base_path)?;

        // Export JSON format
        let json_path = base_path.join(format!("{}_full.json", self.learner_id));
        self.save_to_file(&json_path)?;

        // Export CSV format
        let csv_path = base_path.join(format!("{}_responses.csv", self.learner_id));
        self.save_csv(&csv_path)?;

        // Export session summaries CSV
        let sessions_path = base_path.join(format!("{}_sessions.csv", self.learner_id));
        self.save_sessions_csv(&sessions_path)?;

        // Export model parameters CSV
        let params_path = base_path.join(format!("{}_parameters.csv", self.learner_id));
        self.save_parameters_csv(&params_path)?;

        Ok(())
    }

    fn save_sessions_csv(&self, path: &Path) -> std::io::Result<()> {
        let mut csv = String::new();
        csv.push_str("learner_id,session_id,session_number,start_time,end_time,duration_minutes,n_trials,accuracy,mean_rt_ms,median_rt_ms,strategy_detected\n");

        for (i, session) in self.sessions.iter().enumerate() {
            let duration_minutes = session
                .end_time
                .map(|end| (end - session.start_time).num_minutes() as f64)
                .unwrap_or(0.0);

            csv.push_str(&format!(
                "{},{},{},{},{},{:.1},{},{:.3},{:.1},{:.1},{:?}\n",
                self.learner_id,
                session.session_id,
                i + 1,
                session.start_time.to_rfc3339(),
                session
                    .end_time
                    .map(|t| t.to_rfc3339())
                    .unwrap_or_else(|| "NA".to_string()),
                duration_minutes,
                session.summary.total_tasks,
                session.summary.accuracy,
                session.summary.mean_rt_ms,
                session.summary.median_rt_ms,
                session
                    .summary
                    .strategy_detected
                    .as_ref()
                    .map(|s| format!("{:?}", s))
                    .unwrap_or_else(|| "NA".to_string())
            ));
        }

        let mut file = File::create(path)?;
        file.write_all(csv.as_bytes())?;
        Ok(())
    }

    fn save_parameters_csv(&self, path: &Path) -> std::io::Result<()> {
        let mut csv = String::new();
        csv.push_str(
            "learner_id,timestamp,trial_number,parameter_type,parameter_name,value,uncertainty\n",
        );

        // Export node embeddings
        for (node, embedding) in &self.model_parameters.node_embeddings {
            csv.push_str(&format!(
                "{},{},{},node_embedding,{},{:.6},{:.6}\n",
                self.learner_id,
                self.model_parameters.timestamp.to_rfc3339(),
                self.performance_trajectories.len(),
                node,
                embedding.position,
                embedding.uncertainty
            ));
        }

        // Export operation proficiencies
        for (op, proficiency) in &self.model_parameters.operation_proficiencies {
            csv.push_str(&format!(
                "{},{},{},operation_proficiency,{},{:.6},NA\n",
                self.learner_id,
                self.model_parameters.timestamp.to_rfc3339(),
                self.performance_trajectories.len(),
                op,
                proficiency
            ));
        }

        // Export memory strengths
        for (item, strength) in &self.model_parameters.memory_strengths {
            csv.push_str(&format!(
                "{},{},{},memory_strength,{},{:.6},NA\n",
                self.learner_id,
                self.model_parameters.timestamp.to_rfc3339(),
                self.performance_trajectories.len(),
                item,
                strength
            ));
        }

        // Export chunk boundaries
        for boundary in &self.model_parameters.chunk_boundaries {
            csv.push_str(&format!(
                "{},{},{},chunk_boundary,position_{},{:.6},NA\n",
                self.learner_id,
                self.model_parameters.timestamp.to_rfc3339(),
                self.performance_trajectories.len(),
                boundary.position,
                boundary.strength
            ));
        }

        let mut file = File::create(path)?;
        file.write_all(csv.as_bytes())?;
        Ok(())
    }
}

// Population analyzer for collaborative analysis
#[derive(Debug, Clone)]
pub struct PopulationAnalyzer {
    pub learners: Vec<LearnerDataExport>,
}

impl PopulationAnalyzer {
    pub fn new(learners: Vec<LearnerDataExport>) -> Self {
        PopulationAnalyzer { learners }
    }

    pub fn export_population_data(&self, base_path: &Path) -> std::io::Result<()> {
        // Create output directory structure
        std::fs::create_dir_all(base_path)?;

        // Combine all responses into single CSV
        self.export_combined_responses(&base_path.join("responses.csv"))?;

        // Export participant metadata
        self.export_participants(&base_path.join("participants.csv"))?;

        // Export all sessions
        self.export_all_sessions(&base_path.join("sessions.csv"))?;

        // Export all model parameters
        self.export_all_parameters(&base_path.join("model_parameters.csv"))?;

        Ok(())
    }

    /// Enhanced population export for R/Python/SPSS with analysis scripts
    pub fn export_for_r_analysis(&self, base_path: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(base_path)?;

        // Export data files
        self.export_population_data(base_path)?;

        // Generate comprehensive R analysis script
        let r_script = self.generate_population_r_script()?;
        let script_path = base_path.join("population_analysis.R");
        let mut file = File::create(&script_path)?;
        file.write_all(r_script.as_bytes())?;

        // Generate population codebook
        let codebook = self.generate_population_codebook()?;
        let codebook_path = base_path.join("population_codebook.txt");
        let mut file = File::create(&codebook_path)?;
        file.write_all(codebook.as_bytes())?;

        Ok(())
    }

    pub fn export_for_python_analysis(
        &self,
        base_path: &Path,
        notebook: bool,
    ) -> std::io::Result<()> {
        std::fs::create_dir_all(base_path)?;

        // Export data files
        self.export_population_data(base_path)?;

        if notebook {
            // Generate comprehensive Jupyter notebook
            let notebook_content = self.generate_population_notebook()?;
            let notebook_path = base_path.join("population_analysis.ipynb");
            let mut file = File::create(&notebook_path)?;
            file.write_all(notebook_content.as_bytes())?;
        } else {
            // Generate Python analysis script
            let python_script = self.generate_population_python_script()?;
            let script_path = base_path.join("population_analysis.py");
            let mut file = File::create(&script_path)?;
            file.write_all(python_script.as_bytes())?;
        }

        Ok(())
    }

    fn generate_population_r_script(&self) -> std::io::Result<String> {
        let script = format!(
            r#"
# Population Analysis Script for Learning Experiment
# Generated by Alphabet Terminal Prototype
# Dataset: {} participants

library(tidyverse)
library(lme4)
library(lmerTest)
library(psych)
library(effsize)
library(car)

# Load data
responses <- read.csv("responses.csv", stringsAsFactors = TRUE)
participants <- read.csv("participants.csv", stringsAsFactors = TRUE)
sessions <- read.csv("sessions.csv", stringsAsFactors = TRUE)

cat("=== DATASET OVERVIEW ===\n")
cat("Participants:", nrow(participants), "\n")
cat("Total responses:", nrow(responses), "\n")
cat("Sessions:", nrow(sessions), "\n")

# Data preprocessing
responses$correct_numeric <- as.numeric(responses$correct)
responses$learner_id <- factor(responses$learner_id)
responses$group <- factor(responses$group)
if("phase" %in% colnames(responses)) {{
    responses$phase <- factor(responses$phase, levels = c("training", "test", "transfer"))
}}

# Basic descriptive statistics by group
cat("\n=== GROUP COMPARISONS ===\n")
group_stats <- participants %>%
    group_by(group) %>%
    summarise(
        n = n(),
        mean_accuracy = mean(overall_accuracy, na.rm = TRUE),
        sd_accuracy = sd(overall_accuracy, na.rm = TRUE),
        mean_rt = mean(mean_rt, na.rm = TRUE),
        sd_rt = sd(mean_rt, na.rm = TRUE),
        .groups = "drop"
    )

print(group_stats)

# Statistical tests for group differences
if(length(unique(participants$group)) > 1) {{
    # Accuracy comparison
    accuracy_test <- aov(overall_accuracy ~ group, data = participants)
    cat("\nAccuracy ANOVA:\n")
    print(summary(accuracy_test))
    
    # Effect size
    accuracy_eta <- etaSquared(accuracy_test)
    cat("Effect size (eta-squared):", accuracy_eta$eta.sq, "\n")
    
    # Post-hoc tests if significant
    if(summary(accuracy_test)[[1]]$`Pr(>F)`[1] < 0.05) {{
        posthoc <- TukeyHSD(accuracy_test)
        print(posthoc)
    }}
    
    # RT comparison
    rt_test <- aov(mean_rt ~ group, data = participants)
    cat("\nResponse Time ANOVA:\n")
    print(summary(rt_test))
}}

# Learning curve analysis
cat("\n=== LEARNING CURVES ===\n")
learning_data <- responses %>%
    group_by(group, trial_number) %>%
    summarise(
        accuracy = mean(correct_numeric, na.rm = TRUE),
        rt = mean(rt_ms, na.rm = TRUE),
        n = n(),
        .groups = "drop"
    ) %>%
    filter(n >= 3)  # Only include trial numbers with sufficient data

# Plot learning curves
if(require(ggplot2)) {{
    p1 <- ggplot(learning_data, aes(x = trial_number, y = accuracy, color = group)) +
        geom_smooth(method = "loess", span = 0.3) +
        labs(title = "Learning Curves by Group",
             x = "Trial Number",
             y = "Accuracy") +
        theme_minimal() +
        theme(legend.position = "bottom")
    
    print(p1)
    
    p2 <- ggplot(learning_data, aes(x = trial_number, y = rt, color = group)) +
        geom_smooth(method = "loess", span = 0.3) +
        labs(title = "Response Time Over Trials by Group",
             x = "Trial Number",
             y = "Response Time (ms)") +
        theme_minimal() +
        theme(legend.position = "bottom")
    
    print(p2)
}}

# Mixed-effects models
cat("\n=== MIXED-EFFECTS MODELING ===\n")

# Learning model
model_learning <- glmer(correct ~ trial_number * group + (1 + trial_number | learner_id),
                       data = responses, family = binomial,
                       control = glmerControl(optimizer = "bobyqa"))

cat("Learning Model (Trial × Group interaction):\n")
print(summary(model_learning))

# Phase model (if applicable)
if("phase" %in% colnames(responses)) {{
    model_phase <- glmer(correct ~ phase * group + (1 | learner_id),
                        data = responses, family = binomial,
                        control = glmerControl(optimizer = "bobyqa"))
    
    cat("\nPhase Model (Phase × Group interaction):\n")
    print(summary(model_phase))
    
    # Model comparison
    cat("\nModel comparison:\n")
    print(anova(model_learning, model_phase))
}}

# Transfer analysis (if transfer phase exists)
if("phase" %in% colnames(responses) && "transfer" %in% responses$phase) {{
    cat("\n=== TRANSFER ANALYSIS ===\n")
    
    transfer_data <- responses %>%
        filter(phase == "transfer") %>%
        group_by(learner_id, group) %>%
        summarise(transfer_accuracy = mean(correct_numeric, na.rm = TRUE), .groups = "drop")
    
    transfer_test <- aov(transfer_accuracy ~ group, data = transfer_data)
    cat("Transfer Performance ANOVA:\n")
    print(summary(transfer_test))
    
    # Transfer vs training comparison
    training_transfer <- responses %>%
        filter(phase %in% c("training", "transfer")) %>%
        group_by(learner_id, group, phase) %>%
        summarise(accuracy = mean(correct_numeric, na.rm = TRUE), .groups = "drop") %>%
        pivot_wider(names_from = phase, values_from = accuracy) %>%
        mutate(transfer_gain = transfer - training)
    
    gain_test <- aov(transfer_gain ~ group, data = training_transfer)
    cat("\nTransfer Gain ANOVA:\n")
    print(summary(gain_test))
}}

# Individual differences analysis
cat("\n=== INDIVIDUAL DIFFERENCES ===\n")

# Learning rate estimation
learning_rates <- responses %>%
    group_by(learner_id, group) %>%
    do(model = glm(correct_numeric ~ trial_number, family = binomial, data = .)) %>%
    mutate(learning_rate = map_dbl(model, ~ coef(.)["trial_number"]))

learning_rate_test <- aov(learning_rate ~ group, data = learning_rates)
cat("Learning Rate Differences:\n")
print(summary(learning_rate_test))

# Save results
save.image("population_analysis_results.RData")
cat("\nAnalysis complete! Results saved to population_analysis_results.RData\n")
"#,
            self.learners.len()
        );

        Ok(script)
    }

    fn generate_population_python_script(&self) -> std::io::Result<String> {
        let script = format!(
            r#"
"""
Population Analysis Script for Learning Experiment
Generated by Alphabet Terminal Prototype
Dataset: {} participants
"""

import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import seaborn as sns
from scipy import stats
from scipy.stats import f_oneway
import statsmodels.api as sm
import statsmodels.formula.api as smf
import warnings
warnings.filterwarnings('ignore')

# Set style
plt.style.use('seaborn-v0_8')
sns.set_palette("husl")

# Load data
print("Loading data...")
responses = pd.read_csv('responses.csv')
participants = pd.read_csv('participants.csv')
sessions = pd.read_csv('sessions.csv')

print(f"Dataset: {{len(participants)}} participants, {{len(responses)}} responses, {{len(sessions)}} sessions")

# Data preprocessing
responses['accuracy'] = responses['correct'].astype(int)
responses['log_rt'] = np.log(responses['rt_ms'])

print("\n=== DATASET OVERVIEW ===")
print(f"Participants: {{len(participants)}}")
print(f"Total responses: {{len(responses)}}")
print(f"Sessions: {{len(sessions)}}")
print(f"Groups: {{participants['group'].value_counts()}}")

# Group comparisons
print("\n=== GROUP COMPARISONS ===")
group_stats = participants.groupby('group').agg({{
    'overall_accuracy': ['count', 'mean', 'std'],
    'mean_rt': ['mean', 'std'],
    'total_trials': ['mean', 'std']
}}).round(3)

print("Group Statistics:")
print(group_stats)

# Statistical tests
groups = participants['group'].unique()
if len(groups) > 1:
    print("\n=== STATISTICAL TESTS ===")
    
    # Accuracy ANOVA
    group_accuracies = [participants[participants['group'] == group]['overall_accuracy'] 
                       for group in groups]
    f_stat, p_val = f_oneway(*group_accuracies)
    print(f"Accuracy ANOVA: F = {{f_stat:.3f}}, p = {{p_val:.3f}}")
    
    # Effect size (eta-squared)
    eta_sq = f_stat * (len(groups) - 1) / (f_stat * (len(groups) - 1) + len(participants) - len(groups))
    print(f"Effect size (eta-squared): {{eta_sq:.3f}}")
    
    # RT ANOVA
    group_rts = [participants[participants['group'] == group]['mean_rt'] 
                for group in groups]
    f_stat_rt, p_val_rt = f_oneway(*group_rts)
    print(f"RT ANOVA: F = {{f_stat_rt:.3f}}, p = {{p_val_rt:.3f}}")

# Learning curve visualization
print("\n=== LEARNING CURVES ===")

# Aggregate data for learning curves
learning_data = responses.groupby(['group', 'trial_number']).agg({{
    'accuracy': ['mean', 'sem', 'count']
}}).reset_index()

learning_data.columns = ['group', 'trial_number', 'accuracy', 'sem', 'count']
learning_data = learning_data[learning_data['count'] >= 3]  # Filter for sufficient data

# Plot learning curves
fig, axes = plt.subplots(2, 2, figsize=(15, 10))

# Accuracy over trials
for group in groups:
    group_data = learning_data[learning_data['group'] == group]
    axes[0, 0].plot(group_data['trial_number'], group_data['accuracy'], 
                   label=group, marker='o', alpha=0.7, markersize=3)
    axes[0, 0].fill_between(group_data['trial_number'], 
                           group_data['accuracy'] - group_data['sem'],
                           group_data['accuracy'] + group_data['sem'], 
                           alpha=0.2)

axes[0, 0].set_xlabel('Trial Number')
axes[0, 0].set_ylabel('Accuracy')
axes[0, 0].set_title('Learning Curves by Group')
axes[0, 0].legend()
axes[0, 0].grid(True, alpha=0.3)

# RT over trials
rt_data = responses.groupby(['group', 'trial_number']).agg({{
    'rt_ms': ['mean', 'sem', 'count']
}}).reset_index()
rt_data.columns = ['group', 'trial_number', 'rt_ms', 'sem', 'count']
rt_data = rt_data[rt_data['count'] >= 3]

for group in groups:
    group_data = rt_data[rt_data['group'] == group]
    axes[0, 1].plot(group_data['trial_number'], group_data['rt_ms'], 
                   label=group, marker='o', alpha=0.7, markersize=3)

axes[0, 1].set_xlabel('Trial Number')
axes[0, 1].set_ylabel('Response Time (ms)')
axes[0, 1].set_title('Response Time Over Trials by Group')
axes[0, 1].legend()
axes[0, 1].grid(True, alpha=0.3)

# Group accuracy distribution
participants.boxplot(column='overall_accuracy', by='group', ax=axes[1, 0])
axes[1, 0].set_title('Accuracy Distribution by Group')
axes[1, 0].set_xlabel('Group')
axes[1, 0].set_ylabel('Overall Accuracy')

# Group RT distribution
participants.boxplot(column='mean_rt', by='group', ax=axes[1, 1])
axes[1, 1].set_title('RT Distribution by Group')
axes[1, 1].set_xlabel('Group')
axes[1, 1].set_ylabel('Mean RT (ms)')

plt.tight_layout()
plt.show()

# Mixed-effects modeling (simplified)
print("\n=== MIXED-EFFECTS MODELING ===")

# Learning model using statsmodels
try:
    # Add random effects approximation by centering within participants
    responses_centered = responses.copy()
    participant_means = responses.groupby('learner_id')['accuracy'].transform('mean')
    responses_centered['accuracy_centered'] = responses_centered['accuracy'] - participant_means
    
    # Fit learning model
    learning_model = smf.ols('accuracy ~ trial_number * C(group)', data=responses).fit()
    print("Learning Model (Trial × Group interaction):")
    print(learning_model.summary().tables[1])
    
except Exception as e:
    print(f"Mixed-effects modeling error: {{e}}")
    print("Using simpler linear models...")

# Phase analysis (if applicable)
if 'phase' in responses.columns:
    print("\n=== PHASE ANALYSIS ===")
    
    phase_stats = responses.groupby(['group', 'phase']).agg({{
        'accuracy': ['mean', 'std', 'count'],
        'rt_ms': ['mean', 'std']
    }}).round(3)
    
    print("Phase Statistics:")
    print(phase_stats)
    
    # Phase visualization
    plt.figure(figsize=(12, 5))
    
    plt.subplot(1, 2, 1)
    sns.boxplot(x='phase', y='accuracy', hue='group', data=responses)
    plt.title('Accuracy by Phase and Group')
    plt.ylabel('Accuracy')
    
    plt.subplot(1, 2, 2)
    sns.boxplot(x='phase', y='rt_ms', hue='group', data=responses)
    plt.title('Response Time by Phase and Group')
    plt.ylabel('Response Time (ms)')
    
    plt.tight_layout()
    plt.show()

# Individual differences
print("\n=== INDIVIDUAL DIFFERENCES ===")

# Calculate individual learning rates (simplified)
learning_rates = []
for participant in responses['learner_id'].unique():
    p_data = responses[responses['learner_id'] == participant]
    if len(p_data) > 10:  # Minimum trials for stable estimate
        try:
            # Linear correlation as learning rate proxy
            corr, _ = stats.pearsonr(p_data['trial_number'], p_data['accuracy'])
            learning_rates.append({{
                'learner_id': participant,
                'learning_rate': corr,
                'group': p_data['group'].iloc[0]
            }})
        except:
            pass

if learning_rates:
    lr_df = pd.DataFrame(learning_rates)
    
    plt.figure(figsize=(8, 6))
    sns.boxplot(x='group', y='learning_rate', data=lr_df)
    plt.title('Learning Rate Distribution by Group')
    plt.ylabel('Learning Rate (correlation coefficient)')
    plt.show()
    
    # Statistical test for learning rate differences
    if len(groups) > 1:
        group_lrs = [lr_df[lr_df['group'] == group]['learning_rate'] for group in groups]
        f_lr, p_lr = f_oneway(*group_lrs)
        print(f"Learning Rate ANOVA: F = {{f_lr:.3f}}, p = {{p_lr:.3f}}")

print("\n=== ANALYSIS COMPLETE ===")
print("Results and visualizations generated successfully!")

# Save processed data
responses.to_csv('processed_responses.csv', index=False)
participants.to_csv('processed_participants.csv', index=False)
if learning_rates:
    lr_df.to_csv('learning_rates.csv', index=False)

print("Processed data saved to CSV files.")
"#,
            self.learners.len()
        );

        Ok(script)
    }

    fn generate_population_notebook(&self) -> std::io::Result<String> {
        let notebook = serde_json::json!({
            "cells": [
                {
                    "cell_type": "markdown",
                    "metadata": {},
                    "source": [
                        "# Population Learning Analysis\n",
                        format!("**Dataset:** {} participants\n", self.learners.len()),
                        format!("**Generated:** {}\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")),
                        "\nThis notebook analyzes learning data across multiple participants."
                    ]
                },
                {
                    "cell_type": "code",
                    "execution_count": null,
                    "metadata": {},
                    "outputs": [],
                    "source": [
                        "import pandas as pd\n",
                        "import numpy as np\n",
                        "import matplotlib.pyplot as plt\n",
                        "import seaborn as sns\n",
                        "from scipy import stats\n",
                        "import warnings\n",
                        "warnings.filterwarnings('ignore')\n",
                        "\n",
                        "# Load data\n",
                        "responses = pd.read_csv('responses.csv')\n",
                        "participants = pd.read_csv('participants.csv')\n",
                        "sessions = pd.read_csv('sessions.csv')\n",
                        "\n",
                        "print(f'Dataset: {{len(participants)}} participants, {{len(responses)}} responses')\n",
                        "participants.head()"
                    ]
                },
                {
                    "cell_type": "markdown",
                    "metadata": {},
                    "source": ["## Group Comparisons"]
                },
                {
                    "cell_type": "code",
                    "execution_count": null,
                    "metadata": {},
                    "outputs": [],
                    "source": [
                        "# Group statistics\n",
                        "group_stats = participants.groupby('group').agg({{\n",
                        "    'overall_accuracy': ['count', 'mean', 'std'],\n",
                        "    'mean_rt': ['mean', 'std']\n",
                        "}}).round(3)\n",
                        "\n",
                        "print('Group Statistics:')\n",
                        "display(group_stats)\n",
                        "\n",
                        "# Visualization\n",
                        "fig, axes = plt.subplots(1, 2, figsize=(12, 5))\n",
                        "\n",
                        "participants.boxplot(column='overall_accuracy', by='group', ax=axes[0])\n",
                        "axes[0].set_title('Accuracy by Group')\n",
                        "\n",
                        "participants.boxplot(column='mean_rt', by='group', ax=axes[1])\n",
                        "axes[1].set_title('Response Time by Group')\n",
                        "\n",
                        "plt.tight_layout()\n",
                        "plt.show()"
                    ]
                },
                {
                    "cell_type": "markdown",
                    "metadata": {},
                    "source": ["## Learning Curves"]
                },
                {
                    "cell_type": "code",
                    "execution_count": null,
                    "metadata": {},
                    "outputs": [],
                    "source": [
                        "# Learning curve analysis\n",
                        "responses['accuracy'] = responses['correct'].astype(int)\n",
                        "\n",
                        "learning_data = responses.groupby(['group', 'trial_number']).agg({{\n",
                        "    'accuracy': ['mean', 'sem', 'count']\n",
                        "}}).reset_index()\n",
                        "\n",
                        "learning_data.columns = ['group', 'trial_number', 'accuracy', 'sem', 'count']\n",
                        "learning_data = learning_data[learning_data['count'] >= 3]\n",
                        "\n",
                        "plt.figure(figsize=(10, 6))\n",
                        "for group in learning_data['group'].unique():\n",
                        "    group_data = learning_data[learning_data['group'] == group]\n",
                        "    plt.plot(group_data['trial_number'], group_data['accuracy'], \n",
                        "             label=group, marker='o', alpha=0.7)\n",
                        "    plt.fill_between(group_data['trial_number'], \n",
                        "                     group_data['accuracy'] - group_data['sem'],\n",
                        "                     group_data['accuracy'] + group_data['sem'], \n",
                        "                     alpha=0.2)\n",
                        "\n",
                        "plt.xlabel('Trial Number')\n",
                        "plt.ylabel('Accuracy')\n",
                        "plt.title('Learning Curves by Group')\n",
                        "plt.legend()\n",
                        "plt.grid(True, alpha=0.3)\n",
                        "plt.show()"
                    ]
                }
            ],
            "metadata": {
                "kernelspec": {
                    "display_name": "Python 3",
                    "language": "python",
                    "name": "python3"
                },
                "language_info": {
                    "name": "python",
                    "version": "3.8.0"
                }
            },
            "nbformat": 4,
            "nbformat_minor": 4
        });

        serde_json::to_string_pretty(&notebook)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    fn generate_population_codebook(&self) -> std::io::Result<String> {
        let codebook = format!(
            r#"
POPULATION DATASET CODEBOOK
============================

Dataset: Multi-participant learning experiment
Generated: {}
Participants: {}
Total sessions: {}

FILE DESCRIPTIONS
================

responses.csv: Trial-by-trial response data
    - Contains all individual responses across all participants
    - One row per trial per participant
    - Primary file for detailed analysis

participants.csv: Participant-level summary data
    - Contains aggregate measures for each participant
    - One row per participant
    - Good for between-subjects comparisons

sessions.csv: Session-level data
    - Contains summary data for each testing session
    - Multiple rows per participant (if multi-session design)
    - Good for learning curve and longitudinal analysis

model_parameters.csv: Cognitive model parameters
    - Contains estimated model parameters for each participant
    - Includes node embeddings, proficiencies, memory strengths
    - For advanced cognitive modeling analysis

VARIABLE REFERENCE
==================

Key Variables in responses.csv:
- learner_id: Participant identifier
- group: Experimental condition (Adaptive, Linear, Yoked)
- trial_number: Sequential trial number (1 to N)
- correct: Response accuracy (0/1)
- rt_ms: Response time in milliseconds
- phase: Experimental phase (training/test/transfer)
- difficulty: Task difficulty (0.0 to 1.0)

Key Variables in participants.csv:
- learner_id: Participant identifier
- group: Experimental condition
- overall_accuracy: Mean accuracy across all trials
- mean_rt: Mean response time across all trials
- total_trials: Total number of trials completed

ANALYSIS RECOMMENDATIONS
========================

1. GROUP COMPARISONS:
   - Use overall_accuracy and mean_rt from participants.csv
   - ANOVA for group differences
   - Effect size reporting (eta-squared or Cohen's d)

2. LEARNING CURVES:
   - Use responses.csv grouped by trial_number and group
   - Plot accuracy over trial_number
   - Consider smoothing (LOESS) for visualization

3. MIXED-EFFECTS MODELING:
   - DV: correct (binary) or rt_ms (continuous)
   - Fixed effects: trial_number, group, phase
   - Random effects: (1 | learner_id) at minimum
   - Consider random slopes if justified

4. TRANSFER ANALYSIS:
   - Compare performance in transfer vs training phases
   - Look for group × phase interactions
   - Calculate transfer gain scores

5. INDIVIDUAL DIFFERENCES:
   - Use model_parameters.csv for cognitive profiles
   - Correlate parameters with performance measures
   - Cluster analysis for strategy identification

QUALITY CHECKS
==============

Before analysis, check for:
- Missing data patterns
- Outlier response times (< 100ms or > 10s)
- Participants with < 50% completion
- Session effects (practice, fatigue)
- Ceiling/floor effects in accuracy

STATISTICAL SOFTWARE
====================

R: Use lme4 package for mixed-effects models
Python: Use statsmodels or scipy for statistics
SPSS: Import CSVs and use MIXED for multilevel modeling

Generated by Alphabet Terminal Prototype
Version: {} 
"#,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            self.learners.len(),
            self.learners
                .iter()
                .map(|l| l.sessions.len())
                .sum::<usize>(),
            env!("CARGO_PKG_VERSION")
        );

        Ok(codebook)
    }

    fn export_combined_responses(&self, path: &Path) -> std::io::Result<()> {
        let mut csv = String::new();
        csv.push_str("learner_id,session_id,trial_number,timestamp,group,phase,task_type,task_json,correct,rt_ms,user_answer,correct_answer,difficulty\n");

        for learner in &self.learners {
            let learner_csv = learner.to_csv();
            // Skip header line for subsequent learners
            let lines: Vec<&str> = learner_csv.lines().collect();
            if lines.len() > 1 {
                for line in lines.iter().skip(1) {
                    csv.push_str(line);
                    csv.push('\n');
                }
            }
        }

        let mut file = File::create(path)?;
        file.write_all(csv.as_bytes())?;
        Ok(())
    }

    fn export_participants(&self, path: &Path) -> std::io::Result<()> {
        let mut csv = String::new();
        csv.push_str("learner_id,group,age,gender,start_date,end_date,n_sessions,total_trials,overall_accuracy,mean_rt,completed\n");

        for learner in &self.learners {
            let total_trials: usize = learner.sessions.iter().map(|s| s.summary.total_tasks).sum();

            let total_correct: usize = learner
                .sessions
                .iter()
                .map(|s| s.summary.correct_count)
                .sum();

            let overall_accuracy = if total_trials > 0 {
                total_correct as f64 / total_trials as f64
            } else {
                0.0
            };

            let mean_rt = if !learner.sessions.is_empty() {
                let total_rt: f64 = learner
                    .sessions
                    .iter()
                    .map(|s| s.summary.mean_rt_ms * s.summary.total_tasks as f64)
                    .sum();
                total_rt / total_trials as f64
            } else {
                0.0
            };

            let start_date = learner
                .sessions
                .first()
                .map(|s| s.start_time.date_naive().to_string())
                .unwrap_or_else(|| "NA".to_string());

            let end_date = learner
                .sessions
                .last()
                .and_then(|s| s.end_time)
                .map(|t| t.date_naive().to_string())
                .unwrap_or_else(|| "NA".to_string());

            // Infer group from metadata
            let group = learner
                .metadata
                .experiment_id
                .as_ref()
                .map(|id| {
                    if id.contains("adaptive") {
                        "Adaptive"
                    } else if id.contains("linear") {
                        "Linear"
                    } else {
                        "Yoked"
                    }
                })
                .unwrap_or("Unknown");

            csv.push_str(&format!(
                "{},{},NA,NA,{},{},{},{},{:.3},{:.1},true\n",
                learner.learner_id,
                group,
                start_date,
                end_date,
                learner.sessions.len(),
                total_trials,
                overall_accuracy,
                mean_rt
            ));
        }

        let mut file = File::create(path)?;
        file.write_all(csv.as_bytes())?;
        Ok(())
    }

    fn export_all_sessions(&self, path: &Path) -> std::io::Result<()> {
        let mut csv = String::new();
        csv.push_str("learner_id,session_id,session_number,start_time,end_time,duration_minutes,n_trials,accuracy,mean_rt_ms,median_rt_ms,strategy_detected\n");

        for learner in &self.learners {
            for (i, session) in learner.sessions.iter().enumerate() {
                let duration_minutes = session
                    .end_time
                    .map(|end| (end - session.start_time).num_minutes() as f64)
                    .unwrap_or(0.0);

                csv.push_str(&format!(
                    "{},{},{},{},{},{:.1},{},{:.3},{:.1},{:.1},{:?}\n",
                    learner.learner_id,
                    session.session_id,
                    i + 1,
                    session.start_time.to_rfc3339(),
                    session
                        .end_time
                        .map(|t| t.to_rfc3339())
                        .unwrap_or_else(|| "NA".to_string()),
                    duration_minutes,
                    session.summary.total_tasks,
                    session.summary.accuracy,
                    session.summary.mean_rt_ms,
                    session.summary.median_rt_ms,
                    session
                        .summary
                        .strategy_detected
                        .as_ref()
                        .map(|s| format!("{:?}", s))
                        .unwrap_or_else(|| "NA".to_string())
                ));
            }
        }

        let mut file = File::create(path)?;
        file.write_all(csv.as_bytes())?;
        Ok(())
    }

    fn export_all_parameters(&self, path: &Path) -> std::io::Result<()> {
        let mut csv = String::new();
        csv.push_str(
            "learner_id,timestamp,trial_number,parameter_type,parameter_name,value,uncertainty\n",
        );

        for learner in &self.learners {
            let trial_count = learner.performance_trajectories.len();

            // Export node embeddings
            for (node, embedding) in &learner.model_parameters.node_embeddings {
                csv.push_str(&format!(
                    "{},{},{},node_embedding,{},{:.6},{:.6}\n",
                    learner.learner_id,
                    learner.model_parameters.timestamp.to_rfc3339(),
                    trial_count,
                    node,
                    embedding.position,
                    embedding.uncertainty
                ));
            }

            // Export operation proficiencies
            for (op, proficiency) in &learner.model_parameters.operation_proficiencies {
                csv.push_str(&format!(
                    "{},{},{},operation_proficiency,{},{:.6},NA\n",
                    learner.learner_id,
                    learner.model_parameters.timestamp.to_rfc3339(),
                    trial_count,
                    op,
                    proficiency
                ));
            }

            // Export memory strengths
            for (item, strength) in &learner.model_parameters.memory_strengths {
                csv.push_str(&format!(
                    "{},{},{},memory_strength,{},{:.6},NA\n",
                    learner.learner_id,
                    learner.model_parameters.timestamp.to_rfc3339(),
                    trial_count,
                    item,
                    strength
                ));
            }

            // Export chunk boundaries
            for boundary in &learner.model_parameters.chunk_boundaries {
                csv.push_str(&format!(
                    "{},{},{},chunk_boundary,position_{},{:.6},NA\n",
                    learner.learner_id,
                    learner.model_parameters.timestamp.to_rfc3339(),
                    trial_count,
                    boundary.position,
                    boundary.strength
                ));
            }
        }

        let mut file = File::create(path)?;
        file.write_all(csv.as_bytes())?;
        Ok(())
    }

    pub fn find_population_bottlenecks(&self) -> Vec<(String, String, f64)> {
        let mut transition_errors: HashMap<(String, String), (usize, usize)> = HashMap::new();

        for learner in &self.learners {
            for session in &learner.sessions {
                for window in session.responses.windows(2) {
                    if let [prev, curr] = window {
                        let transition = (
                            format!("{:?}", prev.task.task_type),
                            format!("{:?}", curr.task.task_type),
                        );

                        let entry = transition_errors.entry(transition).or_insert((0, 0));
                        entry.1 += 1; // total
                        if !curr.correct {
                            entry.0 += 1; // errors
                        }
                    }
                }
            }
        }

        let mut bottlenecks: Vec<(String, String, f64)> = transition_errors
            .into_iter()
            .filter(|(_, (_, total))| *total >= 10) // Minimum sample size
            .map(|((from, to), (errors, total))| (from, to, errors as f64 / total as f64))
            .collect();

        bottlenecks.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        bottlenecks.truncate(20);

        bottlenecks
    }

    pub fn compute_item_difficulties(&self) -> HashMap<String, DifficultyNorm> {
        let mut item_performance: HashMap<String, Vec<bool>> = HashMap::new();

        for learner in &self.learners {
            for session in &learner.sessions {
                for response in &session.responses {
                    let item_key = format!("{:?}", response.task.task_type);
                    item_performance
                        .entry(item_key)
                        .or_insert_with(Vec::new)
                        .push(response.correct);
                }
            }
        }

        item_performance
            .into_iter()
            .map(|(item, results)| {
                let success_rate =
                    results.iter().filter(|&&c| c).count() as f64 / results.len() as f64;

                let norm = DifficultyNorm {
                    item_id: item.clone(),
                    empirical_difficulty: 1.0 - success_rate,
                    sample_size: results.len(),
                    confidence_interval: Self::calculate_ci(success_rate, results.len()),
                };

                (item, norm)
            })
            .collect()
    }

    fn calculate_ci(p: f64, n: usize) -> (f64, f64) {
        // Wilson score interval
        let z = 1.96; // 95% confidence
        let n_f = n as f64;

        let denominator = 1.0 + z * z / n_f;
        let center = (p + z * z / (2.0 * n_f)) / denominator;
        let spread = z * (p * (1.0 - p) / n_f + z * z / (4.0 * n_f * n_f)).sqrt() / denominator;

        ((center - spread).max(0.0), (center + spread).min(1.0))
    }

    pub fn cluster_by_strategy(&self) -> HashMap<StrategyType, Vec<String>> {
        let mut clusters: HashMap<StrategyType, Vec<String>> = HashMap::new();

        // Simplified strategy detection based on RT patterns
        for learner in &self.learners {
            let mut rt_by_distance: HashMap<usize, Vec<f64>> = HashMap::new();

            for session in &learner.sessions {
                for response in &session.responses {
                    // Estimate distance from task (simplified)
                    let distance = (response.task.difficulty * 10.0) as usize;
                    rt_by_distance
                        .entry(distance)
                        .or_insert_with(Vec::new)
                        .push(response.response_time_ms as f64);
                }
            }

            // Calculate RT slope
            let distances: Vec<f64> = rt_by_distance.keys().map(|&d| d as f64).collect();
            let mean_rts: Vec<f64> = rt_by_distance
                .values()
                .map(|rts| rts.iter().sum::<f64>() / rts.len() as f64)
                .collect();

            let slope = if distances.len() >= 2 {
                Self::calculate_slope(&distances, &mean_rts)
            } else {
                0.0
            };

            let strategy = if slope > 100.0 {
                StrategyType::SerialScan
            } else if slope < 50.0 {
                StrategyType::DirectIndex
            } else {
                StrategyType::Mixed(50)
            };

            clusters
                .entry(strategy)
                .or_insert_with(Vec::new)
                .push(learner.learner_id.clone());
        }

        clusters
    }

    fn calculate_slope(x: &[f64], y: &[f64]) -> f64 {
        let n = x.len().min(y.len()) as f64;
        if n < 2.0 {
            return 0.0;
        }

        let sum_x: f64 = x.iter().take(n as usize).sum();
        let sum_y: f64 = y.iter().take(n as usize).sum();
        let sum_xy: f64 = x
            .iter()
            .zip(y.iter())
            .take(n as usize)
            .map(|(a, b)| a * b)
            .sum();
        let sum_x2: f64 = x.iter().take(n as usize).map(|a| a * a).sum();

        (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyNorm {
    pub item_id: String,
    pub empirical_difficulty: f64,
    pub sample_size: usize,
    pub confidence_interval: (f64, f64),
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

// Add UUID support
pub use uuid;

// Session exporter for test compatibility
pub struct SessionExporter;

impl SessionExporter {
    pub fn new() -> Self {
        SessionExporter
    }

    pub fn export_to_csv(&self, responses: &[TaskResponse]) -> Result<String, std::io::Error> {
        let mut csv = String::new();
        csv.push_str("task_id,response_time_ms,correct,user_answer\n");

        for (i, response) in responses.iter().enumerate() {
            csv.push_str(&format!(
                "{},{},{},{}\n",
                i, response.response_time_ms, response.correct, response.user_answer
            ));
        }

        Ok(csv)
    }

    pub fn generate_summary(&self, responses: &[TaskResponse]) -> String {
        let total = responses.len();
        let correct = responses.iter().filter(|r| r.correct).count();
        let accuracy = if total > 0 {
            (correct as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        format!(
            "Total responses: {}\nCorrect: {}\nOverall accuracy: {:.2}%",
            total, correct, accuracy
        )
    }
}
