//! 🔄 Feedback Loop - System uczenia się
//! 
//! Pętla uczenia się z wyników transakcji w architekturze Hive Mind

use anyhow::{Result, anyhow};
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::RwLock;
use tracing::{info, debug, warn, error, instrument};

use crate::{
    config::Config,
    memory_store::{MemoryStore, MemoryEntry, MemoryType},
    task_delegation::{Task, TaskResult},
    agent_types::AgentType,
};

/// 📊 Dane feedbacku z wykonania zadania
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackData {
    /// ID zadania
    pub task_id: Uuid,
    /// Typ zadania
    pub task_type: String,
    /// ID agenta, który wykonał zadanie
    pub agent_id: Uuid,
    /// Typ agenta
    pub agent_type: AgentType,
    /// Czy zadanie zakończone sukcesem
    pub success: bool,
    /// Wynik zadania
    pub result: Option<Value>,
    /// Błąd (jeśli wystąpił)
    pub error: Option<String>,
    /// Czas wykonania w milisekundach
    pub execution_time_ms: u64,
    /// Dane wejściowe zadania
    pub input_data: Value,
    /// Warunki rynkowe w czasie wykonania
    pub market_conditions: MarketConditions,
    /// Wynik finansowy (jeśli dotyczy)
    pub financial_outcome: Option<FinancialOutcome>,
    /// Czas utworzenia feedbacku
    pub created_at: DateTime<Utc>,
    /// Metadane dodatkowe
    pub metadata: HashMap<String, String>,
}

/// 📈 Warunki rynkowe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketConditions {
    /// Volatilność rynku (0.0 - 1.0)
    pub volatility: f64,
    /// Wolumen transakcji
    pub volume: f64,
    /// Trend cenowy (-1.0 do 1.0)
    pub price_trend: f64,
    /// Poziom kongestii sieci (0.0 - 1.0)
    pub network_congestion: f64,
    /// Średnia opłata za transakcję
    pub average_fee: f64,
    /// Czas bloku w sekundach
    pub block_time_seconds: f64,
}

/// 💰 Wynik finansowy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialOutcome {
    /// Zysk/strata w SOL
    pub profit_loss_sol: f64,
    /// Zysk/strata w USD
    pub profit_loss_usd: f64,
    /// Zwrot z inwestycji (ROI) w procentach
    pub roi_percent: f64,
    /// Opłaty transakcyjne
    pub transaction_fees: f64,
    /// Czas trwania pozycji w sekundach
    pub position_duration_seconds: u64,
}

/// 📊 Metryki uczenia się
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LearningMetrics {
    /// Całkowita liczba próbek feedbacku
    pub total_feedback_samples: u64,
    /// Wskaźnik sukcesu (0.0 - 1.0)
    pub success_rate: f64,
    /// Średni czas wykonania zadań (ms)
    pub average_execution_time_ms: f64,
    /// Średni ROI (%)
    pub average_roi_percent: f64,
    /// Liczba wykrytych wzorców
    pub detected_patterns: u32,
    /// Dokładność predykcji (0.0 - 1.0)
    pub prediction_accuracy: f64,
    /// Metryki według typu agenta
    pub agent_type_metrics: HashMap<AgentType, AgentLearningMetrics>,
    /// Ostatnia aktualizacja
    pub last_updated: DateTime<Utc>,
}

/// 🤖 Metryki uczenia się dla typu agenta
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentLearningMetrics {
    /// Liczba zadań wykonanych
    pub tasks_completed: u64,
    /// Wskaźnik sukcesu
    pub success_rate: f64,
    /// Średni czas wykonania
    pub average_execution_time_ms: f64,
    /// Średnia dokładność decyzji
    pub average_decision_accuracy: f64,
    /// Trend wydajności (-1.0 do 1.0)
    pub performance_trend: f64,
}

