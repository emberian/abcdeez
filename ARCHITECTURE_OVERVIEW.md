# ARCHITECTURE OVERVIEW
*Comprehensive analysis of the adaptive learning research system*

## Executive Summary

This is a sophisticated **adaptive learning research platform** with a mature architecture that exceeds its current implementation depth. The system shows excellent modular design with 15 focused modules, clean separation of concerns, and research-grade statistical infrastructure. However, analysis reveals **infrastructure over implementation** - many advanced features are architected but contain placeholder algorithms (55 compiler warnings indicate unused parameters in method signatures designed for future capabilities).

## Core System Architecture

### 1. Module Organization
```
abcdeez/
├── core/                    # Core learning algorithms & data structures
│   ├── src/core/           # Bayesian models, learner state, topology
│   ├── src/statistics/     # Statistical analysis & validation
│   ├── src/experimental/   # Experiment design & management
│   ├── src/testing/        # A/B testing framework
│   ├── src/session/        # Multi-session experiment coordination
│   └── src/research/       # Citation tracking & academic compliance
├── web-backend/            # REST API & database layer
│   ├── handlers/          # API endpoints for learner, analytics, research
│   └── services/          # Business logic & adaptation algorithms
├── xilem-app/             # Desktop research dashboard UI
│   └── src/ui/screens/    # Research visualization & control interfaces
└── STATUS.md              # Current implementation status
```

### 2. Key Architectural Patterns

#### Domain-Driven Design
- **Core Domain**: `LearnerModel`, `BayesianLearnerModel`, `AdaptiveScheduler`
- **Research Domain**: `ExperimentFramework`, `ABTestFramework`, `MultiSessionManager`
- **Analytics Domain**: `PowerAnalyzer`, `MixedEffectsAnalyzer`, `StatisticalValidator`

#### Dependency Flow
```
UI Layer (xilem-app) 
    ↓ HTTP/JSON
Web Backend (handlers + services)
    ↓ Direct calls
Core Library (abcdeez-core)
    ↓ Database/Cache
Persistent Storage
```

## Data Flow Architecture

### 1. Learning Session Flow
```
Participant → Task Selection → Response Collection → Model Update → Next Task
     ↑                ↓              ↓              ↓           ↓
     └─── Intervention System ←── Struggle Detection ←── EIG Calculation
```

### 2. Experiment Management Flow
```
Study Design → Participant Assignment → Session Scheduling → Data Collection
     ↓                    ↓                     ↓              ↓
Statistical Plan → Counterbalancing → Multi-Session → Longitudinal Analysis
```

### 3. Research Pipeline Flow
```
Raw Response Data → Statistical Analysis → Citation Generation → IRB Reports
        ↓                     ↓                ↓                 ↓
   Session Analytics → A/B Test Results → Academic Papers → Compliance Docs
```

## Key Abstractions & Relationships

### 1. Learning Core
```rust
// Central model for participant state
struct LearnerModel {
    id: String,
    competency_scores: HashMap<String, f64>,  // Node → proficiency
    response_history: Vec<TaskResponse>,
    model_parameters: Vec<f64>,               // Bayesian parameters
}

// Bayesian inference engine
struct BayesianLearnerModel {
    prior_beliefs: Vec<f64>,
    posterior_distribution: Vec<f64>,
    covariance_matrix: Vec<Vec<f64>>,
    hyperparameters: BayesianHyperparameters,
}

// Task selection optimizer
struct AdaptiveScheduler {
    learner_model: LearnerModel,
    topology: Topology,
    use_eig: bool,                           // Expected Information Gain
}
```

### 2. Experimental Design
```rust
// Multi-session experiment coordination
struct MultiSessionExperiment {
    id: String,
    design: ExperimentalDesign,
    sessions: Vec<SessionPlan>,
    participant_assignments: HashMap<String, ParticipantAssignment>,
    scheduling_rules: SchedulingRules,
    analysis_plan: AnalysisPlan,
}

// A/B testing framework (⚠️ partially implemented)
struct ABTestFramework {
    active_tests: HashMap<String, ABTest>,
    power_analyzer: PowerAnalyzer,           // UNUSED - key integration point
    allocation_strategies: HashMap<String, AllocationStrategy>,
    results_cache: HashMap<String, TestResults>,
}
```

