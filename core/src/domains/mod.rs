//! # Domain-Specific Implementations
//! 
//! This module provides specialized implementations for different learning domains:
//! 
//! - [`MusicTheory`]: Musical note sequences and harmonic relationships
//! - Alphabet learning with phonetic and orthographic patterns
//! - Mathematical sequences with numerical relationships
//! - Spatial navigation and route learning
//! 
//! ## Supported Domains
//! 
//! ### Music
//! - **Note Sequences**: Chromatic, diatonic, pentatonic scales
//! - **Interval Training**: Perfect 5ths, octaves, chord progressions  
//! - **Key Signatures**: Major/minor keys with appropriate accidentals
//! - **Rhythm Patterns**: Temporal sequence learning
//! 
//! ### Alphabet  
//! - **Letter Sequences**: Forward/backward recitation
//! - **Phonetic Patterns**: Vowel/consonant alternations
//! - **Orthographic Rules**: Spelling patterns and exceptions
//! - **Multilingual**: Support for different alphabetic systems
//! 
//! ### Mathematics
//! - **Number Sequences**: Arithmetic, geometric, Fibonacci
//! - **Skip Counting**: By 2s, 5s, 10s, etc.
//! - **Prime Numbers**: Special mathematical sequences
//! - **Modular Arithmetic**: Circular number systems
//! 
//! ### Spatial Navigation
//! - **Route Learning**: Optimal path finding
//! - **Landmark Recognition**: Spatial reference points
//! - **Cognitive Maps**: Mental representation of spatial layouts
//! - **Wayfinding Strategies**: Different navigation approaches
//! 
//! ## Quick Start
//! 
//! ```rust
//! use abcdeez_core::domains::{MusicTheory, MusicStructure};
//! 
//! // Create a music learning domain
//! let music = MusicTheory::new(MusicStructure::ChromaticScale);
//! let topology = music.create_topology();
//! 
//! // Generate music-specific tasks
//! let task_generator = music.create_task_generator(topology);
//! ```

pub mod music;

// Re-export main types
pub use music::{MusicStructure, MusicTaskGenerator, MusicTheory};