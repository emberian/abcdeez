use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

/// Citation Management System for Research Methodology
/// Tracks and formats references for experimental methods, statistical procedures, and tools

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationManager {
    pub references: HashMap<String, Reference>,
    pub methodology_citations: HashMap<String, Vec<String>>, // method -> reference_ids
    pub software_citations: HashMap<String, String>,         // software -> reference_id
    pub used_methods: HashSet<String>,
    pub bibliography_style: BibliographyStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    pub id: String,
    pub reference_type: ReferenceType,
    pub title: String,
    pub authors: Vec<Author>,
    pub year: u32,
    pub publication: Publication,
    pub doi: Option<String>,
    pub url: Option<String>,
    pub abstract_text: Option<String>,
    pub keywords: Vec<String>,
    pub notes: String,
    pub citation_count: u32,
    pub added_date: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub first_name: String,
    pub last_name: String,
    pub middle_initial: Option<String>,
    pub affiliation: Option<String>,
    pub orcid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReferenceType {
    Journal,
    Book,
    Chapter,
    Conference,
    Thesis,
    Software,
    Manual,
    Website,
    Preprint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Publication {
    Journal {
        name: String,
        volume: Option<u32>,
        issue: Option<u32>,
        pages: Option<String>,
        issn: Option<String>,
    },
    Book {
        publisher: String,
        isbn: Option<String>,
        pages: Option<u32>,
        edition: Option<String>,
    },
    Conference {
        name: String,
        proceedings: String,
        location: String,
        pages: Option<String>,
    },
    Thesis {
        degree_type: String,
        institution: String,
        department: String,
    },
    Software {
        version: String,
        platform: String,
        repository: Option<String>,
    },
    Website {
        site_name: String,
        access_date: chrono::NaiveDate,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BibliographyStyle {
    APA,
    Chicago,
    MLA,
    Harvard,
    Vancouver,
    IEEE,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodologyReport {
    pub experiment_id: String,
    pub methods_used: Vec<MethodCitation>,
    pub software_used: Vec<SoftwareCitation>,
    pub statistical_procedures: Vec<StatisticalCitation>,
    pub bibliography: Vec<FormattedCitation>,
    pub method_justifications: HashMap<String, String>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodCitation {
    pub method_name: String,
    pub description: String,
    pub references: Vec<String>,
    pub parameters_used: HashMap<String, String>,
    pub justification: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareCitation {
    pub software_name: String,
    pub version: String,
    pub purpose: String,
    pub reference_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalCitation {
    pub procedure_name: String,
    pub description: String,
    pub assumptions: Vec<String>,
    pub references: Vec<String>,
    pub software_implementation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattedCitation {
    pub reference_id: String,
    pub formatted_text: String,
    pub in_text_citation: String,
}

impl CitationManager {
    pub fn new() -> Self {
        let mut manager = Self {
            references: HashMap::new(),
            methodology_citations: HashMap::new(),
            software_citations: HashMap::new(),
            used_methods: HashSet::new(),
            bibliography_style: BibliographyStyle::APA,
        };

        // Add default methodological references
        manager.add_default_references();
        manager
    }

    pub fn with_style(mut self, style: BibliographyStyle) -> Self {
        self.bibliography_style = style;
        self
    }

    pub fn add_reference(&mut self, reference: Reference) {
        self.references.insert(reference.id.clone(), reference);
    }

    pub fn add_method_citation(&mut self, method_name: &str, reference_ids: Vec<String>) {
        self.methodology_citations
            .insert(method_name.to_string(), reference_ids);
    }

    pub fn add_software_citation(&mut self, software_name: &str, reference_id: String) {
        self.software_citations
            .insert(software_name.to_string(), reference_id);
    }

    pub fn mark_method_used(&mut self, method_name: &str) {
        self.used_methods.insert(method_name.to_string());
    }

    pub fn generate_methodology_report(&self, experiment_id: &str) -> MethodologyReport {
        let mut methods_used = Vec::new();
        let mut software_used = Vec::new();
        let mut statistical_procedures = Vec::new();
        let mut all_reference_ids = HashSet::new();

        // Collect method citations
        for method_name in &self.used_methods {
            if let Some(reference_ids) = self.methodology_citations.get(method_name) {
                all_reference_ids.extend(reference_ids.iter().cloned());

                methods_used.push(MethodCitation {
                    method_name: method_name.clone(),
                    description: self.get_method_description(method_name),
                    references: reference_ids.clone(),
                    parameters_used: self.get_method_parameters(method_name),
                    justification: self.get_method_justification(method_name),
                });
            }
        }

        // Collect software citations
        for (software_name, reference_id) in &self.software_citations {
            if self.is_software_used(software_name) {
                all_reference_ids.insert(reference_id.clone());

                if let Some(reference) = self.references.get(reference_id) {
                    software_used.push(SoftwareCitation {
                        software_name: software_name.clone(),
                        version: self.extract_version(reference),
                        purpose: self.get_software_purpose(software_name),
                        reference_id: reference_id.clone(),
                    });
                }
            }
        }

        // Collect statistical procedure citations
        statistical_procedures = self.get_statistical_citations(&all_reference_ids);

        // Generate formatted bibliography
        let bibliography = self.format_bibliography(&all_reference_ids);

        MethodologyReport {
            experiment_id: experiment_id.to_string(),
            methods_used,
            software_used,
            statistical_procedures,
            bibliography,
            method_justifications: self.get_all_justifications(),
            generated_at: chrono::Utc::now(),
        }
    }

    pub fn export_bibliography(
        &self,
        path: &PathBuf,
        format: BibliographyFormat,
    ) -> std::io::Result<()> {
        let content = match format {
            BibliographyFormat::BibTeX => self.generate_bibtex(),
            BibliographyFormat::RIS => self.generate_ris(),
            BibliographyFormat::EndNote => self.generate_endnote(),
            BibliographyFormat::Formatted => self.generate_formatted_bibliography(),
        };

        let mut file = File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn import_from_doi(&mut self, doi: &str) -> Result<String, String> {
        // Mock implementation - would use DOI resolution service
        self.mock_doi_import(doi)
    }

    pub fn search_references(&self, query: &str) -> Vec<&Reference> {
        let query_lower = query.to_lowercase();

        self.references
            .values()
            .filter(|reference| {
                reference.title.to_lowercase().contains(&query_lower)
                    || reference
                        .authors
                        .iter()
                        .any(|author| author.last_name.to_lowercase().contains(&query_lower))
                    || reference
                        .keywords
                        .iter()
                        .any(|keyword| keyword.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    pub fn get_in_text_citation(&self, reference_id: &str) -> String {
        if let Some(reference) = self.references.get(reference_id) {
            self.format_in_text_citation(reference)
        } else {
            format!("[{}]", reference_id)
        }
    }

    pub fn generate_methods_section(&self, experiment_id: &str) -> String {
        let report = self.generate_methodology_report(experiment_id);
        self.format_methods_section(&report)
    }

    // Private helper methods
    fn add_default_references(&mut self) {
        // Add key methodological references
        self.add_bayesian_references();
        self.add_statistical_references();
        self.add_experimental_design_references();
        self.add_software_references();
    }

    fn add_bayesian_references(&mut self) {
        let reference = Reference {
            id: "kruschke2014".to_string(),
            reference_type: ReferenceType::Book,
            title: "Doing Bayesian Data Analysis: A Tutorial with R, JAGS, and Stan".to_string(),
            authors: vec![Author {
                first_name: "John".to_string(),
                last_name: "Kruschke".to_string(),
                middle_initial: Some("K".to_string()),
                affiliation: Some("Indiana University".to_string()),
                orcid: None,
            }],
            year: 2014,
            publication: Publication::Book {
                publisher: "Academic Press".to_string(),
                isbn: Some("978-0124058880".to_string()),
                pages: Some(776),
                edition: Some("2nd".to_string()),
            },
            doi: None,
            url: None,
            abstract_text: Some(
                "A comprehensive introduction to Bayesian data analysis".to_string(),
            ),
            keywords: vec![
                "Bayesian statistics".to_string(),
                "MCMC".to_string(),
                "Data analysis".to_string(),
            ],
            notes: "Standard reference for Bayesian analysis methods".to_string(),
            citation_count: 5000,
            added_date: chrono::Utc::now(),
        };

        self.add_reference(reference);
        self.add_method_citation("bayesian_inference", vec!["kruschke2014".to_string()]);

        // Add adaptive learning reference
        let adaptive_ref = Reference {
            id: "settles2009".to_string(),
            reference_type: ReferenceType::Journal,
            title: "Active Learning Literature Survey".to_string(),
            authors: vec![Author {
                first_name: "Burr".to_string(),
                last_name: "Settles".to_string(),
                middle_initial: None,
                affiliation: Some("University of Wisconsin-Madison".to_string()),
                orcid: None,
            }],
            year: 2009,
            publication: Publication::Journal {
                name: "Computer Sciences Technical Report".to_string(),
                volume: Some(1648),
                issue: None,
                pages: Some("1-67".to_string()),
                issn: None,
            },
            doi: None,
            url: Some("http://burrsettles.com/pub/settles.activelearning.pdf".to_string()),
            abstract_text: Some("Comprehensive survey of active learning methods".to_string()),
            keywords: vec![
                "Active learning".to_string(),
                "Machine learning".to_string(),
                "Adaptive systems".to_string(),
            ],
            notes: "Foundational reference for adaptive learning algorithms".to_string(),
            citation_count: 3500,
            added_date: chrono::Utc::now(),
        };

        self.add_reference(adaptive_ref);
        self.add_method_citation("adaptive_scheduling", vec!["settles2009".to_string()]);
    }

    fn add_statistical_references(&mut self) {
        // Mixed-effects models
        let lme4_ref = Reference {
            id: "bates2015".to_string(),
            reference_type: ReferenceType::Journal,
            title: "Fitting Linear Mixed-Effects Models Using lme4".to_string(),
            authors: vec![
                Author {
                    first_name: "Douglas".to_string(),
                    last_name: "Bates".to_string(),
                    middle_initial: None,
                    affiliation: Some("University of Wisconsin-Madison".to_string()),
                    orcid: None,
                },
                Author {
                    first_name: "Martin".to_string(),
                    last_name: "Mächler".to_string(),
                    middle_initial: None,
                    affiliation: Some("ETH Zurich".to_string()),
                    orcid: None,
                },
            ],
            year: 2015,
            publication: Publication::Journal {
                name: "Journal of Statistical Software".to_string(),
                volume: Some(67),
                issue: Some(1),
                pages: Some("1-48".to_string()),
                issn: Some("1548-7660".to_string()),
            },
            doi: Some("10.18637/jss.v067.i01".to_string()),
            url: Some("https://www.jstatsoft.org/article/view/v067i01".to_string()),
            abstract_text: Some(
                "Description of the lme4 package for fitting linear mixed-effects models"
                    .to_string(),
            ),
            keywords: vec![
                "Mixed-effects models".to_string(),
                "R".to_string(),
                "Hierarchical models".to_string(),
            ],
            notes: "Primary reference for lme4 mixed-effects modeling".to_string(),
            citation_count: 15000,
            added_date: chrono::Utc::now(),
        };

        self.add_reference(lme4_ref);
        self.add_method_citation("mixed_effects_modeling", vec!["bates2015".to_string()]);
        self.add_software_citation("lme4", "bates2015".to_string());

        // Power analysis
        let power_ref = Reference {
            id: "cohen1988".to_string(),
            reference_type: ReferenceType::Book,
            title: "Statistical Power Analysis for the Behavioral Sciences".to_string(),
            authors: vec![Author {
                first_name: "Jacob".to_string(),
                last_name: "Cohen".to_string(),
                middle_initial: None,
                affiliation: Some("New York University".to_string()),
                orcid: None,
            }],
            year: 1988,
            publication: Publication::Book {
                publisher: "Lawrence Erlbaum Associates".to_string(),
                isbn: Some("978-0805802832".to_string()),
                pages: Some(567),
                edition: Some("2nd".to_string()),
            },
            doi: None,
            url: None,
            abstract_text: Some(
                "Comprehensive treatment of statistical power analysis".to_string(),
            ),
            keywords: vec![
                "Power analysis".to_string(),
                "Effect size".to_string(),
                "Sample size".to_string(),
            ],
            notes: "Classic reference for power analysis methodology".to_string(),
            citation_count: 25000,
            added_date: chrono::Utc::now(),
        };

        self.add_reference(power_ref);
        self.add_method_citation("power_analysis", vec!["cohen1988".to_string()]);
    }

    fn add_experimental_design_references(&mut self) {
        let counterbalancing_ref = Reference {
            id: "keppel2004".to_string(),
            reference_type: ReferenceType::Book,
            title: "Design and Analysis: A Researcher's Handbook".to_string(),
            authors: vec![
                Author {
                    first_name: "Geoffrey".to_string(),
                    last_name: "Keppel".to_string(),
                    middle_initial: None,
                    affiliation: Some("University of California, Berkeley".to_string()),
                    orcid: None,
                },
                Author {
                    first_name: "Thomas".to_string(),
                    last_name: "Wickens".to_string(),
                    middle_initial: Some("D".to_string()),
                    affiliation: Some("University of California, Los Angeles".to_string()),
                    orcid: None,
                },
            ],
            year: 2004,
            publication: Publication::Book {
                publisher: "Pearson Prentice Hall".to_string(),
                isbn: Some("978-0131424319".to_string()),
                pages: Some(712),
                edition: Some("4th".to_string()),
            },
            doi: None,
            url: None,
            abstract_text: Some(
                "Comprehensive guide to experimental design and analysis".to_string(),
            ),
            keywords: vec![
                "Experimental design".to_string(),
                "Counterbalancing".to_string(),
                "ANOVA".to_string(),
            ],
            notes: "Standard reference for counterbalancing and experimental design".to_string(),
            citation_count: 3000,
            added_date: chrono::Utc::now(),
        };

        self.add_reference(counterbalancing_ref);
        self.add_method_citation("counterbalancing", vec!["keppel2004".to_string()]);
        self.add_method_citation("latin_squares", vec!["keppel2004".to_string()]);
    }

    fn add_software_references(&mut self) {
        // R
        let r_ref = Reference {
            id: "rcoreteam2023".to_string(),
            reference_type: ReferenceType::Software,
            title: "R: A Language and Environment for Statistical Computing".to_string(),
            authors: vec![Author {
                first_name: "R Core".to_string(),
                last_name: "Team".to_string(),
                middle_initial: None,
                affiliation: Some("R Foundation for Statistical Computing".to_string()),
                orcid: None,
            }],
            year: 2023,
            publication: Publication::Software {
                version: "4.3.0".to_string(),
                platform: "Cross-platform".to_string(),
                repository: Some("https://www.R-project.org/".to_string()),
            },
            doi: None,
            url: Some("https://www.R-project.org/".to_string()),
            abstract_text: Some(
                "R is a free software environment for statistical computing and graphics"
                    .to_string(),
            ),
            keywords: vec![
                "R".to_string(),
                "Statistical software".to_string(),
                "Open source".to_string(),
            ],
            notes: "Primary statistical computing environment".to_string(),
            citation_count: 50000,
            added_date: chrono::Utc::now(),
        };

        self.add_reference(r_ref);
        self.add_software_citation("R", "rcoreteam2023".to_string());

        // Python/SciPy
        let scipy_ref = Reference {
            id: "virtanen2020".to_string(),
            reference_type: ReferenceType::Journal,
            title: "SciPy 1.0: Fundamental Algorithms for Scientific Computing in Python"
                .to_string(),
            authors: vec![Author {
                first_name: "Pauli".to_string(),
                last_name: "Virtanen".to_string(),
                middle_initial: None,
                affiliation: Some("University of Helsinki".to_string()),
                orcid: Some("0000-0003-0111-2701".to_string()),
            }],
            year: 2020,
            publication: Publication::Journal {
                name: "Nature Methods".to_string(),
                volume: Some(17),
                issue: None,
                pages: Some("261-272".to_string()),
                issn: Some("1548-7105".to_string()),
            },
            doi: Some("10.1038/s41592-019-0686-2".to_string()),
            url: Some("https://doi.org/10.1038/s41592-019-0686-2".to_string()),
            abstract_text: Some(
                "SciPy is an open-source scientific computing library for Python".to_string(),
            ),
            keywords: vec![
                "Python".to_string(),
                "Scientific computing".to_string(),
                "Statistics".to_string(),
            ],
            notes: "Primary Python scientific computing library".to_string(),
            citation_count: 8000,
            added_date: chrono::Utc::now(),
        };

        self.add_reference(scipy_ref);
        self.add_software_citation("SciPy", "virtanen2020".to_string());
        self.add_software_citation("Python", "virtanen2020".to_string());
    }

    fn get_method_description(&self, method_name: &str) -> String {
        match method_name {
            "bayesian_inference" => "Bayesian statistical inference using prior distributions and likelihood functions".to_string(),
            "adaptive_scheduling" => "Adaptive task scheduling based on learner performance and information gain".to_string(),
            "mixed_effects_modeling" => "Linear mixed-effects models for hierarchical and repeated measures data".to_string(),
            "power_analysis" => "Statistical power analysis for sample size determination and effect size estimation".to_string(),
            "counterbalancing" => "Systematic counterbalancing to control for order effects".to_string(),
            "latin_squares" => "Latin square designs for balanced experimental conditions".to_string(),
            _ => format!("Description for {}", method_name),
        }
    }

    fn get_method_parameters(&self, method_name: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        match method_name {
            "bayesian_inference" => {
                params.insert("prior_type".to_string(), "Non-informative".to_string());
                params.insert("mcmc_chains".to_string(), "4".to_string());
                params.insert("iterations".to_string(), "2000".to_string());
            }
            "power_analysis" => {
                params.insert("alpha".to_string(), "0.05".to_string());
                params.insert("power".to_string(), "0.80".to_string());
                params.insert("effect_size".to_string(), "Cohen's d = 0.5".to_string());
            }
            _ => {}
        }
        params
    }

    fn get_method_justification(&self, method_name: &str) -> String {
        match method_name {
            "bayesian_inference" => "Bayesian methods provide principled uncertainty quantification and naturally incorporate prior knowledge".to_string(),
            "adaptive_scheduling" => "Adaptive methods optimize learning by presenting tasks that maximize information gain".to_string(),
            "mixed_effects_modeling" => "Mixed-effects models account for individual differences and repeated measures correlation structure".to_string(),
            "power_analysis" => "Power analysis ensures adequate sample sizes for detecting meaningful effects".to_string(),
            _ => format!("Standard methodology for {}", method_name),
        }
    }

    fn is_software_used(&self, software_name: &str) -> bool {
        // Check if software is mentioned in used methods
        self.used_methods
            .iter()
            .any(|method| method.contains(software_name))
            || matches!(software_name, "R" | "Python" | "SciPy" | "lme4")
    }

    fn extract_version(&self, reference: &Reference) -> String {
        match &reference.publication {
            Publication::Software { version, .. } => version.clone(),
            _ => "Unknown".to_string(),
        }
    }

    fn get_software_purpose(&self, software_name: &str) -> String {
        match software_name {
            "R" => "Statistical analysis and data visualization".to_string(),
            "Python" => "Data processing and statistical modeling".to_string(),
            "SciPy" => "Scientific computing and statistical analysis".to_string(),
            "lme4" => "Linear mixed-effects modeling".to_string(),
            _ => format!("Analysis tool: {}", software_name),
        }
    }

    fn get_statistical_citations(
        &self,
        reference_ids: &HashSet<String>,
    ) -> Vec<StatisticalCitation> {
        let mut citations = Vec::new();

        if reference_ids.contains("bates2015") {
            citations.push(StatisticalCitation {
                procedure_name: "Linear Mixed-Effects Models".to_string(),
                description: "Hierarchical models for repeated measures and nested data"
                    .to_string(),
                assumptions: vec![
                    "Residuals normally distributed".to_string(),
                    "Random effects normally distributed".to_string(),
                    "Homoscedasticity".to_string(),
                ],
                references: vec!["bates2015".to_string()],
                software_implementation: Some("lme4 package in R".to_string()),
            });
        }

        if reference_ids.contains("cohen1988") {
            citations.push(StatisticalCitation {
                procedure_name: "Power Analysis".to_string(),
                description: "Sample size determination and effect size estimation".to_string(),
                assumptions: vec![
                    "Normal sampling distribution".to_string(),
                    "Known effect size".to_string(),
                ],
                references: vec!["cohen1988".to_string()],
                software_implementation: Some("pwr package in R".to_string()),
            });
        }

        citations
    }

    fn get_all_justifications(&self) -> HashMap<String, String> {
        let mut justifications = HashMap::new();
        for method in &self.used_methods {
            justifications.insert(method.clone(), self.get_method_justification(method));
        }
        justifications
    }

    fn format_bibliography(&self, reference_ids: &HashSet<String>) -> Vec<FormattedCitation> {
        let mut bibliography = Vec::new();

        for reference_id in reference_ids {
            if let Some(reference) = self.references.get(reference_id) {
                let formatted_text = self.format_reference(reference);
                let in_text_citation = self.format_in_text_citation(reference);

                bibliography.push(FormattedCitation {
                    reference_id: reference_id.clone(),
                    formatted_text,
                    in_text_citation,
                });
            }
        }

        // Sort alphabetically by first author's last name
        bibliography.sort_by(|a, b| a.formatted_text.cmp(&b.formatted_text));
        bibliography
    }

    fn format_reference(&self, reference: &Reference) -> String {
        match self.bibliography_style {
            BibliographyStyle::APA => self.format_apa(reference),
            BibliographyStyle::Chicago => self.format_chicago(reference),
            BibliographyStyle::MLA => self.format_mla(reference),
            _ => self.format_apa(reference), // Default to APA
        }
    }

    fn format_apa(&self, reference: &Reference) -> String {
        let authors = self.format_authors_apa(&reference.authors);
        let year = reference.year;
        let title = &reference.title;

        match &reference.publication {
            Publication::Journal {
                name,
                volume,
                issue,
                pages,
                ..
            } => {
                let mut citation = format!("{}. ({}). {}. *{}*", authors, year, title, name);

                if let Some(vol) = volume {
                    citation.push_str(&format!(", *{}*", vol));
                    if let Some(iss) = issue {
                        citation.push_str(&format!("({})", iss));
                    }
                }

                if let Some(pgs) = pages {
                    citation.push_str(&format!(", {}", pgs));
                }

                if let Some(doi) = &reference.doi {
                    citation.push_str(&format!(". https://doi.org/{}", doi));
                }

                citation.push('.');
                citation
            }
            Publication::Book { publisher, .. } => {
                format!("{}. ({}). *{}*. {}.", authors, year, title, publisher)
            }
            Publication::Software { version, .. } => {
                format!(
                    "{}. ({}). *{}* (Version {}) [Computer software].",
                    authors, year, title, version
                )
            }
            _ => format!("{}. ({}). {}.", authors, year, title),
        }
    }

    fn format_chicago(&self, reference: &Reference) -> String {
        let authors = self.format_authors_chicago(&reference.authors);
        format!("{}. \"{}.\" {}", authors, reference.title, reference.year)
    }

    fn format_mla(&self, reference: &Reference) -> String {
        let authors = self.format_authors_mla(&reference.authors);
        format!("{}. \"{}.\" {}", authors, reference.title, reference.year)
    }

    fn format_authors_apa(&self, authors: &[Author]) -> String {
        if authors.is_empty() {
            return "Unknown Author".to_string();
        }

        if authors.len() == 1 {
            let author = &authors[0];
            let middle = author
                .middle_initial
                .as_ref()
                .map(|m| format!(" {}.", m))
                .unwrap_or_default();
            format!(
                "{}, {}.{}",
                author.last_name,
                author.first_name.chars().next().unwrap(),
                middle
            )
        } else if authors.len() <= 20 {
            let mut result = String::new();
            for (i, author) in authors.iter().enumerate() {
                if i > 0 {
                    if i == authors.len() - 1 {
                        result.push_str(", & ");
                    } else {
                        result.push_str(", ");
                    }
                }
                let middle = author
                    .middle_initial
                    .as_ref()
                    .map(|m| format!(" {}.", m))
                    .unwrap_or_default();
                result.push_str(&format!(
                    "{}, {}.{}",
                    author.last_name,
                    author.first_name.chars().next().unwrap(),
                    middle
                ));
            }
            result
        } else {
            // More than 20 authors - use et al.
            let first_19: String = authors
                .iter()
                .take(19)
                .enumerate()
                .map(|(i, author)| {
                    let prefix = if i == 0 { "" } else { ", " };
                    let middle = author
                        .middle_initial
                        .as_ref()
                        .map(|m| format!(" {}.", m))
                        .unwrap_or_default();
                    format!(
                        "{}{}, {}.{}",
                        prefix,
                        author.last_name,
                        author.first_name.chars().next().unwrap(),
                        middle
                    )
                })
                .collect();

            let last_author = authors.last().unwrap();
            let middle = last_author
                .middle_initial
                .as_ref()
                .map(|m| format!(" {}.", m))
                .unwrap_or_default();
            format!(
                "{}, ... {}, {}.{}",
                first_19,
                last_author.last_name,
                last_author.first_name.chars().next().unwrap(),
                middle
            )
        }
    }

    fn format_authors_chicago(&self, authors: &[Author]) -> String {
        if authors.is_empty() {
            return "Unknown Author".to_string();
        }

        authors
            .iter()
            .enumerate()
            .map(|(i, author)| {
                if i == 0 {
                    format!("{}, {}", author.last_name, author.first_name)
                } else {
                    format!("{} {}", author.first_name, author.last_name)
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn format_authors_mla(&self, authors: &[Author]) -> String {
        if authors.is_empty() {
            return "Unknown Author".to_string();
        }

        authors
            .iter()
            .enumerate()
            .map(|(i, author)| {
                if i == 0 {
                    format!("{}, {}", author.last_name, author.first_name)
                } else {
                    format!("{} {}", author.first_name, author.last_name)
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn format_in_text_citation(&self, reference: &Reference) -> String {
        match self.bibliography_style {
            BibliographyStyle::APA => {
                if reference.authors.len() == 1 {
                    format!("({}, {})", reference.authors[0].last_name, reference.year)
                } else if reference.authors.len() == 2 {
                    format!(
                        "({} & {}, {})",
                        reference.authors[0].last_name,
                        reference.authors[1].last_name,
                        reference.year
                    )
                } else if reference.authors.len() <= 20 {
                    format!(
                        "({} et al., {})",
                        reference.authors[0].last_name, reference.year
                    )
                } else {
                    format!(
                        "({} et al., {})",
                        reference.authors[0].last_name, reference.year
                    )
                }
            }
            _ => format!(
                "({}, {})",
                reference
                    .authors
                    .first()
                    .map(|a| &a.last_name)
                    .unwrap_or(&"Unknown".to_string()),
                reference.year
            ),
        }
    }

    fn format_methods_section(&self, report: &MethodologyReport) -> String {
        let mut methods = String::new();

        methods.push_str("# Methods\n\n");

        if !report.methods_used.is_empty() {
            methods.push_str("## Methodology\n\n");
            for method in &report.methods_used {
                methods.push_str(&format!(
                    "**{}**: {}",
                    method.method_name, method.description
                ));
                if !method.justification.is_empty() {
                    methods.push_str(&format!(" {}", method.justification));
                }

                // Add citations
                let citations: Vec<String> = method
                    .references
                    .iter()
                    .map(|ref_id| self.get_in_text_citation(ref_id))
                    .collect();
                if !citations.is_empty() {
                    methods.push_str(&format!(" {}", citations.join(", ")));
                }
                methods.push_str(".\n\n");
            }
        }

        if !report.software_used.is_empty() {
            methods.push_str("## Software\n\n");
            methods.push_str("Analysis was conducted using ");
            let software_citations: Vec<String> = report
                .software_used
                .iter()
                .map(|software| {
                    let citation = self.get_in_text_citation(&software.reference_id);
                    format!(
                        "{} {} {}",
                        software.software_name, software.version, citation
                    )
                })
                .collect();
            methods.push_str(&software_citations.join(", "));
            methods.push_str(".\n\n");
        }

        if !report.statistical_procedures.is_empty() {
            methods.push_str("## Statistical Analysis\n\n");
            for procedure in &report.statistical_procedures {
                methods.push_str(&format!(
                    "**{}**: {}",
                    procedure.procedure_name, procedure.description
                ));
                let citations: Vec<String> = procedure
                    .references
                    .iter()
                    .map(|ref_id| self.get_in_text_citation(ref_id))
                    .collect();
                if !citations.is_empty() {
                    methods.push_str(&format!(" {}", citations.join(", ")));
                }
                methods.push_str(".\n\n");
            }
        }

        methods
    }

    fn generate_bibtex(&self) -> String {
        let mut bibtex = String::new();

        for reference in self.references.values() {
            bibtex.push_str(&self.format_bibtex_entry(reference));
            bibtex.push('\n');
        }

        bibtex
    }

    fn format_bibtex_entry(&self, reference: &Reference) -> String {
        let entry_type = match reference.reference_type {
            ReferenceType::Journal => "article",
            ReferenceType::Book => "book",
            ReferenceType::Conference => "inproceedings",
            ReferenceType::Software => "misc",
            _ => "misc",
        };

        let mut entry = format!("@{}{{{}}},\n", entry_type, reference.id);

        // Authors
        let author_str = reference
            .authors
            .iter()
            .map(|a| format!("{}, {}", a.last_name, a.first_name))
            .collect::<Vec<_>>()
            .join(" and ");
        entry.push_str(&format!("  author = {{{}}},\n", author_str));

        // Title
        entry.push_str(&format!("  title = {{{}}},\n", reference.title));

        // Year
        entry.push_str(&format!("  year = {{{}}},\n", reference.year));

        // Publication-specific fields
        match &reference.publication {
            Publication::Journal {
                name,
                volume,
                pages,
                ..
            } => {
                entry.push_str(&format!("  journal = {{{}}},\n", name));
                if let Some(vol) = volume {
                    entry.push_str(&format!("  volume = {{{}}},\n", vol));
                }
                if let Some(pgs) = pages {
                    entry.push_str(&format!("  pages = {{{}}},\n", pgs));
                }
            }
            Publication::Book { publisher, .. } => {
                entry.push_str(&format!("  publisher = {{{}}},\n", publisher));
            }
            _ => {}
        }

        if let Some(doi) = &reference.doi {
            entry.push_str(&format!("  doi = {{{}}},\n", doi));
        }

        entry.push_str("}\n");
        entry
    }

    fn generate_ris(&self) -> String {
        let mut ris = String::new();

        for reference in self.references.values() {
            ris.push_str(&self.format_ris_entry(reference));
            ris.push_str("\n\n");
        }

        ris
    }

    fn format_ris_entry(&self, reference: &Reference) -> String {
        let mut entry = String::new();

        // Type
        let ty = match reference.reference_type {
            ReferenceType::Journal => "JOUR",
            ReferenceType::Book => "BOOK",
            ReferenceType::Conference => "CONF",
            _ => "GEN",
        };
        entry.push_str(&format!("TY  - {}\n", ty));

        // Authors
        for author in &reference.authors {
            entry.push_str(&format!(
                "AU  - {}, {}\n",
                author.last_name, author.first_name
            ));
        }

        // Title
        entry.push_str(&format!("TI  - {}\n", reference.title));

        // Year
        entry.push_str(&format!("PY  - {}\n", reference.year));

        // Publication details
        match &reference.publication {
            Publication::Journal {
                name,
                volume,
                pages,
                ..
            } => {
                entry.push_str(&format!("JO  - {}\n", name));
                if let Some(vol) = volume {
                    entry.push_str(&format!("VL  - {}\n", vol));
                }
                if let Some(pgs) = pages {
                    entry.push_str(&format!("SP  - {}\n", pgs));
                }
            }
            Publication::Book { publisher, .. } => {
                entry.push_str(&format!("PB  - {}\n", publisher));
            }
            _ => {}
        }

        if let Some(doi) = &reference.doi {
            entry.push_str(&format!("DO  - {}\n", doi));
        }

        entry.push_str("ER  - \n");
        entry
    }

    fn generate_endnote(&self) -> String {
        // Simplified EndNote format
        let mut endnote = String::new();

        for reference in self.references.values() {
            endnote.push_str(&format!("{}\n", self.format_apa(reference)));
        }

        endnote
    }

    fn generate_formatted_bibliography(&self) -> String {
        let mut bibliography = String::new();
        bibliography.push_str("# References\n\n");

        for reference in self.references.values() {
            bibliography.push_str(&format!(
                "{}. {}\n\n",
                reference.id,
                self.format_reference(reference)
            ));
        }

        bibliography
    }

    fn mock_doi_import(&mut self, doi: &str) -> Result<String, String> {
        // Mock implementation - would query CrossRef API or similar
        let reference = Reference {
            id: format!("doi_{}", doi.replace("/", "_").replace(".", "_")),
            reference_type: ReferenceType::Journal,
            title: format!("Article with DOI {}", doi),
            authors: vec![Author {
                first_name: "John".to_string(),
                last_name: "Doe".to_string(),
                middle_initial: None,
                affiliation: None,
                orcid: None,
            }],
            year: 2023,
            publication: Publication::Journal {
                name: "Journal of Research Methods".to_string(),
                volume: Some(1),
                issue: Some(1),
                pages: Some("1-10".to_string()),
                issn: None,
            },
            doi: Some(doi.to_string()),
            url: Some(format!("https://doi.org/{}", doi)),
            abstract_text: None,
            keywords: Vec::new(),
            notes: String::new(),
            citation_count: 0,
            added_date: chrono::Utc::now(),
        };

        let reference_id = reference.id.clone();
        self.add_reference(reference);
        Ok(reference_id)
    }
}

#[derive(Debug, Clone)]
pub enum BibliographyFormat {
    BibTeX,
    RIS,
    EndNote,
    Formatted,
}

impl Default for CitationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_citation_manager_creation() {
        let manager = CitationManager::new();
        assert!(!manager.references.is_empty());
        assert!(manager.references.contains_key("kruschke2014"));
    }

    #[test]
    fn test_add_method_citation() {
        let mut manager = CitationManager::new();
        manager.mark_method_used("bayesian_inference");

        let report = manager.generate_methodology_report("test_experiment");
        assert!(!report.methods_used.is_empty());
        assert!(report
            .methods_used
            .iter()
            .any(|m| m.method_name == "bayesian_inference"));
    }

    #[test]
    fn test_apa_formatting() {
        let manager = CitationManager::new();
        let reference = manager.references.get("kruschke2014").unwrap();
        let formatted = manager.format_apa(reference);

        assert!(formatted.contains("Kruschke"));
        assert!(formatted.contains("2014"));
        assert!(formatted.contains("Academic Press"));
    }

    #[test]
    fn test_in_text_citation() {
        let manager = CitationManager::new();
        let citation = manager.get_in_text_citation("kruschke2014");
        assert_eq!(citation, "(Kruschke, 2014)");
    }

    #[test]
    fn test_bibtex_generation() {
        let manager = CitationManager::new();
        let bibtex = manager.generate_bibtex();

        assert!(bibtex.contains("@book{kruschke2014"));
        assert!(bibtex.contains("author = {Kruschke, John}"));
        assert!(bibtex.contains("year = {2014}"));
    }

    #[test]
    fn test_methodology_report_generation() {
        let mut manager = CitationManager::new();
        manager.mark_method_used("bayesian_inference");
        manager.mark_method_used("power_analysis");

        let report = manager.generate_methodology_report("test_experiment");

        assert_eq!(report.methods_used.len(), 2);
        assert!(!report.bibliography.is_empty());
        assert!(report
            .methods_used
            .iter()
            .any(|m| m.method_name == "bayesian_inference"));
        assert!(report
            .methods_used
            .iter()
            .any(|m| m.method_name == "power_analysis"));
    }
}
