// offline.rs - Offline-first storage and sync system for the Adaptive Learning System

use chrono::{DateTime, Utc};
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::models::{Learner, PerformanceMetrics, Session, User};
use abcdeez_core::tasks::TaskResponse as CoreTaskResponse;

/// Offline storage provider using SQLite for local persistence
pub struct OfflineStorage {
    db: Arc<RwLock<rusqlite::Connection>>,
    sync_queue: Arc<RwLock<SyncQueue>>,
    cache: Arc<RwLock<LocalCache>>,
    config: OfflineConfig,
}

/// Configuration for offline behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineConfig {
    pub auto_sync: bool,
    pub sync_interval_seconds: u64,
    pub max_cache_size_mb: usize,
    pub retention_days: u32,
    pub conflict_resolution: ConflictResolution,
    pub compression_enabled: bool,
}

impl Default for OfflineConfig {
    fn default() -> Self {
        Self {
            auto_sync: true,
            sync_interval_seconds: 300, // 5 minutes
            max_cache_size_mb: 100,
            retention_days: 30,
            conflict_resolution: ConflictResolution::LastWriteWins,
            compression_enabled: true,
        }
    }
}

/// Conflict resolution strategies for sync
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ConflictResolution {
    LastWriteWins,
    FirstWriteWins,
    MergeResponses,
    Manual,
}

/// Queue for pending sync operations
#[derive(Debug, Clone)]
pub struct SyncQueue {
    pending_operations: VecDeque<SyncOperation>,
    failed_operations: Vec<FailedOperation>,
    last_sync: Option<DateTime<Utc>>,
    sync_in_progress: bool,
}

/// Individual sync operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncOperation {
    pub id: String,
    pub operation_type: OperationType,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub retry_count: u32,
    pub priority: SyncPriority,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OperationType {
    CreateSession,
    UpdateSession,
    SubmitResponse,
    UpdateMetrics,
    CreateUser,
    UpdateUser,
    ExportData,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SyncPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Failed operation with error details
#[derive(Debug, Clone)]
pub struct FailedOperation {
    pub operation: SyncOperation,
    pub error: String,
    pub failed_at: DateTime<Utc>,
    pub permanent_failure: bool,
}

/// Local cache for fast access
pub struct LocalCache {
    sessions: HashMap<String, Session>,
    responses: HashMap<String, Vec<CoreTaskResponse>>,
    metrics: HashMap<String, PerformanceMetrics>,
    users: HashMap<String, User>,
    learners: HashMap<String, Learner>,
    cache_stats: CacheStats,
}

#[derive(Debug, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub size_bytes: usize,
}

/// Sync status for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub online: bool,
    pub last_sync: Option<DateTime<Utc>>,
    pub pending_count: usize,
    pub failed_count: usize,
    pub sync_progress: Option<SyncProgress>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncProgress {
    pub current: usize,
    pub total: usize,
    pub message: String,
}

impl OfflineStorage {
    /// Initialize offline storage with database
    pub async fn new(data_dir: PathBuf) -> Result<Self, StorageError> {
        let db_path = data_dir.join("offline.db");
        let conn = rusqlite::Connection::open(&db_path)?;

        // Initialize database schema
        Self::init_schema(&conn)?;

        let storage = Self {
            db: Arc::new(RwLock::new(conn)),
            sync_queue: Arc::new(RwLock::new(SyncQueue {
                pending_operations: VecDeque::new(),
                failed_operations: Vec::new(),
                last_sync: None,
                sync_in_progress: false,
            })),
            cache: Arc::new(RwLock::new(LocalCache {
                sessions: HashMap::new(),
                responses: HashMap::new(),
                metrics: HashMap::new(),
                users: HashMap::new(),
                learners: HashMap::new(),
                cache_stats: CacheStats::default(),
            })),
            config: OfflineConfig::default(),
        };

        Ok(storage)
    }

    /// Initialize database schema
    fn init_schema(conn: &rusqlite::Connection) -> Result<(), StorageError> {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT NOT NULL,
                email TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                sync_status TEXT DEFAULT 'pending'
            );