/// 🧠 Wykryty wzorzec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPattern {
    /// ID wzorca
    pub id: Uuid,
    /// Typ wzorca
    pub pattern_type: PatternType,
    /// Opis wzorca
    pub description: String,
    /// Warunki wystąpienia
    pub conditions: Vec<PatternCondition>,
    /// Przewidywany wynik
    pub predicted_outcome: PredictedOutcome,
    /// Poziom pewności (0.0 - 1.0)
    pub confidence: f64,
    /// Liczba obserwacji
    pub observation_count: u32,
    /// Czas wykrycia
    pub detected_at: DateTime<Utc>,
}

/// 🔍 Typ wzorca
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternType {
    /// Wzorzec sukcesu
    SuccessPattern,
    /// Wzorzec niepowodzenia
    FailurePattern,
    /// Wzorzec rynkowy
    MarketPattern,
    /// Wzorzec wydajności agenta
    AgentPerformancePattern,
    /// Wzorzec czasowy
    TemporalPattern,
}

/// ⚖️ Warunek wzorca
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternCondition {
    /// Pole do sprawdzenia
    pub field: String,
    /// Operator porównania
    pub operator: ComparisonOperator,
    /// Wartość do porównania
    pub value: Value,
    /// Waga warunku (0.0 - 1.0)
    pub weight: f64,
}

/// 🔢 Operator porównania
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Contains,
    InRange,
}

/// 🎯 Przewidywany wynik
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictedOutcome {
    /// Przewidywana prawdopodobieństwo sukcesu
    pub success_probability: f64,
    /// Przewidywany czas wykonania (ms)
    pub estimated_execution_time_ms: u64,
    /// Przewidywany ROI (%)
    pub estimated_roi_percent: f64,
    /// Rekomendowany agent
    pub recommended_agent_type: Option<AgentType>,
}

/// 🔄 Pętla uczenia się
pub struct FeedbackLoop {
    /// Konfiguracja
    config: Arc<Config>,
    /// Magazyn pamięci
    memory_store: Arc<MemoryStore>,
    /// Metryki uczenia się
    metrics: Arc<RwLock<LearningMetrics>>,
    /// Wykryte wzorce
    patterns: Arc<RwLock<Vec<DetectedPattern>>>,
    /// Historia feedbacku
    feedback_history: Arc<RwLock<Vec<FeedbackData>>>,
}

impl FeedbackLoop {
    /// Tworzy nową pętlę uczenia się
    #[instrument(skip(config, memory_store))]
    pub async fn new(config: Arc<Config>, memory_store: Arc<MemoryStore>) -> Result<Self> {
        info!("🔄 Initializing FeedbackLoop...");
        
        Ok(Self {
            config,
            memory_store,
            metrics: Arc::new(RwLock::new(LearningMetrics::default())),
            patterns: Arc::new(RwLock::new(Vec::new())),
            feedback_history: Arc::new(RwLock::new(Vec::new())),
        })
    }
    
    /// Przetwarza wynik zadania i uczy się z niego
    #[instrument(skip(self, task, result))]
    pub async fn process_task_result(&self, task: Task, result: TaskResult) -> Result<()> {
        debug!("🔄 Processing task result for learning: {}", task.id);
        
        // Utwórz dane feedbacku
        let feedback_data = self.create_feedback_data(task, result).await?;
        
        // Zapisz w historii
        {
            let mut history = self.feedback_history.write().await;
            history.push(feedback_data.clone());
            
            // Ogranicz rozmiar historii
            if history.len() > 10000 {
                history.remove(0);
            }
        }
        
        // Zapisz w pamięci długoterminowej
        let memory_entry = MemoryEntry::new(
            MemoryType::LongTerm,
            "feedback_data".to_string(),
            feedback_data.task_id.to_string(),
            serde_json::to_value(&feedback_data)?,
        );
        
        self.memory_store.store(memory_entry).await?;
        
        // Aktualizuj metryki
        self.update_learning_metrics(&feedback_data).await?;
        
        // Wykryj nowe wzorce
        self.detect_patterns().await?;
        
        info!("✅ Task result processed for learning: {}", feedback_data.task_id);
        Ok(())
    }
    