### 3. Statistical Infrastructure
```rust
// Statistical validation engine
struct StatisticalValidator {
    confidence_level: f64,
    correction_method: Option<CorrectionMethod>,
}

// Power analysis (exists but not integrated)
struct PowerAnalyzer {
    significance_level: f64,
    minimum_power: f64,
    effect_size_estimates: HashMap<String, f64>,
}

// Mixed effects modeling (placeholder implementation)
struct MixedEffectsAnalyzer {
    fixed_effects: Vec<FixedEffect>,
    random_effects: Vec<RandomEffect>,
    convergence_criteria: ConvergenceCriteria,
}
```

## Integration Points & Critical Connections

### 1. ⚠️ A/B Testing → Statistical Analysis Gap
```rust
impl ABTestFramework {
    // SIGNATURE EXISTS, IMPLEMENTATION MISSING
    fn calculate_statistical_power(&self, 
        confidence_level: f64,    // UNUSED parameter
        test: &ABTest            // UNUSED parameter
    ) -> f64 {
        // TODO: Integrate with self.power_analyzer
        0.8  // Placeholder return
    }
}
```

**Integration Required**: Connect `ABTestFramework.power_analyzer` field to actual power calculations.

### 2. ⚠️ Mixed Effects → Dashboard Gap
```rust
// EXISTS: Statistical model
struct MixedEffectsAnalyzer { /* fully defined */ }

// MISSING: Dashboard integration
// No handlers in web-backend/src/handlers/ for mixed effects
// No UI components in xilem-app/src/ui/ for mixed effects visualization
```

### 3. ⚠️ Citation System → Statistics Gap
```rust
// EXISTS: Citation tracking
struct CitationTracker {
    statistical_procedures: Vec<StatisticalProcedure>, // POPULATED but unused
    generated_citations: Vec<Citation>,
}

// MISSING: Automatic citation when statistical tests run
// Statistical tests don't call citation_tracker.record_procedure()
```

## Unused/Underused Components Analysis

### 1. High-Impact Unused Fields
```rust
// ABTestFramework - 15+ unused parameters in methods
struct ABTestFramework {
    power_analyzer: PowerAnalyzer,              // FIELD: Unused in any method
    allocation_strategies: HashMap<...>,        // FIELD: Unused in assignment
    statistical_validator: StatisticalValidator, // FIELD: Unused in analysis
}

// MultiSessionExperiment - Scheduling logic incomplete  
struct SchedulingRules {
    preferred_time_windows: Vec<TimeWindow>,    // FIELD: Unused in scheduling
    blocked_dates: Vec<chrono::NaiveDate>,     // FIELD: Unused in validation
    reminder_settings: ReminderSettings,       // FIELD: Unused in notifications
}
```

### 2. Statistical Pipeline Gaps
```rust
// Power analysis exists but isolated
impl PowerAnalyzer {
    fn calculate_sample_size_for_test(&self, test_type: TestType) -> usize {
        // METHOD: Complete implementation, never called
    }
}

// Mixed effects complete but not integrated
impl MixedEffectsAnalyzer {  
    fn fit_longitudinal_model(&mut self, data: &LongitudinalData) -> ModelResults {
        // METHOD: Complete implementation, never called
    }
}
```

## Statistical Integration Architecture

### 1. Current State: Isolation
```
PowerAnalyzer ──┐
                ├── (No Integration)
MixedEffects ───┤
                ├── ABTestFramework ──→ Placeholder Results
Citations ──────┘
```

### 2. Required Integration: Unified Pipeline  
```
Study Design ──→ Power Analysis ──→ Sample Size Calculation
     ↓                 ↓                    ↓
A/B Test Setup ──→ Statistical Tests ──→ Results Analysis
     ↓                 ↓                    ↓
Citation Tracking ←── Mixed Effects ←── Multi-Session Data
```

## Implementation Status by Component

### ✅ Complete & Production Ready
- **Bayesian Learning Engine**: Full implementation with EIG calculation
- **Task Generation & Selection**: Adaptive algorithms working
- **Topology Management**: Graph structure for curriculum organization  
- **Response Time Analysis**: Ex-Gaussian modeling, strategy detection
- **Database Integration**: SQLx with proper error handling
- **IRB Compliance**: Document generation for research ethics

