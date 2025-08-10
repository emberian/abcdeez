use crate::core::learner::OperationType;
use crate::tasks::types::{Task, TaskType};
use crate::core::topology::{Edge, Node, Topology, TopologyType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MusicStructure {
    ChromaticScale,
    CircleOfFifths,
    MajorScale(String), // Root note
    MinorScale(String, MinorType),
    ChordProgression(ProgressionType),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MinorType {
    Natural,
    Harmonic,
    Melodic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProgressionType {
    IVVi,      // I-IV-V-I
    IIVi,      // ii-V-I
    VIIVIIIVi, // vi-ii-V-I
    Blues,     // I-I-I-I-IV-IV-I-I-V-IV-I-V
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicTheory {
    pub structure: MusicStructure,
    pub topology: Topology,
    pub note_attributes: HashMap<String, NoteAttributes>,
    pub intervals: HashMap<(String, String), Interval>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteAttributes {
    pub pitch_class: u8,            // 0-11 (C=0, C#=1, etc.)
    pub frequency: f64,             // Hz (A4 = 440)
    pub enharmonic: Option<String>, // Alternative spelling
    pub color: String,              // For synesthesia training
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interval {
    pub semitones: u8,
    pub name: String,
    pub quality: IntervalQuality,
    pub consonance: f64, // 0.0 (dissonant) to 1.0 (consonant)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntervalQuality {
    Perfect,
    Major,
    Minor,
    Augmented,
    Diminished,
}

impl MusicTheory {
    pub fn chromatic_scale() -> Self {
        let notes = vec![
            ("C", 0, None),
            ("C#", 1, Some("Db")),
            ("D", 2, None),
            ("D#", 3, Some("Eb")),
            ("E", 4, None),
            ("F", 5, None),
            ("F#", 6, Some("Gb")),
            ("G", 7, None),
            ("G#", 8, Some("Ab")),
            ("A", 9, None),
            ("A#", 10, Some("Bb")),
            ("B", 11, None),
        ];

        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut node_map = HashMap::new();
        let mut note_attributes = HashMap::new();

        for (i, (note, pitch_class, enharmonic)) in notes.iter().enumerate() {
            let id = format!("note_{}", i);
            nodes.push(Node {
                id: id.clone(),
                label: note.to_string(),
                position: i as f64,
            });
            node_map.insert(id.clone(), i);

            // Create edge to next note (cyclic)
            let next_idx = (i + 1) % 12;
            edges.push(Edge {
                from: id.clone(),
                to: format!("note_{}", next_idx),
                weight: 1.0, // semitone
            });

            // Calculate frequency (A4 = 440Hz)
            let a4_position = 9; // A is at position 9
            let _octave = 4;
            let semitones_from_a4 = *pitch_class as i32 - a4_position;
            let frequency = 440.0 * 2.0_f64.powf(semitones_from_a4 as f64 / 12.0);

            note_attributes.insert(
                note.to_string(),
                NoteAttributes {
                    pitch_class: *pitch_class,
                    frequency,
                    enharmonic: enharmonic.as_ref().map(|s| s.to_string()),
                    color: Self::note_to_color(note),
                },
            );
        }

        let topology = Topology {
            topology_type: TopologyType::Cyclic,
            nodes,
            edges,
            node_map,
        };

        let intervals = Self::generate_interval_map(&topology);

        MusicTheory {
            structure: MusicStructure::ChromaticScale,
            topology,
            note_attributes,
            intervals,
        }
    }

    pub fn circle_of_fifths() -> Self {
        let notes_order = vec![
            "C", "G", "D", "A", "E", "B", "F#", "C#", "G#", "D#", "A#", "F",
        ];

        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut node_map = HashMap::new();
        let mut note_attributes = HashMap::new();

        for (i, note) in notes_order.iter().enumerate() {
            let id = format!("note_{}", i);
            nodes.push(Node {
                id: id.clone(),
                label: note.to_string(),
                position: (i as f64 * std::f64::consts::TAU) / 12.0,
            });
            node_map.insert(id.clone(), i);

            // Edge to next fifth (7 semitones)
            let next_idx = (i + 1) % 12;
            edges.push(Edge {
                from: id.clone(),
                to: format!("note_{}", next_idx),
                weight: 7.0, // perfect fifth
            });

            // Also add edge to relative minor (down 3 semitones)
            let relative_minor_idx = (i + 9) % 12; // -3 = +9 mod 12
            edges.push(Edge {
                from: id.clone(),
                to: format!("note_{}", relative_minor_idx),
                weight: 3.0, // minor third down
            });

            note_attributes.insert(
                note.to_string(),
                NoteAttributes {
                    pitch_class: Self::note_to_pitch_class(note),
                    frequency: Self::note_to_frequency(note, 4),
                    enharmonic: Self::get_enharmonic(note),
                    color: Self::note_to_color(note),
                },
            );
        }

        let topology = Topology {
            topology_type: TopologyType::Cyclic,
            nodes,
            edges,
            node_map,
        };

        let intervals = Self::generate_interval_map(&topology);

        MusicTheory {
            structure: MusicStructure::CircleOfFifths,
            topology,
            note_attributes,
            intervals,
        }
    }

    pub fn major_scale(root: &str) -> Self {
        let root_pc = Self::note_to_pitch_class(root);
        let intervals = vec![0, 2, 4, 5, 7, 9, 11]; // W-W-H-W-W-W-H pattern

        let scale_notes: Vec<String> = intervals
            .iter()
            .map(|&interval| {
                let pc = (root_pc + interval) % 12;
                Self::pitch_class_to_note(pc, true) // Use sharps in major keys
            })
            .collect();

        Self::create_scale_topology(scale_notes, MusicStructure::MajorScale(root.to_string()))
    }

    pub fn minor_scale(root: &str, minor_type: MinorType) -> Self {
        let root_pc = Self::note_to_pitch_class(root);

        let intervals = match minor_type {
            MinorType::Natural => vec![0, 2, 3, 5, 7, 8, 10], // W-H-W-W-H-W-W
            MinorType::Harmonic => vec![0, 2, 3, 5, 7, 8, 11], // W-H-W-W-H-Aug2-H
            MinorType::Melodic => vec![0, 2, 3, 5, 7, 9, 11], // W-H-W-W-W-W-H (ascending)
        };

        let scale_notes: Vec<String> = intervals
            .iter()
            .map(|&interval| {
                let pc = (root_pc + interval) % 12;
                Self::pitch_class_to_note(pc, false) // Use flats in minor keys
            })
            .collect();

        Self::create_scale_topology(
            scale_notes,
            MusicStructure::MinorScale(root.to_string(), minor_type),
        )
    }

    pub fn chord_progression(prog_type: ProgressionType) -> Self {
        let chords = match prog_type {
            ProgressionType::IVVi => vec!["I", "IV", "V", "I"],
            ProgressionType::IIVi => vec!["ii", "V", "I"],
            ProgressionType::VIIVIIIVi => vec!["vi", "ii", "V", "I"],
            ProgressionType::Blues => vec![
                "I", "I", "I", "I", "IV", "IV", "I", "I", "V", "IV", "I", "V",
            ],
        };

        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut node_map = HashMap::new();

        for (i, chord) in chords.iter().enumerate() {
            let id = format!("chord_{}", i);
            nodes.push(Node {
                id: id.clone(),
                label: chord.to_string(),
                position: i as f64,
            });
            node_map.insert(id.clone(), i);

            if i < chords.len() - 1 {
                edges.push(Edge {
                    from: id.clone(),
                    to: format!("chord_{}", i + 1),
                    weight: 1.0,
                });
            }
        }

        // Add common substitutions and voice leading
        match prog_type {
            ProgressionType::IIVi => {
                // Add tritone substitution for V
                edges.push(Edge {
                    from: "chord_0".to_string(), // ii
                    to: "chord_2".to_string(),   // I (skip V)
                    weight: 1.5,
                });
            }
            _ => {}
        }

        let topology = Topology {
            topology_type: TopologyType::PartialOrder,
            nodes,
            edges,
            node_map,
        };

        MusicTheory {
            structure: MusicStructure::ChordProgression(prog_type),
            topology,
            note_attributes: HashMap::new(),
            intervals: HashMap::new(),
        }
    }

    fn create_scale_topology(notes: Vec<String>, structure: MusicStructure) -> Self {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut node_map = HashMap::new();
        let mut note_attributes = HashMap::new();

        for (i, note) in notes.iter().enumerate() {
            let id = format!("note_{}", i);
            nodes.push(Node {
                id: id.clone(),
                label: note.clone(),
                position: i as f64,
            });
            node_map.insert(id.clone(), i);

            // Linear edges within scale
            if i < notes.len() - 1 {
                edges.push(Edge {
                    from: id.clone(),
                    to: format!("note_{}", i + 1),
                    weight: 1.0,
                });
            }

            // Add octave edge
            if i == 0 {
                edges.push(Edge {
                    from: format!("note_{}", notes.len() - 1),
                    to: id.clone(),
                    weight: 1.0,
                });
            }

            note_attributes.insert(
                note.clone(),
                NoteAttributes {
                    pitch_class: Self::note_to_pitch_class(note),
                    frequency: Self::note_to_frequency(note, 4),
                    enharmonic: Self::get_enharmonic(note),
                    color: Self::note_to_color(note),
                },
            );
        }

        let topology = Topology {
            topology_type: TopologyType::Cyclic,
            nodes,
            edges,
            node_map,
        };

        let intervals = Self::generate_interval_map(&topology);

        MusicTheory {
            structure,
            topology,
            note_attributes,
            intervals,
        }
    }

    fn generate_interval_map(topology: &Topology) -> HashMap<(String, String), Interval> {
        let mut intervals = HashMap::new();

        let interval_names = vec![
            (0, "Unison", IntervalQuality::Perfect, 1.0),
            (1, "Minor 2nd", IntervalQuality::Minor, 0.1),
            (2, "Major 2nd", IntervalQuality::Major, 0.4),
            (3, "Minor 3rd", IntervalQuality::Minor, 0.7),
            (4, "Major 3rd", IntervalQuality::Major, 0.8),
            (5, "Perfect 4th", IntervalQuality::Perfect, 0.9),
            (6, "Tritone", IntervalQuality::Augmented, 0.0),
            (7, "Perfect 5th", IntervalQuality::Perfect, 1.0),
            (8, "Minor 6th", IntervalQuality::Minor, 0.6),
            (9, "Major 6th", IntervalQuality::Major, 0.7),
            (10, "Minor 7th", IntervalQuality::Minor, 0.3),
            (11, "Major 7th", IntervalQuality::Major, 0.2),
            (12, "Octave", IntervalQuality::Perfect, 1.0),
        ];

        for node1 in &topology.nodes {
            for node2 in &topology.nodes {
                if let (Some(attr1), Some(attr2)) = (
                    Self::get_note_pitch_class(&node1.label),
                    Self::get_note_pitch_class(&node2.label),
                ) {
                    let semitones = ((attr2 as i32 - attr1 as i32 + 12) % 12) as u8;

                    if let Some((_, name, quality, consonance)) =
                        interval_names.iter().find(|(s, _, _, _)| *s == semitones)
                    {
                        intervals.insert(
                            (node1.label.clone(), node2.label.clone()),
                            Interval {
                                semitones,
                                name: name.to_string(),
                                quality: quality.clone(),
                                consonance: *consonance,
                            },
                        );
                    }
                }
            }
        }

        intervals
    }

    fn note_to_pitch_class(note: &str) -> u8 {
        match note {
            "C" => 0,
            "C#" | "Db" => 1,
            "D" => 2,
            "D#" | "Eb" => 3,
            "E" => 4,
            "F" => 5,
            "F#" | "Gb" => 6,
            "G" => 7,
            "G#" | "Ab" => 8,
            "A" => 9,
            "A#" | "Bb" => 10,
            "B" => 11,
            _ => 0,
        }
    }

    fn get_note_pitch_class(note: &str) -> Option<u8> {
        Some(Self::note_to_pitch_class(note))
    }

    fn pitch_class_to_note(pc: u8, use_sharps: bool) -> String {
        let sharps = vec![
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];
        let flats = vec![
            "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B",
        ];

        let notes = if use_sharps { sharps } else { flats };
        notes[pc as usize % 12].to_string()
    }

    fn note_to_frequency(note: &str, octave: i32) -> f64 {
        let pc = Self::note_to_pitch_class(note);
        let a4_pitch = 9; // A = 9
        let semitones_from_a4 = pc as i32 - a4_pitch + (octave - 4) * 12;
        440.0 * 2.0_f64.powf(semitones_from_a4 as f64 / 12.0)
    }

    fn get_enharmonic(note: &str) -> Option<String> {
        match note {
            "C#" => Some("Db".to_string()),
            "Db" => Some("C#".to_string()),
            "D#" => Some("Eb".to_string()),
            "Eb" => Some("D#".to_string()),
            "F#" => Some("Gb".to_string()),
            "Gb" => Some("F#".to_string()),
            "G#" => Some("Ab".to_string()),
            "Ab" => Some("G#".to_string()),
            "A#" => Some("Bb".to_string()),
            "Bb" => Some("A#".to_string()),
            _ => None,
        }
    }

    fn note_to_color(note: &str) -> String {
        // Scriabin's color associations
        match note {
            "C" => "Red",
            "C#" | "Db" => "Violet",
            "D" => "Yellow",
            "D#" | "Eb" => "Glint of steel",
            "E" => "Sky blue",
            "F" => "Deep red",
            "F#" | "Gb" => "Bright blue",
            "G" => "Orange",
            "G#" | "Ab" => "Purple",
            "A" => "Green",
            "A#" | "Bb" => "Glint of steel",
            "B" => "Soft blue",
            _ => "Unknown",
        }
        .to_string()
    }
}

// Music-specific task generator
pub struct MusicTaskGenerator {
    theory: MusicTheory,
}

impl MusicTaskGenerator {
    pub fn new(theory: MusicTheory) -> Self {
        MusicTaskGenerator { theory }
    }

    pub fn generate_interval_task(&self, from: String, to: String) -> Task {
        let default_interval = Interval {
            semitones: 0,
            name: "Unknown".to_string(),
            quality: IntervalQuality::Major,
            consonance: 0.5,
        };

        let interval = self
            .theory
            .intervals
            .get(&(from.clone(), to.clone()))
            .unwrap_or(&default_interval);

        let prompt = format!("What interval is between {} and {}?", from, to);
        let correct_answer = interval.name.clone();

        let options = vec![
            "Minor 2nd".to_string(),
            "Major 2nd".to_string(),
            "Minor 3rd".to_string(),
            "Major 3rd".to_string(),
            "Perfect 4th".to_string(),
            "Perfect 5th".to_string(),
            correct_answer.clone(),
        ];

        let mut unique_options: Vec<String> = options
            .into_iter()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .take(4)
            .collect();
        unique_options.sort();

        Task {
            task_type: TaskType::PairwiseOrder { a: from, b: to },
            prompt,
            correct_answer,
            options: unique_options,
            difficulty: 0.5 + (interval.semitones as f64 / 24.0),
            operation: OperationType::PairwiseOrder,
        }
    }

    pub fn generate_scale_degree_task(&self, scale_degree: usize) -> Task {
        let note = self
            .theory
            .topology
            .nodes
            .get(scale_degree - 1)
            .map(|n| n.label.clone())
            .unwrap_or("Unknown".to_string());

        let prompt = format!("What is scale degree {} in this scale?", scale_degree);

        let options: Vec<String> = self
            .theory
            .topology
            .nodes
            .iter()
            .map(|n| n.label.clone())
            .collect();

        Task {
            task_type: TaskType::Index { item: note.clone() },
            prompt,
            correct_answer: note,
            options,
            difficulty: 0.3 + (scale_degree as f64 / 14.0),
            operation: OperationType::Index,
        }
    }

    pub fn generate_chord_progression_task(&self, position: usize) -> Task {
        let chord = self
            .theory
            .topology
            .nodes
            .get(position)
            .map(|n| n.label.clone())
            .unwrap_or("Unknown".to_string());

        let next_chord = self
            .theory
            .topology
            .nodes
            .get(position + 1)
            .map(|n| n.label.clone())
            .unwrap_or("End".to_string());

        let prompt = format!("After {}, what chord typically follows?", chord);

        let options = vec![
            "I".to_string(),
            "ii".to_string(),
            "IV".to_string(),
            "V".to_string(),
            "vi".to_string(),
            next_chord.clone(),
        ];

        let mut unique_options: Vec<String> = options
            .into_iter()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .take(4)
            .collect();
        unique_options.sort();

        Task {
            task_type: TaskType::Successor { item: chord },
            prompt,
            correct_answer: next_chord,
            options: unique_options,
            difficulty: 0.4,
            operation: OperationType::Successor,
        }
    }

    pub fn generate_transposition_task(&self, note: String, interval: i32) -> Task {
        let from_pc = MusicTheory::note_to_pitch_class(&note);
        let to_pc = ((from_pc as i32 + interval + 12) % 12) as u8;
        let result = MusicTheory::pitch_class_to_note(to_pc, interval > 0);

        let prompt = format!(
            "Transpose {} {} {} semitones",
            note,
            if interval > 0 { "up" } else { "down" },
            interval.abs()
        );

        let options: Vec<String> = (0..12)
            .map(|pc| MusicTheory::pitch_class_to_note(pc, true))
            .filter(|n| rand::random::<f64>() < 0.3 || n == &result)
            .take(4)
            .collect();

        Task {
            task_type: TaskType::KJump {
                start: note,
                k: interval,
            },
            prompt,
            correct_answer: result,
            options,
            difficulty: 0.3 + (interval.abs() as f64 / 12.0),
            operation: OperationType::KJump(interval),
        }
    }
}
