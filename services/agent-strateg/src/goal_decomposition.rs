//! 🎯 Goal Decomposition - Dekompozycja celów strategicznych
//! 
//! System dekompozycji wysokopoziomowych celów na wykonalne pod-cele i zadania

use anyhow::{Result, anyhow};
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{info, debug, warn, instrument};

use crate::{
    config::Config,
    ai_models::AIResponse,
};

/// 🎯 Priorytet celu
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GoalPriority {
    /// Krytyczny - natychmiastowe wykonanie
    Critical = 4,
    /// Wysokie - wykonanie w ciągu godziny
    High = 3,
    /// Średnie - wykonanie w ciągu dnia
    Medium = 2,
    /// Niskie - wykonanie w ciągu tygodnia
    Low = 1,
}

/// 📊 Status celu
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalStatus {
    /// Cel oczekuje na dekompozycję
    Pending,
    /// Cel jest w trakcie dekompozycji
    Decomposing,
    /// Cel został zdekomponowany
    Decomposed,
    /// Cel jest w trakcie wykonywania
    InProgress,
    /// Cel został ukończony
    Completed,
    /// Cel nie powiódł się
    Failed,
    /// Cel został anulowany
    Cancelled,
    /// Cel przekroczył deadline
    Expired,
}

/// 🎯 Definicja celu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    /// Unikalny identyfikator celu
    pub id: Uuid,
    /// Tytuł celu
    pub title: String,
    /// Szczegółowy opis
    pub description: String,
    /// Priorytet celu
    pub priority: GoalPriority,
    /// Deadline wykonania
    pub deadline: DateTime<Utc>,
    /// Kontekst i dane wejściowe
    pub context: Value,
    /// Lista pod-celów
    pub sub_goals: Vec<SubGoal>,
    /// Aktualny status
    pub status: GoalStatus,
    /// Czas utworzenia
    pub created_at: DateTime<Utc>,
    /// ID celu nadrzędnego (jeśli jest pod-celem)
    pub parent_goal_id: Option<Uuid>,
    /// Metadane celu
    pub metadata: HashMap<String, String>,
    /// Wymagane zasoby
    pub required_resources: Vec<String>,
    /// Oczekiwane rezultaty
    pub expected_outcomes: Vec<String>,
}

impl Goal {
    /// Tworzy nowy cel
    pub fn new(
        title: String,
        description: String,
        priority: GoalPriority,
        context: Value,
    ) -> Self {
        let now = Utc::now();
        let deadline = match priority {
            GoalPriority::Critical => now + Duration::minutes(30),
            GoalPriority::High => now + Duration::hours(2),
            GoalPriority::Medium => now + Duration::hours(8),
            GoalPriority::Low => now + Duration::days(1),
        };
        
        Self {
            id: Uuid::new_v4(),
            title,
            description,
            priority,
            deadline,
            context,
            sub_goals: Vec::new(),
            status: GoalStatus::Pending,
            created_at: now,
            parent_goal_id: None,
            metadata: HashMap::new(),
            required_resources: Vec::new(),
            expected_outcomes: Vec::new(),
        }
    }
    
    /// Sprawdza czy cel wygasł
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.deadline
    }
    
    /// Sprawdza czy cel jest ukończony
    pub fn is_completed(&self) -> bool {
        matches!(self.status, GoalStatus::Completed)
    }
    
    /// Oblicza postęp celu na podstawie pod-celów
    pub fn calculate_progress(&self) -> f64 {
        if self.sub_goals.is_empty() {
            return match self.status {
                GoalStatus::Completed => 1.0,
                GoalStatus::InProgress => 0.5,
                _ => 0.0,
            };
        }
        
        let completed_count = self.sub_goals
            .iter()
            .filter(|sg| sg.status == SubGoalStatus::Completed)
            .count();
        
        completed_count as f64 / self.sub_goals.len() as f64
    }
}

