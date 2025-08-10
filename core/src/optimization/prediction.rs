use crate::core::learner::LearnerModel;
use crate::tasks::types::TaskResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PredictorType {
    PowerLaw,          // P(t) = a * t^b + c
    Exponential,       // P(t) = a * (1 - e^(-b*t))
    Logistic,          // P(t) = L / (1 + e^(-k*(t-t0)))
    BiExponential,     // P(t) = a*e^(-b*t) + c*e^(-d*t)
    AdaptiveComposite, // Combines multiple models
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictorParams {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub l: f64,  // Logistic asymptote
    pub k: f64,  // Logistic growth rate
    pub t0: f64, // Logistic midpoint
}

impl Default for PredictorParams {
    fn default() -> Self {
        PredictorParams {
            a: 0.5,
            b: 0.3,
            c: 0.5,
            d: 0.1,
            l: 0.95,
            k: 0.1,
            t0: 20.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PerformancePredictor {
    pub model_type: PredictorType,
    pub parameters: PredictorParams,
    pub fit_quality: f64, // R-squared value
}

impl PerformancePredictor {
    pub fn new(model_type: PredictorType) -> Self {
        PerformancePredictor {
            model_type,
            parameters: PredictorParams::default(),
            fit_quality: 0.0,
        }
    }

    pub fn fit(&mut self, data: &[(usize, f64)]) {
        if data.len() < 3 {
            return; // Not enough data to fit
        }

        match self.model_type {
            PredictorType::PowerLaw => self.fit_power_law(data),
            PredictorType::Exponential => self.fit_exponential(data),
            PredictorType::Logistic => self.fit_logistic(data),
            PredictorType::BiExponential => self.fit_biexponential(data),
            PredictorType::AdaptiveComposite => self.fit_adaptive(data),
        }

        self.fit_quality = self.calculate_r_squared(data);
    }

    fn fit_power_law(&mut self, data: &[(usize, f64)]) {
        // P(t) = a * t^b + c
        // Using logarithmic transformation for linear regression
        let log_data: Vec<(f64, f64)> = data
            .iter()
            .filter(|(t, p)| *t > 0 && *p > 0.0)
            .map(|(t, p)| ((*t as f64).ln(), p.ln()))
            .collect();

        if log_data.len() < 2 {
            return;
        }

        let (slope, intercept) = self.linear_regression(&log_data);

        self.parameters.b = slope;
        self.parameters.a = intercept.exp();

        // Estimate asymptote
        let last_values: Vec<f64> = data.iter().rev().take(5).map(|(_, p)| *p).collect();
        self.parameters.c = if !last_values.is_empty() {
            last_values.iter().sum::<f64>() / last_values.len() as f64
        } else {
            0.5
        };
    }

    fn fit_exponential(&mut self, data: &[(usize, f64)]) {
        // P(t) = a * (1 - e^(-b*t))
        // Estimate asymptote
        let last_values: Vec<f64> = data.iter().rev().take(10).map(|(_, p)| *p).collect();

        self.parameters.a = if !last_values.is_empty() {
            last_values.iter().sum::<f64>() / last_values.len() as f64
        } else {
            0.95
        };

        // Estimate rate using early data
        if data.len() >= 2 {
            let early_slope = (data[1].1 - data[0].1) / (data[1].0 - data[0].0) as f64;
            self.parameters.b = early_slope / self.parameters.a;
        }
    }

    fn fit_logistic(&mut self, data: &[(usize, f64)]) {
        // P(t) = L / (1 + e^(-k*(t-t0)))

        // Estimate asymptote
        let last_values: Vec<f64> = data.iter().rev().take(10).map(|(_, p)| *p).collect();

        self.parameters.l = if !last_values.is_empty() {
            last_values.iter().sum::<f64>() / last_values.len() as f64
        } else {
            0.95
        };

        // Find midpoint (where P ≈ L/2)
        let half_max = self.parameters.l / 2.0;
        let midpoint = data
            .iter()
            .find(|(_, p)| *p >= half_max)
            .map(|(t, _)| *t as f64)
            .unwrap_or(20.0);

        self.parameters.t0 = midpoint;

        // Estimate growth rate from slope at midpoint
        let window = 3;
        let mid_idx = data
            .iter()
            .position(|(t, _)| *t as f64 >= midpoint)
            .unwrap_or(data.len() / 2);

        if mid_idx > window && mid_idx + window < data.len() {
            let before = data[mid_idx - window].1;
            let after = data[mid_idx + window].1;
            let dt = (2 * window) as f64;
            let slope = (after - before) / dt;

            // At midpoint, dP/dt = k*L/4
            self.parameters.k = 4.0 * slope / self.parameters.l;
        } else {
            self.parameters.k = 0.1;
        }
    }

    fn fit_biexponential(&mut self, data: &[(usize, f64)]) {
        // P(t) = a*e^(-b*t) + c*e^(-d*t)
        // Simplified: fast and slow learning components

        // Split data into early and late phases
        let split_point = data.len() / 3;
        let early_data = &data[..split_point];
        let late_data = &data[split_point..];

        // Fit fast component to early data
        if !early_data.is_empty() {
            let early_rate =
                (early_data.last().unwrap().1 - early_data[0].1) / early_data.len() as f64;
            self.parameters.b = early_rate.abs();
            self.parameters.a = 0.5;
        }

        // Fit slow component to late data
        if !late_data.is_empty() {
            let late_rate = (late_data.last().unwrap().1 - late_data[0].1) / late_data.len() as f64;
            self.parameters.d = late_rate.abs() / 10.0; // Slower decay
            self.parameters.c = 0.5;
        }
    }

    fn fit_adaptive(&mut self, data: &[(usize, f64)]) {
        // Try all models and pick the best
        let mut best_model = PredictorType::PowerLaw;
        let mut best_r2 = 0.0;

        for model_type in [
            PredictorType::PowerLaw,
            PredictorType::Exponential,
            PredictorType::Logistic,
        ] {
            let mut temp_predictor = PerformancePredictor::new(model_type.clone());
            temp_predictor.fit(data);
            let r2 = temp_predictor.calculate_r_squared(data);

            if r2 > best_r2 {
                best_r2 = r2;
                best_model = model_type;
                self.parameters = temp_predictor.parameters.clone();
            }
        }

        self.model_type = best_model;
        self.fit_quality = best_r2;
    }

    fn linear_regression(&self, data: &[(f64, f64)]) -> (f64, f64) {
        let n = data.len() as f64;
        let sum_x: f64 = data.iter().map(|(x, _)| x).sum();
        let sum_y: f64 = data.iter().map(|(_, y)| y).sum();
        let sum_xy: f64 = data.iter().map(|(x, y)| x * y).sum();
        let sum_x2: f64 = data.iter().map(|(x, _)| x * x).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;

        (slope, intercept)
    }

    pub fn predict(&self, trial: usize) -> f64 {
        let t = trial as f64;

        match self.model_type {
            PredictorType::PowerLaw => {
                self.parameters.a * t.powf(self.parameters.b) + self.parameters.c
            }
            PredictorType::Exponential => {
                self.parameters.a * (1.0 - (-self.parameters.b * t).exp())
            }
            PredictorType::Logistic => {
                self.parameters.l / (1.0 + (-self.parameters.k * (t - self.parameters.t0)).exp())
            }
            PredictorType::BiExponential => {
                self.parameters.a * (-self.parameters.b * t).exp()
                    + self.parameters.c * (-self.parameters.d * t).exp()
            }
            PredictorType::AdaptiveComposite => {
                // Weighted average based on trial number
                let early_weight = (-t / 20.0).exp();
                let late_weight = 1.0 - early_weight;

                let exp_pred = self.parameters.a * (1.0 - (-self.parameters.b * t).exp());
                let log_pred = self.parameters.l
                    / (1.0 + (-self.parameters.k * (t - self.parameters.t0)).exp());

                early_weight * exp_pred + late_weight * log_pred
            }
        }
        .max(0.0)
        .min(1.0)
    }

    pub fn predict_range(&self, start: usize, end: usize) -> Vec<(usize, f64)> {
        (start..=end).map(|t| (t, self.predict(t))).collect()
    }

    pub fn confidence_interval(&self, trial: usize, _alpha: f64) -> (f64, f64) {
        let prediction = self.predict(trial);

        // Simple confidence interval based on fit quality
        let stderr = (1.0 - self.fit_quality).sqrt() * 0.1;
        let z = 1.96; // 95% confidence for alpha = 0.05

        let margin = z * stderr;

        (
            (prediction - margin).max(0.0),
            (prediction + margin).min(1.0),
        )
    }

    fn calculate_r_squared(&self, data: &[(usize, f64)]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        let mean_y = data.iter().map(|(_, y)| y).sum::<f64>() / data.len() as f64;

        let ss_tot: f64 = data.iter().map(|(_, y)| (y - mean_y).powi(2)).sum();

        let ss_res: f64 = data
            .iter()
            .map(|(t, y)| {
                let pred = self.predict(*t);
                (y - pred).powi(2)
            })
            .sum();

        if ss_tot == 0.0 {
            0.0
        } else {
            1.0 - (ss_res / ss_tot)
        }
    }
}

// Optimal schedule generation
pub struct ScheduleOptimizer {
    learner_model: LearnerModel,
    predictor: PerformancePredictor,
}

impl ScheduleOptimizer {
    pub fn new(learner_model: LearnerModel) -> Self {
        ScheduleOptimizer {
            learner_model,
            predictor: PerformancePredictor::new(PredictorType::AdaptiveComposite),
        }
    }

    pub fn generate_optimal_schedule(
        &mut self,
        target_mastery: f64,
        max_sessions: usize,
        session_length: usize,
    ) -> Vec<SessionPlan> {
        let mut schedule = Vec::new();
        let mut current_mastery = self.estimate_current_mastery();

        for session_num in 1..=max_sessions {
            if current_mastery >= target_mastery {
                break;
            }

            let session_plan = self.plan_session(
                session_num,
                session_length,
                target_mastery - current_mastery,
            );

            // Simulate expected improvement
            let expected_improvement = session_plan.expected_improvement;
            current_mastery += expected_improvement;

            schedule.push(session_plan);
        }

        schedule
    }

    fn estimate_current_mastery(&self) -> f64 {
        // Average proficiency across all operations
        let proficiencies: Vec<f64> = self
            .learner_model
            .operation_proficiencies
            .values()
            .map(|p| sigmoid(p.theta))
            .collect();

        if proficiencies.is_empty() {
            0.0
        } else {
            proficiencies.iter().sum::<f64>() / proficiencies.len() as f64
        }
    }

    fn plan_session(&self, session_number: usize, length: usize, _mastery_gap: f64) -> SessionPlan {
        let mut task_types = Vec::new();
        let mut difficulties = Vec::new();

        // Progressive difficulty based on session number
        let base_difficulty = 0.3 + (session_number as f64 * 0.05).min(0.4);

        // Mix of task types weighted by weakness
        let weak_operations = self.identify_weak_operations();

        for i in 0..length {
            // Alternate between weak areas and general practice
            let task_type = if i % 3 == 0 && !weak_operations.is_empty() {
                weak_operations[i % weak_operations.len()].clone()
            } else {
                self.select_balanced_task_type()
            };

            // Vary difficulty around base
            let difficulty = base_difficulty + (rand::random::<f64>() - 0.5) * 0.2;

            task_types.push(task_type);
            difficulties.push(difficulty.max(0.1).min(0.9));
        }

        // Estimate improvement based on predictor
        let current_trial = session_number * length;
        let future_trial = (session_number + 1) * length;

        let current_pred = self.predictor.predict(current_trial);
        let future_pred = self.predictor.predict(future_trial);
        let expected_improvement = (future_pred - current_pred).max(0.0);

        SessionPlan {
            session_number,
            task_types,
            difficulties,
            expected_improvement,
            estimated_duration_minutes: (length as f64 * 1.5) as usize,
        }
    }

    fn identify_weak_operations(&self) -> Vec<String> {
        let mut weak = Vec::new();

        for (op, prof) in &self.learner_model.operation_proficiencies {
            if sigmoid(prof.theta) < 0.6 {
                weak.push(op.clone());
            }
        }

        weak
    }

    fn select_balanced_task_type(&self) -> String {
        let types = vec![
            "Successor",
            "Predecessor",
            "PairwiseOrder",
            "Segment",
            "KJump",
            "Index",
        ];

        types[rand::random::<usize>() % types.len()].to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPlan {
    pub session_number: usize,
    pub task_types: Vec<String>,
    pub difficulties: Vec<f64>,
    pub expected_improvement: f64,
    pub estimated_duration_minutes: usize,
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

// Learning trajectory analysis
pub struct TrajectoryAnalyzer {
    responses: Vec<TaskResponse>,
}

impl TrajectoryAnalyzer {
    pub fn new(responses: Vec<TaskResponse>) -> Self {
        TrajectoryAnalyzer { responses }
    }

    pub fn analyze_learning_trajectory(&self) -> LearningTrajectory {
        let mut accuracy_over_time = Vec::new();
        let mut rt_over_time = Vec::new();
        let mut cumulative_correct = 0;

        for (i, response) in self.responses.iter().enumerate() {
            if response.correct {
                cumulative_correct += 1;
            }

            let accuracy = cumulative_correct as f64 / (i + 1) as f64;
            accuracy_over_time.push((i + 1, accuracy));
            rt_over_time.push((i + 1, response.response_time_ms as f64));
        }

        // Fit performance curve
        let mut predictor = PerformancePredictor::new(PredictorType::AdaptiveComposite);
        predictor.fit(&accuracy_over_time);

        // Detect plateau
        let plateau_point = self.detect_plateau(&accuracy_over_time);

        // Calculate improvement rate
        let improvement_rate = if accuracy_over_time.len() >= 2 {
            let early = accuracy_over_time[0].1;
            let late = accuracy_over_time.last().unwrap().1;
            let trials = accuracy_over_time.len() as f64;
            (late - early) / trials
        } else {
            0.0
        };

        LearningTrajectory {
            accuracy_trajectory: accuracy_over_time,
            rt_trajectory: rt_over_time,
            fitted_model: predictor,
            plateau_trial: plateau_point,
            improvement_rate,
            final_performance: self
                .responses
                .last()
                .map(|r| if r.correct { 1.0 } else { 0.0 })
                .unwrap_or(0.0),
        }
    }

    fn detect_plateau(&self, trajectory: &[(usize, f64)]) -> Option<usize> {
        if trajectory.len() < 10 {
            return None;
        }

        // Look for where improvement rate drops below threshold
        let window_size = 5;
        let threshold = 0.01; // 1% improvement per trial

        for i in window_size..trajectory.len() - window_size {
            let before = trajectory[i - window_size].1;
            let after = trajectory[i + window_size].1;
            let improvement = (after - before) / (2 * window_size) as f64;

            if improvement.abs() < threshold {
                return Some(trajectory[i].0);
            }
        }

        None
    }
}

#[derive(Debug, Clone)]
pub struct LearningTrajectory {
    pub accuracy_trajectory: Vec<(usize, f64)>,
    pub rt_trajectory: Vec<(usize, f64)>,
    pub fitted_model: PerformancePredictor,
    pub plateau_trial: Option<usize>,
    pub improvement_rate: f64,
    pub final_performance: f64,
}