            CREATE TABLE IF NOT EXISTS learners (
                id TEXT PRIMARY KEY,
                user_id TEXT,
                display_name TEXT,
                created_at TEXT NOT NULL,
                model_data BLOB,
                metadata TEXT,
                sync_status TEXT DEFAULT 'pending',
                FOREIGN KEY (user_id) REFERENCES users(id)
            );

            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                learner_id TEXT NOT NULL,
                topology_type TEXT NOT NULL,
                topology_data TEXT,
                start_time TEXT NOT NULL,
                end_time TEXT,
                status TEXT NOT NULL,
                summary TEXT,
                sync_status TEXT DEFAULT 'pending',
                FOREIGN KEY (learner_id) REFERENCES learners(id)
            );

            CREATE TABLE IF NOT EXISTS responses (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                task_data TEXT NOT NULL,
                response TEXT NOT NULL,
                correct BOOLEAN NOT NULL,
                response_time_ms INTEGER NOT NULL,
                timestamp TEXT NOT NULL,
                hint_used BOOLEAN DEFAULT FALSE,
                sync_status TEXT DEFAULT 'pending',
                FOREIGN KEY (session_id) REFERENCES sessions(id)
            );

            CREATE TABLE IF NOT EXISTS sync_queue (
                id TEXT PRIMARY KEY,
                operation_type TEXT NOT NULL,
                payload TEXT NOT NULL,
                created_at TEXT NOT NULL,
                retry_count INTEGER DEFAULT 0,
                priority INTEGER DEFAULT 1
            );

            CREATE TABLE IF NOT EXISTS sync_history (
                id TEXT PRIMARY KEY,
                sync_time TEXT NOT NULL,
                operations_synced INTEGER NOT NULL,
                success BOOLEAN NOT NULL,
                error_message TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_sessions_learner ON sessions(learner_id);
            CREATE INDEX IF NOT EXISTS idx_responses_session ON responses(session_id);
            CREATE INDEX IF NOT EXISTS idx_sync_queue_priority ON sync_queue(priority DESC);
            "#,
        )?;

        Ok(())
    }

    /// Save session locally
    pub async fn save_session(&self, session: &Session) -> Result<(), StorageError> {
        let db = self.db.write().await;

        let topology_json = serde_json::to_string(&session.topology)?;
        let summary_json = session
            .summary
            .as_ref()
            .map(|s| serde_json::to_string(s))
            .transpose()?;

        db.execute(
            "INSERT OR REPLACE INTO sessions
             (id, learner_id, topology_type, topology_data, start_time, end_time, status, summary, sync_status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'pending')",
            rusqlite::params![
                session.id,
                session.learner_id,
                session.topology_type,
                topology_json,
                session.start_time.to_rfc3339(),
                session.end_time.map(|t| t.to_rfc3339()),
                session.status,
                summary_json,
            ],
        )?;

        // Add to sync queue
        self.queue_sync(SyncOperation {
            id: uuid::Uuid::new_v4().to_string(),
            operation_type: OperationType::CreateSession,
            payload: serde_json::to_value(session)?,
            created_at: Utc::now(),
            retry_count: 0,
            priority: SyncPriority::Normal,
        })
        .await?;

        // Update cache
        let mut cache = self.cache.write().await;
        cache.sessions.insert(session.id.clone(), session.clone());

        Ok(())
    }

    /// Save response locally
    pub async fn save_response(
        &self,
        session_id: &str,
        response: &CoreTaskResponse,
    ) -> Result<(), StorageError> {
        let db = self.db.write().await;

        let task_json = serde_json::to_string(&response.task)?;

        db.execute(
            "INSERT INTO responses
             (id, session_id, task_data, response, correct, response_time_ms, timestamp, sync_status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'pending')",
            rusqlite::params![
                uuid::Uuid::new_v4().to_string(),
                session_id,
                task_json,
                response.user_answer,
                response.correct,
                (response.response_time_ms as i64),
                response.timestamp.to_rfc3339(),
            ],
        )?;

        // Add to sync queue with high priority for immediate feedback
        self.queue_sync(SyncOperation {
            id: uuid::Uuid::new_v4().to_string(),
            operation_type: OperationType::SubmitResponse,
            payload: serde_json::json!({
                "session_id": session_id,
                "response": response
            }),
            created_at: Utc::now(),
            retry_count: 0,
            priority: SyncPriority::High,
        })
        .await?;

        // Update cache
        let mut cache = self.cache.write().await;
        cache
            .responses
            .entry(session_id.to_string())
            .or_insert_with(Vec::new)
            .push(response.clone());

        Ok(())
    }

    /// Load session from local storage
    pub async fn load_session(&self, session_id: &str) -> Result<Option<Session>, StorageError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(session) = cache.sessions.get(session_id) {
                return Ok(Some(session.clone()));
            }
        }

        // Load from database
        let db = self.db.read().await;
        let mut stmt = db.prepare(
            "SELECT id, learner_id, topology_type, topology_data, start_time, end_time, status, summary
             FROM sessions WHERE id = ?1"
        )?;

        let session = stmt
            .query_row([session_id], |row| {
                let topology_json: String = row.get(3)?;
                let summary_json: Option<String> = row.get(7)?;

                Ok(Session {
                    id: row.get(0)?,
                    learner_id: row.get(1)?,
                    topology_type: row.get(2)?,
                    topology: serde_json::from_str(&topology_json).ok(),
                    start_time: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                4,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })?
                        .with_timezone(&Utc),
                    end_time: row
                        .get::<_, Option<String>>(5)?
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&Utc)),
                    status: row.get(6)?,
                    summary: summary_json.and_then(|s| serde_json::from_str(&s).ok()),
                    responses: Vec::new(), // Load separately if needed
                })
            })
            .optional()?;

        // Update cache if found
        if let Some(ref session) = session {
            let mut cache = self.cache.write().await;
            cache
                .sessions
                .insert(session_id.to_string(), session.clone());
        }

        Ok(session)
    }

    /// Load all responses for a session
    pub async fn load_responses(
        &self,
        session_id: &str,
    ) -> Result<Vec<CoreTaskResponse>, StorageError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(responses) = cache.responses.get(session_id) {
                return Ok(responses.clone());
            }
        }

        // Load from database
        let db = self.db.read().await;
        let mut stmt = db.prepare(
            "SELECT task_data, response, correct, response_time_ms, timestamp
             FROM responses WHERE session_id = ?1 ORDER BY timestamp",
        )?;

        let responses = stmt
            .query_map([session_id], |row| {
                let task_json: String = row.get(0)?;

                Ok(CoreTaskResponse {
                    task: serde_json::from_str(&task_json).map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?,
                    user_answer: row.get::<_, String>(1)?,
                    correct: row.get::<_, bool>(2)?,
                    response_time_ms: {
                        let v: i64 = row.get(3)?;
                        v as u128
                    },
                    timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                4,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })?
                        .with_timezone(&Utc),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        // Update cache
        let mut cache = self.cache.write().await;
        cache
            .responses
            .insert(session_id.to_string(), responses.clone());

        Ok(responses)
    }

    /// Queue operation for sync
    async fn queue_sync(&self, operation: SyncOperation) -> Result<(), StorageError> {
        let mut queue = self.sync_queue.write().await;

        // Save to database for persistence
        let db = self.db.write().await;
        db.execute(
            "INSERT INTO sync_queue (id, operation_type, payload, created_at, retry_count, priority)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                operation.id,
                serde_json::to_string(&operation.operation_type)?,
                serde_json::to_string(&operation.payload)?,
                operation.created_at.to_rfc3339(),
                operation.retry_count,
                operation.priority as i32,
            ],
        )?;

        // Add to in-memory queue
        queue.pending_operations.push_back(operation);

        Ok(())
    }

    /// Perform sync with server
    pub async fn sync(&self, api_client: &impl ApiClient) -> Result<SyncResult, StorageError> {
        let mut queue = self.sync_queue.write().await;

        if queue.sync_in_progress {
            return Ok(SyncResult {
                synced_count: 0,
                failed_count: 0,
                remaining_count: queue.pending_operations.len(),
            });
        }

        queue.sync_in_progress = true;
        let start_time = Utc::now();

        let mut synced = 0;
        let mut failed = 0;

        // Sort by priority
        let mut operations: Vec<_> = queue.pending_operations.drain(..).collect();
        operations.sort_by_key(|op| std::cmp::Reverse(op.priority));

        for mut operation in operations {
            match self.sync_operation(&operation, api_client).await {
                Ok(_) => {
                    synced += 1;
                    // Remove from database
                    let db = self.db.write().await;
                    db.execute("DELETE FROM sync_queue WHERE id = ?1", [&operation.id])?;
                }
                Err(e) => {
                    failed += 1;
                    operation.retry_count += 1;

                    if operation.retry_count < 3 {
                        // Re-queue for retry
                        queue.pending_operations.push_back(operation);
                    } else {
                        // Move to failed operations
                        queue.failed_operations.push(FailedOperation {
                            operation,
                            error: e.to_string(),
                            failed_at: Utc::now(),
                            permanent_failure: false,
                        });
                    }
                }
            }
        }

        queue.last_sync = Some(Utc::now());
        queue.sync_in_progress = false;

        // Record sync history
        let db = self.db.write().await;
        db.execute(
            "INSERT INTO sync_history (id, sync_time, operations_synced, success, error_message)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                uuid::Uuid::new_v4().to_string(),
                start_time.to_rfc3339(),
                synced,
                failed == 0,
                if failed > 0 {
                    Some(format!("{} operations failed", failed))
                } else {
                    None
                },
            ],
        )?;

        Ok(SyncResult {
            synced_count: synced,
            failed_count: failed,
            remaining_count: queue.pending_operations.len(),
        })
    }

    /// Sync individual operation
    async fn sync_operation(
        &self,
        operation: &SyncOperation,
        api_client: &impl ApiClient,
    ) -> Result<(), StorageError> {
        match operation.operation_type {
            OperationType::CreateSession => {
                let session: Session = serde_json::from_value(operation.payload.clone())?;
                api_client.create_session(&session).await?;
            }
            OperationType::SubmitResponse => {
                let data: serde_json::Value = operation.payload.clone();
                let session_id = data["session_id"]
                    .as_str()
                    .ok_or(StorageError::InvalidData)?;
                let response: CoreTaskResponse = serde_json::from_value(data["response"].clone())?;
                api_client.submit_response(session_id, &response).await?;
            }
            _ => {
                // Handle other operation types
            }
        }

        Ok(())
    }

    /// Get sync status
    pub async fn get_sync_status(&self) -> SyncStatus {
        let queue = self.sync_queue.read().await;

        SyncStatus {
            online: true, // This should check actual connectivity
            last_sync: queue.last_sync,
            pending_count: queue.pending_operations.len(),
            failed_count: queue.failed_operations.len(),
            sync_progress: if queue.sync_in_progress {
                Some(SyncProgress {
                    current: 0,
                    total: queue.pending_operations.len(),
                    message: "Syncing...".to_string(),
                })
            } else {
                None
            },
        }
    }

    /// Clear old data based on retention policy
    pub async fn cleanup(&self) -> Result<(), StorageError> {
        let db = self.db.write().await;
        let cutoff = Utc::now() - chrono::Duration::days(self.config.retention_days as i64);

        db.execute(
            "DELETE FROM responses WHERE timestamp < ?1",
            [cutoff.to_rfc3339()],
        )?;

        db.execute(
            "DELETE FROM sessions WHERE end_time < ?1 AND status = 'completed'",
            [cutoff.to_rfc3339()],
        )?;

        Ok(())
    }

    /// Export all local data
    pub async fn export_all(&self) -> Result<serde_json::Value, StorageError> {
        let db = self.db.read().await;

        let sessions: Vec<serde_json::Value> = db
            .prepare("SELECT * FROM sessions")?
            .query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, String>(0)?,
                    "learner_id": row.get::<_, String>(1)?,
                    "topology_type": row.get::<_, String>(2)?,
                    "start_time": row.get::<_, String>(4)?,
                    "status": row.get::<_, String>(6)?,
                }))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(serde_json::json!({
            "export_time": Utc::now(),
            "sessions": sessions,
            "config": self.config,
        }))
    }
}