/// 🎯 Pod-cel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubGoal {
    /// Unikalny identyfikator pod-celu
    pub id: Uuid,
    /// Tytuł pod-celu
    pub title: String,
    /// Opis pod-celu
    pub description: String,
    /// Priorytet pod-celu
    pub priority: GoalPriority,
    /// Status pod-celu
    pub status: SubGoalStatus,
    /// Deadline wykonania
    pub deadline: DateTime<Utc>,
    /// Wymagane możliwości agenta
    pub required_capabilities: Vec<String>,
    /// Preferowany typ agenta
    pub preferred_agent_type: Option<String>,
    /// Dane wejściowe
    pub input_data: Value,
    /// Oczekiwane rezultaty
    pub expected_output: Value,
    /// Czas utworzenia
    pub created_at: DateTime<Utc>,
}

/// 📊 Status pod-celu
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubGoalStatus {
    /// Pod-cel oczekuje na przydzielenie
    Pending,
    /// Pod-cel został przydzielony agentowi
    Assigned,
    /// Pod-cel jest w trakcie wykonywania
    InProgress,
    /// Pod-cel został ukończony
    Completed,
    /// Pod-cel nie powiódł się
    Failed,
    /// Pod-cel został anulowany
    Cancelled,
}

/// 🎯 Dekompozycja celów
pub struct GoalDecomposer {
    /// Konfiguracja
    config: Arc<Config>,
    /// Szablony dekompozycji
    decomposition_templates: HashMap<String, DecompositionTemplate>,
    /// Statystyki dekompozycji
    stats: DecompositionStats,
}

/// 📋 Szablon dekompozycji
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecompositionTemplate {
    /// Nazwa szablonu
    pub name: String,
    /// Typ celu, do którego pasuje
    pub goal_type: String,
    /// Lista kroków dekompozycji
    pub steps: Vec<DecompositionStep>,
    /// Wymagane dane wejściowe
    pub required_inputs: Vec<String>,
    /// Oczekiwane rezultaty
    pub expected_outputs: Vec<String>,
}

/// 📝 Krok dekompozycji
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecompositionStep {
    /// Nazwa kroku
    pub name: String,
    /// Opis kroku
    pub description: String,
    /// Typ agenta do wykonania
    pub agent_type: String,
    /// Wymagane możliwości
    pub required_capabilities: Vec<String>,
    /// Szacowany czas wykonania (minuty)
    pub estimated_duration_minutes: u32,
    /// Zależności od innych kroków
    pub dependencies: Vec<String>,
}

/// 📊 Statystyki dekompozycji
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DecompositionStats {
    pub total_goals_decomposed: u64,
    pub successful_decompositions: u64,
    pub failed_decompositions: u64,
    pub average_sub_goals_per_goal: f64,
    pub average_decomposition_time_ms: f64,
}

impl GoalDecomposer {
    /// Tworzy nowy dekompozycja celów
    #[instrument(skip(config))]
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        info!("🎯 Initializing GoalDecomposer...");
        
        let mut decomposer = Self {
            config,
            decomposition_templates: HashMap::new(),
            stats: DecompositionStats::default(),
        };
        
        // Załaduj domyślne szablony
        decomposer.load_default_templates().await?;
        