### ⚠️ Architected but Incomplete  
- **A/B Testing Framework**: 35+ method parameters unused, core statistics missing
- **Multi-Session Scheduling**: Rules defined, algorithms placeholder
- **Mixed Effects Analysis**: Model complete, dashboard integration missing
- **Statistical Validation**: Tests implemented, not called by frameworks
- **Citation System**: Tracking works, automatic recording missing

### 🚧 Designed but Not Implemented
- **Audio/Sensor Integration**: Structs defined, processing missing
- **Advanced Counterbalancing**: Latin Square methods defined, unused
- **Real-time Analytics**: WebSocket infrastructure, computation missing
- **Export/Import Systems**: File format support incomplete

## Critical Path for Statistical Integration

### Phase 1: A/B Testing Core (Priority 3 from IMPLEMENTATION_PLAN.md)
1. **Connect Power Analysis**
   ```rust
   impl ABTestFramework {
       fn calculate_statistical_power(&self, test: &ABTest) -> f64 {
           self.power_analyzer.calculate_power_for_test(test) // USE the field
       }
   }
   ```

2. **Implement Statistical Tests**
   ```rust
   fn analyze_test_results(&self, test: &ABTest) -> TestResults {
       let validator = &self.statistical_validator; // USE the field
       validator.t_test(&control_data, &treatment_data)
   }
   ```

### Phase 2: Mixed Effects Integration
1. **Add API Endpoints**: `/api/research/mixed-effects-analysis`
2. **Dashboard Components**: Mixed effects results visualization
3. **Automatic Model Fitting**: Trigger on multi-session completion

### Phase 3: Citation Automation
1. **Hook Statistical Tests**: Auto-record when tests run
2. **Generate Method Sections**: For papers using specific tests
3. **Export Bibliography**: Integration with academic writing tools

## Dashboard Integration Architecture

### Current UI Structure
```
Research Dashboard
├── Experiment Management ✅
├── Participant Monitoring ✅  
├── Real-time Analytics ⚠️ (WebSocket ready, computation missing)
├── Statistical Results ⚠️ (A/B tests not integrated)
└── Export/Reports ⚠️ (IRB complete, statistical exports missing)
```

### Multi-Session Data Flow
```
Session Start → Data Collection → Interim Analysis Check → Session End
     ↓               ↓                    ↓                  ↓
Participant UI → WebSocket Updates → Dashboard Alert → Schedule Next
```

**Gap**: Interim analysis rules defined but not executed automatically.

## Recommendations for Completion

### 1. Immediate (Week 1)
- **Fix ABTestFramework Parameter Usage**: Address 15+ unused parameters
- **Integrate PowerAnalyzer**: Connect to actual test calculations  
- **Implement Core Statistical Tests**: t-tests, chi-square, ANOVA

### 2. Short-term (Weeks 2-4)  
- **Mixed Effects Dashboard**: Add visualization components
- **Multi-Session Scheduling**: Implement constraint-based algorithms
- **Citation Automation**: Hook statistical procedure recording

### 3. Medium-term (Weeks 5-8)
- **Advanced A/B Features**: Sequential testing, multi-armed bandits
- **Audio Integration**: Connect sensor data to learning models  
- **Export System**: Statistical results to academic formats

### 4. Research Impact
- **Academic Integration**: Citation system → paper writing pipeline
- **Replication Package**: Automated generation from experiments  
- **Open Science**: Data/analysis sharing infrastructure

## Architecture Strengths

1. **Modular Design**: Clean separation allows independent development
2. **Type Safety**: Comprehensive error handling with Result<T, E>
3. **Research Focus**: Academic compliance built into core architecture
4. **Statistical Rigor**: Proper normality tests, multiple comparison corrections
5. **Extensibility**: Plugin architecture for new statistical methods

## Architecture Risks

1. **Over-Engineering**: UI infrastructure exceeds current needs
2. **Integration Debt**: Many components exist but aren't connected  
3. **Documentation Gap**: Implementation status not clear from code
4. **Testing Coverage**: Advanced statistical methods need validation
5. **Performance**: Multi-session coordination could become bottleneck

---

*This analysis reveals a sophisticated research platform with excellent foundations but significant integration work needed to realize its full potential. The architecture is sound - the challenge is connecting the pieces.*