    /// Przewiduje wynik zadania na podstawie wzorców
    #[instrument(skip(self, task))]
    pub async fn predict_task_outcome(&self, task: &Task) -> Result<PredictedOutcome> {
        debug!("🔮 Predicting outcome for task: {}", task.id);
        
        let patterns = self.patterns.read().await;
        let mut best_prediction = PredictedOutcome {
            success_probability: 0.5, // Domyślne 50%
            estimated_execution_time_ms: 30000, // 30 sekund
            estimated_roi_percent: 0.0,
            recommended_agent_type: task.preferred_agent_type.clone(),
        };
        
        let mut best_confidence = 0.0;
        
        // Sprawdź wszystkie wzorce
        for pattern in patterns.iter() {
            if self.pattern_matches_task(pattern, task) {
                if pattern.confidence > best_confidence {
                    best_confidence = pattern.confidence;
                    best_prediction = pattern.predicted_outcome.clone();
                }
            }
        }
        
        debug!("🔮 Prediction completed with confidence: {:.2}", best_confidence);
        Ok(best_prediction)
    }
    
    /// Rekomenduje najlepszego agenta dla zadania
    #[instrument(skip(self, task))]
    pub async fn recommend_agent(&self, task: &Task) -> Result<Option<AgentType>> {
        debug!("🎯 Recommending agent for task: {}", task.task_type);
        
        let prediction = self.predict_task_outcome(task).await?;
        
        // Jeśli mamy rekomendację z wysoką pewnością, użyj jej
        if prediction.success_probability > 0.7 {
            return Ok(prediction.recommended_agent_type);
        }
        
        // W przeciwnym razie sprawdź historyczne wyniki według typu agenta
        let metrics = self.metrics.read().await;
        let mut best_agent_type = None;
        let mut best_score = 0.0;
        
        for (agent_type, agent_metrics) in &metrics.agent_type_metrics {
            // Oblicz wynik na podstawie sukcesu i wydajności
            let score = agent_metrics.success_rate * 0.7 + 
                       (1.0 - agent_metrics.average_execution_time_ms / 60000.0) * 0.3;
            
            if score > best_score {
                best_score = score;
                best_agent_type = Some(agent_type.clone());
            }
        }
        
        debug!("🎯 Agent recommendation: {:?} (score: {:.2})", best_agent_type, best_score);
        Ok(best_agent_type)
    }
    
