//! # Research Infrastructure
//! 
//! This module provides comprehensive research infrastructure for academic compliance,
//! citation management, protocol versioning, and audit trails:
//! 
//! - [`CitationManager`]: Academic reference and citation management
//! - [`IRBComplianceGenerator`]: IRB/ethics board compliance documentation
//! - [`ProtocolVersionControl`]: Research protocol versioning and change tracking
//! - [`AuditTrailManager`]: Complete audit trail for research activities
//! 
//! ## Citation Management
//! 
//! - **Multiple Styles**: APA, Chicago, MLA, Harvard, Vancouver, IEEE
//! - **Reference Types**: Journal articles, books, conferences, software
//! - **Methodology Tracking**: Link methods to specific references
//! - **Automatic Formatting**: Generate properly formatted bibliographies
//! 
//! ## IRB/Ethics Compliance
//! 
//! - **Consent Templates**: Generate informed consent forms
//! - **Study Summaries**: Structured summaries for IRB submissions
//! - **Risk Assessment**: Automated risk level classification
//! - **Compliance Tracking**: Monitor adherence to approved protocols
//! 
//! ## Protocol Versioning
//! 
//! - **Semantic Versioning**: Major.Minor.Patch version tracking
//! - **Change Documentation**: Detailed change logs and rationales
//! - **Collaboration**: Multi-researcher protocol development
//! - **Approval Workflows**: Track IRB approvals across versions
//! 
//! ## Audit Trails
//! 
//! - **Complete Logging**: All system activities with timestamps
//! - **User Attribution**: Track who performed each action
//! - **Data Integrity**: Immutable log entries with cryptographic hashes
//! - **Compliance Reporting**: Generate audit reports for regulatory review
//! 
//! ## Quick Start
//! 
//! ```rust
//! use abcdeez_core::research::{CitationManager, BibliographyStyle, IRBComplianceGenerator};
//! 
//! // Citation management
//! let mut citations = CitationManager::new(BibliographyStyle::APA);
//! citations.add_reference(reference);
//! let bibliography = citations.generate_bibliography();
//! 
//! // IRB compliance
//! let irb_gen = IRBComplianceGenerator::new();
//! let application = irb_gen.generate_irb_application(&study_config)?;
//! ```

pub mod citations;
pub mod compliance;
pub mod versioning;
pub mod audit;
pub mod preregistration;
pub mod protocol_versioning;

// Re-export main types
pub use citations::{
    CitationManager, BibliographyStyle, Reference, ReferenceType,
    Author, Publication, MethodologyReport
};
pub use compliance::{
    IRBComplianceGenerator, IRBApplication, ConsentTemplate, StudySummary
};
pub use versioning::{
    ProtocolVersionControl, ProtocolVersion, ProtocolChange,
    ProtocolSnapshot, CollaboratorRole, ProtocolVersionManager
};
pub use audit::{
    AuditTrailManager, AuditConfiguration, EventType, Actor,
    ActorType, Operation, Outcome, Resource, AuditLevel
};