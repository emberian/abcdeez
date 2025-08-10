use crate::{
    cache, cache::ConnectionManager, config::Config, db::DbPool, services::BatchJobService,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    // For legacy code paths that referenced Redis explicitly.
    // We back this with the in-memory cache manager to avoid external deps.
    pub redis_conn: ConnectionManager,
    // New preferred field name for cache usage.
    pub cache_conn: ConnectionManager,
    pub config: Arc<Config>,
    pub batch_job_service: Arc<BatchJobService>,
}

impl AppState {
    // Primary constructor: provide a single cache connection manager
    // and it will be used for both `redis_conn` and `cache_conn` for compatibility.
    pub fn new(db_pool: DbPool, cache_conn: ConnectionManager, config: Arc<Config>) -> Self {
        let batch_job_service = Arc::new(BatchJobService::new(
            Arc::new(db_pool.clone()),
            cache_conn.clone(),
            config.clone(),
        ));

        Self {
            db_pool,
            redis_conn: cache_conn.clone(),
            cache_conn,
            config,
            batch_job_service,
        }
    }

    // Convenience constructor for local/dev: creates an in-memory cache manager.
    pub fn new_in_memory(db_pool: DbPool, config: Config) -> Self {
        let conn = cache::connection_manager();
        Self::new(db_pool, conn, Arc::new(config))
    }
}