/// API client trait for sync operations
#[async_trait::async_trait]
pub trait ApiClient: Send + Sync {
    async fn create_session(&self, session: &Session) -> Result<(), ApiError>;
    async fn submit_response(
        &self,
        session_id: &str,
        response: &CoreTaskResponse,
    ) -> Result<(), ApiError>;
}

/// Sync result
#[derive(Debug)]
pub struct SyncResult {
    pub synced_count: usize,
    pub failed_count: usize,
    pub remaining_count: usize,
}

/// Storage errors
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("API error: {0}")]
    Api(#[from] ApiError),

    #[error("Invalid data")]
    InvalidData,

    #[error("Storage full")]
    StorageFull,
}

/// API errors
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Authentication error")]
    Authentication,

    #[error("Server error: {0}")]
    Server(String),
}

/// Network connectivity monitor
pub struct ConnectivityMonitor {
    online: Arc<RwLock<bool>>,
    // Listeners do not need to be Send/Sync because callbacks are executed
    // on the same task that sets the status. Relaxing this bound avoids
    // unnecessary Send requirements for captured values (like local storage).
    listeners: Arc<RwLock<Vec<Box<dyn Fn(bool)>>>>,
}

impl ConnectivityMonitor {
    pub fn new() -> Self {
        Self {
            online: Arc::new(RwLock::new(true)),
            listeners: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn is_online(&self) -> bool {
        *self.online.read().await
    }

    pub async fn set_online(&self, online: bool) {
        let mut status = self.online.write().await;
        if *status != online {
            *status = online;

            // Notify listeners
            let listeners = self.listeners.read().await;
            for listener in listeners.iter() {
                listener(online);
            }
        }
    }

    pub async fn add_listener<F>(&self, listener: F)
    where
        F: Fn(bool) + 'static,
    {
        let mut listeners = self.listeners.write().await;
        listeners.push(Box::new(listener));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_offline_storage_creation() {
        let temp_dir = TempDir::new().unwrap();
        let storage = OfflineStorage::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();

        assert!(storage.get_sync_status().await.pending_count == 0);
    }

    #[tokio::test]
    async fn test_save_and_load_session() {
        let temp_dir = TempDir::new().unwrap();
        let storage = OfflineStorage::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();

        let session = Session {
            id: "test-123".to_string(),
            learner_id: "learner-456".to_string(),
            topology_type: "alphabet".to_string(),
            topology: None,
            start_time: Utc::now(),
            end_time: None,
            status: "active".to_string(),
            summary: None,
            responses: Vec::new(),
        };

        storage.save_session(&session).await.unwrap();

        let loaded = storage.load_session("test-123").await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().id, "test-123");
    }
}