        info!("✅ GoalDecomposer initialized");
        Ok(decomposer)
    }
    
    /// Dekomponuje cel na pod-cele
    #[instrument(skip(self, goal, ai_analysis))]
    pub async fn decompose_goal(
        &mut self,
        mut goal: Goal,
        ai_analysis: AIResponse,
    ) -> Result<Vec<Goal>> {
        debug!("🎯 Decomposing goal: {}", goal.title);
        
        let start_time = std::time::Instant::now();
        goal.status = GoalStatus::Decomposing;
        
        // Znajdź odpowiedni szablon
        let template = self.find_template(&goal, &ai_analysis)?;
        
        // Wykonaj dekompozycję
        let sub_goals = self.execute_decomposition(&goal, &template, &ai_analysis).await?;
        
        // Aktualizuj cel
        goal.status = GoalStatus::Decomposed;
        
        // Aktualizuj statystyki
        let duration = start_time.elapsed();
        self.update_stats(true, sub_goals.len(), duration.as_millis() as f64);
        
        info!("✅ Goal decomposed into {} sub-goals", sub_goals.len());
        Ok(sub_goals)
    }
    
    /// Ładuje domyślne szablony dekompozycji
    async fn load_default_templates(&mut self) -> Result<()> {
        // Szablon dla analizy tokenów
        let token_analysis_template = DecompositionTemplate {
            name: "Token Analysis".to_string(),
            goal_type: "token_analysis".to_string(),
            steps: vec![
                DecompositionStep {
                    name: "Market Data Collection".to_string(),
                    description: "Collect current market data for the token".to_string(),
                    agent_type: "Quant".to_string(),
                    required_capabilities: vec!["DataCollection".to_string()],
                    estimated_duration_minutes: 5,
                    dependencies: vec![],
                },
                DecompositionStep {
                    name: "Sentiment Analysis".to_string(),
                    description: "Analyze social sentiment and community activity".to_string(),
                    agent_type: "Analityk".to_string(),
                    required_capabilities: vec!["SentimentAnalysis".to_string()],
                    estimated_duration_minutes: 10,
                    dependencies: vec![],
                },
                DecompositionStep {
                    name: "Risk Assessment".to_string(),
                    description: "Evaluate potential risks and security concerns".to_string(),
                    agent_type: "Nadzorca".to_string(),
                    required_capabilities: vec!["RiskAssessment".to_string()],
                    estimated_duration_minutes: 15,
                    dependencies: vec!["Market Data Collection".to_string()],
                },
            ],
            required_inputs: vec!["token_address".to_string()],
            expected_outputs: vec!["analysis_report".to_string(), "recommendation".to_string()],
        };
        
        self.decomposition_templates.insert(
            "token_analysis".to_string(),
            token_analysis_template,
        );
        
        debug!("📋 Loaded {} decomposition templates", self.decomposition_templates.len());
        Ok(())
    }
    
    /// Znajduje odpowiedni szablon dla celu
    fn find_template(&self, goal: &Goal, _ai_analysis: &AIResponse) -> Result<&DecompositionTemplate> {
        // Prosta logika dopasowania na podstawie metadanych celu
        let goal_type = goal.metadata.get("type")
            .unwrap_or(&"default".to_string())
            .clone();
        
        self.decomposition_templates.get(&goal_type)
            .ok_or_else(|| anyhow!("No template found for goal type: {}", goal_type))
    }
    
    /// Wykonuje dekompozycję na podstawie szablonu
    async fn execute_decomposition(
        &self,
        goal: &Goal,
        template: &DecompositionTemplate,
        _ai_analysis: &AIResponse,
    ) -> Result<Vec<Goal>> {
        let mut sub_goals = Vec::new();
        
        for step in &template.steps {
            let sub_goal = Goal {
                id: Uuid::new_v4(),
                title: step.name.clone(),
                description: step.description.clone(),
                priority: goal.priority.clone(),
                deadline: goal.deadline,
                context: goal.context.clone(),
                sub_goals: Vec::new(),
                status: GoalStatus::Pending,
                created_at: Utc::now(),
                parent_goal_id: Some(goal.id),
                metadata: {
                    let mut metadata = HashMap::new();
                    metadata.insert("agent_type".to_string(), step.agent_type.clone());
                    metadata.insert("estimated_duration".to_string(), step.estimated_duration_minutes.to_string());
                    metadata
                },
                required_resources: step.required_capabilities.clone(),
                expected_outcomes: vec![],
            };
            
            sub_goals.push(sub_goal);
        }
        
        Ok(sub_goals)
    }
    
    /// Aktualizuje statystyki dekompozycji
    fn update_stats(&mut self, success: bool, sub_goals_count: usize, duration_ms: f64) {
        self.stats.total_goals_decomposed += 1;
        
        if success {
            self.stats.successful_decompositions += 1;
        } else {
            self.stats.failed_decompositions += 1;
        }
        
        // Aktualizuj średnią liczbę pod-celów
        let total_successful = self.stats.successful_decompositions as f64;
        self.stats.average_sub_goals_per_goal = 
            (self.stats.average_sub_goals_per_goal * (total_successful - 1.0) + sub_goals_count as f64) / total_successful;
        
        // Aktualizuj średni czas dekompozycji
        let total_goals = self.stats.total_goals_decomposed as f64;
        self.stats.average_decomposition_time_ms = 
            (self.stats.average_decomposition_time_ms * (total_goals - 1.0) + duration_ms) / total_goals;
    }
    
    /// Pobiera statystyki dekompozycji
    pub fn get_stats(&self) -> &DecompositionStats {
        &self.stats
    }
}
