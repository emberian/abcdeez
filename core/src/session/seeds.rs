use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Comprehensive seed management system for reproducible randomization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedManager {
    pub master_seed: Option<u64>,
    pub experiment_seeds: HashMap<String, ExperimentSeed>,
    pub session_seeds: HashMap<String, SessionSeed>,
    pub randomization_log: Vec<RandomizationEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentSeed {
    pub experiment_id: String,
    pub seed: u64,
    pub created_at: DateTime<Utc>,
    pub algorithm: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSeed {
    pub session_id: String,
    pub experiment_id: String,
    pub participant_id: String,
    pub seed: u64,
    pub parent_seed: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomizationEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub seed_used: u64,
    pub operation: String,
    pub experiment_id: String,
    pub session_id: Option<String>,
    pub participant_id: Option<String>,
    pub outcome: String,
}

impl SeedManager {
    pub fn new(master_seed: Option<u64>) -> Self {
        Self {
            master_seed,
            experiment_seeds: HashMap::new(),
            session_seeds: HashMap::new(),
            randomization_log: Vec::new(),
        }
    }

    pub fn generate_experiment_seed(&mut self, experiment_id: String, description: String) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let seed = if let Some(master) = self.master_seed {
            let mut hasher = DefaultHasher::new();
            master.hash(&mut hasher);
            experiment_id.hash(&mut hasher);
            hasher.finish()
        } else {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64
        };

        let experiment_seed = ExperimentSeed {
            experiment_id: experiment_id.clone(),
            seed,
            created_at: Utc::now(),
            algorithm: "default_hasher".to_string(),
            description,
        };

        self.experiment_seeds
            .insert(experiment_id.clone(), experiment_seed);

        self.log_randomization_event(
            seed,
            "generate_experiment_seed".to_string(),
            experiment_id,
            None,
            None,
            format!("Generated experiment seed: {}", seed),
        );

        seed
    }

    pub fn generate_session_seed(
        &mut self,
        session_id: String,
        experiment_id: String,
        participant_id: String,
    ) -> u64 {
        let parent_seed = self
            .experiment_seeds
            .get(&experiment_id)
            .map(|es| es.seed)
            .unwrap_or_else(|| {
                self.generate_experiment_seed(experiment_id.clone(), "Auto-generated".to_string())
            });

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        parent_seed.hash(&mut hasher);
        session_id.hash(&mut hasher);
        participant_id.hash(&mut hasher);
        let seed = hasher.finish();

        let session_seed = SessionSeed {
            session_id: session_id.clone(),
            experiment_id: experiment_id.clone(),
            participant_id: participant_id.clone(),
            seed,
            parent_seed,
            created_at: Utc::now(),
        };

        self.session_seeds.insert(session_id.clone(), session_seed);

        self.log_randomization_event(
            seed,
            "generate_session_seed".to_string(),
            experiment_id,
            Some(session_id),
            Some(participant_id),
            format!("Generated session seed: {}", seed),
        );

        seed
    }

    fn log_randomization_event(
        &mut self,
        seed_used: u64,
        operation: String,
        experiment_id: String,
        session_id: Option<String>,
        participant_id: Option<String>,
        outcome: String,
    ) {
        let event = RandomizationEvent {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            seed_used,
            operation,
            experiment_id,
            session_id,
            participant_id,
            outcome,
        };

        self.randomization_log.push(event);
    }

    pub fn get_reproducibility_manifest(
        &self,
        experiment_id: &str,
    ) -> Option<ReproducibilityManifest> {
        if let Some(experiment_seed) = self.experiment_seeds.get(experiment_id) {
            let session_seeds: Vec<_> = self
                .session_seeds
                .values()
                .filter(|ss| ss.experiment_id == experiment_id)
                .collect();

            let randomization_events: Vec<_> = self
                .randomization_log
                .iter()
                .filter(|event| event.experiment_id == experiment_id)
                .collect();

            Some(ReproducibilityManifest {
                experiment_id: experiment_id.to_string(),
                master_seed: self.master_seed,
                experiment_seed: experiment_seed.clone(),
                session_seeds: session_seeds.into_iter().cloned().collect(),
                randomization_events: randomization_events.into_iter().cloned().collect(),
                generated_at: Utc::now(),
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReproducibilityManifest {
    pub experiment_id: String,
    pub master_seed: Option<u64>,
    pub experiment_seed: ExperimentSeed,
    pub session_seeds: Vec<SessionSeed>,
    pub randomization_events: Vec<RandomizationEvent>,
    pub generated_at: DateTime<Utc>,
}
