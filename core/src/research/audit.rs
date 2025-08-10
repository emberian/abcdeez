use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

/// Comprehensive audit trail system for research compliance and data integrity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrailManager {
    pub config: AuditConfiguration,
    pub events: Vec<AuditEvent>,
    pub session_id: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfiguration {
    pub level: AuditLevel,
    pub include_system_events: bool,
    pub include_user_events: bool,
    pub include_data_events: bool,
    pub retention_days: u32,
    pub anonymize_data: bool,
    pub hash_sensitive_fields: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditLevel {
    Minimal,  // Only critical events
    Standard, // Common events for compliance
    Detailed, // All events for research
    Forensic, // Everything including debug info
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
    pub actor: Actor,
    pub resource: Resource,
    pub operation: Operation,
    pub outcome: Outcome,
    pub details: HashMap<String, String>,
    pub session_context: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    Authentication,
    Authorization,
    DataAccess,
    DataModification,
    SystemEvent,
    UserAction,
    ConfigurationChange,
    SecurityEvent,
    PerformanceEvent,
    ErrorEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    pub id: String,
    pub actor_type: ActorType,
    pub name: Option<String>,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActorType {
    User,
    System,
    Service,
    Administrator,
    Researcher,
    Participant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub id: String,
    pub resource_type: String,
    pub name: Option<String>,
    pub classification: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    Create,
    Read,
    Update,
    Delete,
    Execute,
    Login,
    Logout,
    Export,
    Import,
    Backup,
    Restore,
    Configure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Outcome {
    Success,
    Failure,
    Partial,
    Cancelled,
    Timeout,
    Error,
}

impl Default for AuditConfiguration {
    fn default() -> Self {
        Self {
            level: AuditLevel::Standard,
            include_system_events: true,
            include_user_events: true,
            include_data_events: true,
            retention_days: 2555, // 7 years for research compliance
            anonymize_data: false,
            hash_sensitive_fields: true,
        }
    }
}

impl AuditTrailManager {
    pub fn new(config: Option<AuditConfiguration>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            events: Vec::new(),
            session_id: None,
            metadata: HashMap::new(),
        }
    }

    #[instrument(level = "info", fields(session_id = %session_id))]
    pub fn start_session(&mut self, session_id: String) {
        info!(session_id = %session_id, "Starting audit session");

        self.session_id = Some(session_id.clone());

        let event_id = self.log_event(
            EventType::SystemEvent,
            Actor {
                id: "system".to_string(),
                actor_type: ActorType::System,
                name: Some("Audit System".to_string()),
                roles: vec!["auditor".to_string()],
            },
            Resource {
                id: session_id.clone(),
                resource_type: "session".to_string(),
                name: Some("Audit Session".to_string()),
                classification: None,
            },
            Operation::Create,
            Outcome::Success,
            HashMap::new(),
        );

        info!(
            session_id = %session_id,
            event_id = %event_id,
            audit_level = ?self.config.level,
            "Audit session started successfully"
        );
    }

    #[instrument(level = "debug", fields(
        event_type = ?event_type,
        actor_id = %actor.id,
        resource_id = %resource.id,
        operation = ?operation,
        outcome = ?outcome
    ))]
    pub fn log_event(
        &mut self,
        event_type: EventType,
        actor: Actor,
        resource: Resource,
        operation: Operation,
        outcome: Outcome,
        details: HashMap<String, String>,
    ) -> String {
        let event_id = Uuid::new_v4().to_string();

        debug!(
            event_id = %event_id,
            actor_type = ?actor.actor_type,
            resource_type = %resource.resource_type,
            session_context = ?self.session_id,
            detail_count = details.len(),
            "Creating audit event"
        );

        let event = AuditEvent {
            id: event_id.clone(),
            timestamp: Utc::now(),
            event_type: event_type.clone(),
            actor,
            resource,
            operation: operation.clone(),
            outcome: outcome.clone(),
            details,
            session_context: self.session_id.clone(),
            ip_address: None, // Would be filled by network layer
            user_agent: None, // Would be filled by web layer
            checksum: None,   // Could implement integrity checking
        };

        // Apply audit level filtering
        let should_log = self.should_log_event(&event);
        debug!(should_log, audit_level = ?self.config.level, "Audit level filtering applied");

        if should_log {
            self.events.push(event);
            info!(
                event_id = %event_id,
                event_type = ?event_type,
                operation = ?operation,
                outcome = ?outcome,
                total_events = self.events.len(),
                "Audit event logged"
            );
        } else {
            debug!(
                event_id = %event_id,
                event_type = ?event_type,
                "Audit event filtered out by audit level"
            );
        }

        event_id
    }

    fn should_log_event(&self, event: &AuditEvent) -> bool {
        match self.config.level {
            AuditLevel::Minimal => matches!(
                event.event_type,
                EventType::Authentication | EventType::Authorization | EventType::SecurityEvent
            ),
            AuditLevel::Standard => !matches!(event.event_type, EventType::PerformanceEvent),
            AuditLevel::Detailed | AuditLevel::Forensic => true,
        }
    }

    pub fn log_user_action(
        &mut self,
        user_id: &str,
        action: &str,
        resource_id: &str,
        outcome: Outcome,
    ) -> String {
        let mut details = HashMap::new();
        details.insert("action".to_string(), action.to_string());

        self.log_event(
            EventType::UserAction,
            Actor {
                id: user_id.to_string(),
                actor_type: ActorType::User,
                name: None,
                roles: vec!["participant".to_string()],
            },
            Resource {
                id: resource_id.to_string(),
                resource_type: "user_interface".to_string(),
                name: Some(action.to_string()),
                classification: None,
            },
            Operation::Execute,
            outcome,
            details,
        )
    }

    pub fn log_data_access(
        &mut self,
        user_id: &str,
        data_type: &str,
        data_id: &str,
        operation: Operation,
        outcome: Outcome,
    ) -> String {
        let mut details = HashMap::new();
        details.insert("data_type".to_string(), data_type.to_string());

        self.log_event(
            EventType::DataAccess,
            Actor {
                id: user_id.to_string(),
                actor_type: ActorType::User,
                name: None,
                roles: vec!["participant".to_string()],
            },
            Resource {
                id: data_id.to_string(),
                resource_type: data_type.to_string(),
                name: None,
                classification: Some("research_data".to_string()),
            },
            operation,
            outcome,
            details,
        )
    }

    pub fn log_system_event(
        &mut self,
        component: &str,
        event_description: &str,
        outcome: Outcome,
    ) -> String {
        let mut details = HashMap::new();
        details.insert("component".to_string(), component.to_string());
        details.insert("description".to_string(), event_description.to_string());

        self.log_event(
            EventType::SystemEvent,
            Actor {
                id: "system".to_string(),
                actor_type: ActorType::System,
                name: Some(component.to_string()),
                roles: vec!["system".to_string()],
            },
            Resource {
                id: component.to_string(),
                resource_type: "system_component".to_string(),
                name: Some(component.to_string()),
                classification: None,
            },
            Operation::Execute,
            outcome,
            details,
        )
    }

    pub fn get_events_for_session(&self, session_id: &str) -> Vec<&AuditEvent> {
        self.events
            .iter()
            .filter(|event| {
                event
                    .session_context
                    .as_ref()
                    .map_or(false, |s| s == session_id)
            })
            .collect()
    }

    pub fn get_events_for_user(&self, user_id: &str) -> Vec<&AuditEvent> {
        self.events
            .iter()
            .filter(|event| event.actor.id == user_id)
            .collect()
    }

    pub fn get_events_by_type(&self, event_type: &EventType) -> Vec<&AuditEvent> {
        self.events
            .iter()
            .filter(|event| {
                std::mem::discriminant(&event.event_type) == std::mem::discriminant(event_type)
            })
            .collect()
    }

    pub fn export_audit_trail(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.events)
    }

    pub fn generate_compliance_report(&self) -> String {
        let total_events = self.events.len();
        let successful_operations = self
            .events
            .iter()
            .filter(|e| matches!(e.outcome, Outcome::Success))
            .count();
        let failed_operations = self
            .events
            .iter()
            .filter(|e| matches!(e.outcome, Outcome::Failure | Outcome::Error))
            .count();

        let unique_users = self
            .events
            .iter()
            .filter(|e| matches!(e.actor.actor_type, ActorType::User | ActorType::Participant))
            .map(|e| &e.actor.id)
            .collect::<std::collections::HashSet<_>>()
            .len();

        format!(
            "Audit Trail Compliance Report\n\
            ================================\n\
            Total Events: {}\n\
            Successful Operations: {}\n\
            Failed Operations: {}\n\
            Unique Users: {}\n\
            Audit Level: {:?}\n\
            Session ID: {:?}\n\
            Report Generated: {}\n",
            total_events,
            successful_operations,
            failed_operations,
            unique_users,
            self.config.level,
            self.session_id,
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )
    }

    pub fn cleanup_old_events(&mut self, days: u32) {
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);
        self.events.retain(|event| event.timestamp > cutoff);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_trail_creation() {
        let mut audit = AuditTrailManager::new(None);
        assert_eq!(audit.events.len(), 0);
        assert!(audit.session_id.is_none());
    }

    #[test]
    fn test_event_logging() {
        let mut audit = AuditTrailManager::new(None);

        let event_id =
            audit.log_user_action("user123", "button_click", "submit_button", Outcome::Success);

        assert_eq!(audit.events.len(), 1);
        assert!(!event_id.is_empty());
        assert_eq!(audit.events[0].actor.id, "user123");
    }

    #[test]
    fn test_session_filtering() {
        let mut audit = AuditTrailManager::new(None);
        audit.start_session("session1".to_string());

        audit.log_user_action("user1", "action1", "resource1", Outcome::Success);

        let events = audit.get_events_for_session("session1");
        assert!(events.len() >= 1); // At least the session start event
    }
}
