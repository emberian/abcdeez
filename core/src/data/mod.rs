//! # Data Management & Persistence
//! 
//! This module handles data export, import, storage, and format conversion:
//! 
//! - [`LearnerDataExport`]: Export learner data in various formats
//! - [`PopulationAnalyzer`]: Analyze data across multiple learners
//! - Data import/export in CSV, JSON, HDF5, R formats
//! - Integration with statistical analysis software
//! 
//! ## Export Formats
//! 
//! - **CSV**: For Excel and general statistical software
//! - **JSON**: For web applications and JavaScript analysis  
//! - **HDF5**: For large datasets and scientific computing
//! - **R Data**: Direct integration with R statistical environment
//! - **SPSS/SAS**: For traditional statistical analysis packages
//! 
//! ## Data Types
//! 
//! - **Trial-level**: Individual responses with timestamps
//! - **Session-level**: Aggregated session metrics
//! - **Learner-level**: Individual difference measures
//! - **Population-level**: Cross-learner comparisons
//! 
//! ## Privacy & Compliance
//! 
//! - **De-identification**: Remove/hash identifying information
//! - **GDPR Compliance**: Right to erasure, data portability
//! - **IRB Requirements**: Structured data for research approval
//! - **Audit Trails**: Track data access and modifications
//! 
//! ## Quick Start
//! 
//! ```rust
//! use abcdeez_core::data::{LearnerDataExport, ExportFormat};
//! 
//! let exporter = LearnerDataExport::new();
//! 
//! // Export single learner data
//! exporter.export_learner_data(
//!     &learner_model, 
//!     "output.csv", 
//!     ExportFormat::CSV
//! )?;
//! 
//! // Export population analysis
//! exporter.export_population_summary(
//!     &learners, 
//!     "population_report.json"
//! )?;
//! ```

pub mod export;

// Re-export main types
pub use export::{
    LearnerDataExport, PopulationAnalyzer, SessionData,
    SessionSummary, PerformancePoint, ErrorAnalysis,
    ModelSnapshot, NodeEmbeddingExport, ChunkBoundaryExport, ExportMetadata
};