    /// Pobiera metryki uczenia się
    pub async fn get_learning_metrics(&self) -> LearningMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }
    
    /// Pobiera wykryte wzorce
    pub async fn get_detected_patterns(&self) -> Vec<DetectedPattern> {
        let patterns = self.patterns.read().await;
        patterns.clone()
    }
    
    /// Tworzy dane feedbacku z zadania i wyniku
    async fn create_feedback_data(&self, task: Task, result: TaskResult) -> Result<FeedbackData> {
        // Symulacja warunków rynkowych - w rzeczywistości pobierane z zewnętrznych źródeł
        let market_conditions = MarketConditions {
            volatility: 0.5,
            volume: 1000000.0,
            price_trend: 0.1,
            network_congestion: 0.3,
            average_fee: 0.001,
            block_time_seconds: 0.4,
        };
        
        // Symulacja wyniku finansowego - w rzeczywistości obliczane na podstawie rzeczywistych transakcji
        let financial_outcome = if result.success {
            Some(FinancialOutcome {
                profit_loss_sol: 0.1,
                profit_loss_usd: 10.0,
                roi_percent: 5.0,
                transaction_fees: 0.001,
                position_duration_seconds: (result.completed_at - result.started_at).num_seconds() as u64,
            })
        } else {
            None
        };
        
        Ok(FeedbackData {
            task_id: task.id,
            task_type: task.task_type,
            agent_id: result.agent_id,
            agent_type: AgentType::Strateg, // Należy pobrać z registry
            success: result.success,
            result: result.result,
            error: result.error,
            execution_time_ms: result.metrics.execution_time_ms,
            input_data: task.payload,
            market_conditions,
            financial_outcome,
            created_at: Utc::now(),
            metadata: HashMap::new(),
        })
    }
    
    /// Aktualizuje metryki uczenia się
    async fn update_learning_metrics(&self, feedback: &FeedbackData) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        
        metrics.total_feedback_samples += 1;
        
        // Aktualizuj ogólne metryki
        let success_rate = if feedback.success { 1.0 } else { 0.0 };
        metrics.success_rate = (metrics.success_rate * 0.95) + (success_rate * 0.05);
        
        metrics.average_execution_time_ms = 
            (metrics.average_execution_time_ms * 0.95) + (feedback.execution_time_ms as f64 * 0.05);
        
        if let Some(outcome) = &feedback.financial_outcome {
            metrics.average_roi_percent = 
                (metrics.average_roi_percent * 0.95) + (outcome.roi_percent * 0.05);
        }
        
        // Aktualizuj metryki dla typu agenta
        let agent_metrics = metrics.agent_type_metrics
            .entry(feedback.agent_type.clone())
            .or_insert_with(Default::default);
        
        agent_metrics.tasks_completed += 1;
        agent_metrics.success_rate = (agent_metrics.success_rate * 0.9) + (success_rate * 0.1);
        agent_metrics.average_execution_time_ms = 
            (agent_metrics.average_execution_time_ms * 0.9) + (feedback.execution_time_ms as f64 * 0.1);
        
        metrics.last_updated = Utc::now();
        
        Ok(())
    }
    
    /// Wykrywa nowe wzorce w danych
    async fn detect_patterns(&self) -> Result<()> {
        // Uproszczona implementacja wykrywania wzorców
        // W rzeczywistości używałby zaawansowanych algorytmów ML
        
        let history = self.feedback_history.read().await;
        
        if history.len() < 10 {
            return Ok(()); // Za mało danych
        }
        
        // Przykład: wykryj wzorzec sukcesu dla szybkich zadań
        let fast_successful_tasks: Vec<_> = history
            .iter()
            .filter(|f| f.success && f.execution_time_ms < 5000)
            .collect();
        
        if fast_successful_tasks.len() > 5 {
            let pattern = DetectedPattern {
                id: Uuid::new_v4(),
                pattern_type: PatternType::SuccessPattern,
                description: "Fast execution leads to higher success rate".to_string(),
                conditions: vec![
                    PatternCondition {
                        field: "execution_time_ms".to_string(),
                        operator: ComparisonOperator::LessThan,
                        value: serde_json::Value::Number(5000.into()),
                        weight: 0.8,
                    }
                ],
                predicted_outcome: PredictedOutcome {
                    success_probability: 0.9,
                    estimated_execution_time_ms: 3000,
                    estimated_roi_percent: 5.0,
                    recommended_agent_type: Some(AgentType::Quant),
                },
                confidence: 0.8,
                observation_count: fast_successful_tasks.len() as u32,
                detected_at: Utc::now(),
            };
            
            let mut patterns = self.patterns.write().await;
            
            // Sprawdź czy wzorzec już istnieje
            if !patterns.iter().any(|p| p.pattern_type == pattern.pattern_type && 
                                         p.description == pattern.description) {
                patterns.push(pattern);
                
                let mut metrics = self.metrics.write().await;
                metrics.detected_patterns += 1;
                
                debug!("🔍 New pattern detected: Fast execution success pattern");
            }
        }
        
        Ok(())
    }
    
    /// Sprawdza czy wzorzec pasuje do zadania
    fn pattern_matches_task(&self, pattern: &DetectedPattern, task: &Task) -> bool {
        // Uproszczona implementacja dopasowania wzorca
        // W rzeczywistości używałby bardziej zaawansowanej logiki
        
        for condition in &pattern.conditions {
            match condition.field.as_str() {
                "task_type" => {
                    if let Some(pattern_task_type) = condition.value.as_str() {
                        if task.task_type != pattern_task_type {
                            return false;
                        }
                    }
                }
                "priority" => {
                    // Można dodać więcej warunków
                }
                _ => {}
            }
        }
        
        true
    }
}
