//! 🔬 Decision Synthesis - Synteza decyzji z raportów agentów
//! 
//! System syntezy końcowych decyzji na podstawie raportów od wyspecjalizowanych agentów

use anyhow::{Result, anyhow};
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{info, debug, instrument};

use crate::{
    config::Config,
    ai_models::AIResponse,
    risk_management::RiskAssessment,
};

/// 🎯 Poziom pewności decyzji
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionConfidence {
    /// Bardzo wysoka pewność (>90%)
    VeryHigh(f64),
    /// Wysoka pewność (70-90%)
    High(f64),
    /// Średnia pewność (50-70%)
    Medium(f64),
    /// Niska pewność (30-50%)
    Low(f64),
    /// Bardzo niska pewność (<30%)
    VeryLow(f64),
}

impl DecisionConfidence {
    /// Tworzy nowy poziom pewności
    pub fn new(value: f64) -> Self {
        match value {
            v if v >= 0.9 => DecisionConfidence::VeryHigh(v),
            v if v >= 0.7 => DecisionConfidence::High(v),
            v if v >= 0.5 => DecisionConfidence::Medium(v),
            v if v >= 0.3 => DecisionConfidence::Low(v),
            v => DecisionConfidence::VeryLow(v),
        }
    }
    
    /// Pobiera wartość numeryczną
    pub fn value(&self) -> f64 {
        match self {
            DecisionConfidence::VeryHigh(v) => *v,
            DecisionConfidence::High(v) => *v,
            DecisionConfidence::Medium(v) => *v,
            DecisionConfidence::Low(v) => *v,
            DecisionConfidence::VeryLow(v) => *v,
        }
    }
    
    /// Sprawdza czy pewność jest wystarczająca
    pub fn is_sufficient(&self, threshold: f64) -> bool {
        self.value() >= threshold
    }
}

/// 🎯 Typ decyzji
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionType {
    /// Decyzja o kupnie
    Buy,
    /// Decyzja o sprzedaży
    Sell,
    /// Decyzja o trzymaniu pozycji
    Hold,
    /// Decyzja o zamknięciu pozycji
    Close,
    /// Decyzja o anulowaniu operacji
    Cancel,
    /// Decyzja o oczekiwaniu
    Wait,
}

/// 🎯 Decyzja strategiczna
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    /// ID decyzji
    pub id: Uuid,
    /// Typ decyzji
    pub decision_type: DecisionType,
    /// Poziom pewności
    pub confidence: DecisionConfidence,
    /// Uzasadnienie decyzji
    pub rationale: DecisionRationale,
    /// Parametry decyzji
    pub parameters: DecisionParameters,
    /// Ocena ryzyka
    pub risk_assessment: RiskAssessment,
    /// Czas podjęcia decyzji
    pub timestamp: DateTime<Utc>,
    /// Metadane decyzji
    pub metadata: HashMap<String, Value>,
}

/// 📝 Uzasadnienie decyzji
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRationale {
    /// Główne uzasadnienie
    pub primary_reason: String,
    /// Dodatkowe czynniki
    pub supporting_factors: Vec<String>,
    /// Czynniki ryzyka
    pub risk_factors: Vec<String>,
    /// Wagi poszczególnych agentów w decyzji
    pub agent_weights: HashMap<String, f64>,
    /// Podsumowanie analiz agentów
    pub agent_summaries: HashMap<String, String>,
}

/// ⚙️ Parametry decyzji
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionParameters {
    /// Rozmiar pozycji (w SOL)
    pub position_size: Option<f64>,
    /// Cena docelowa
    pub target_price: Option<f64>,
    /// Stop loss
    pub stop_loss: Option<f64>,
    /// Take profit
    pub take_profit: Option<f64>,
    /// Timeout decyzji (sekundy)
    pub timeout_seconds: Option<u64>,
    /// Dodatkowe parametry
    pub additional: HashMap<String, Value>,
}

/// 🔬 Syntezator decyzji
pub struct DecisionSynthesizer {
    /// Konfiguracja
    config: Arc<Config>,
    /// Wagi agentów w decyzjach
    agent_weights: HashMap<String, f64>,
    /// Statystyki syntezy
    stats: SynthesisStats,
}

/// 📊 Statystyki syntezy
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SynthesisStats {
    pub total_decisions: u64,
    pub high_confidence_decisions: u64,
    pub low_confidence_decisions: u64,
    pub average_synthesis_time_ms: f64,
    pub decision_accuracy: f64,
    pub decisions_by_type: HashMap<String, u64>,
}

impl DecisionSynthesizer {
    /// Tworzy nowy syntezator decyzji
    #[instrument(skip(config))]
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        info!("🔬 Initializing DecisionSynthesizer...");
        
        // Ustaw wagi agentów zgodnie z konfiguracją
        let mut agent_weights = HashMap::new();
        agent_weights.insert("Strateg".to_string(), config.strategy.decision_weight);
        agent_weights.insert("Analityk".to_string(), 0.25);
        agent_weights.insert("Quant".to_string(), 0.30);
        agent_weights.insert("Nadzorca".to_string(), 0.05);
        
        Ok(Self {
            config,
            agent_weights,
            stats: SynthesisStats::default(),
        })
    }
    
    /// Syntetyzuje decyzję na podstawie odpowiedzi agentów
    #[instrument(skip(self, agent_responses, risk_assessment))]
    pub async fn synthesize_decision(
        &mut self,
        agent_responses: Vec<AIResponse>,
        risk_assessment: RiskAssessment,
    ) -> Result<Decision> {
        debug!("🔬 Synthesizing decision from {} agent responses", agent_responses.len());
        
        let start_time = std::time::Instant::now();
        
        // Analiza odpowiedzi agentów
        let analysis = self.analyze_agent_responses(&agent_responses)?;
        
        // Oblicz ważoną pewność
        let weighted_confidence = self.calculate_weighted_confidence(&agent_responses)?;
        
        // Określ typ decyzji
        let decision_type = self.determine_decision_type(&agent_responses, &risk_assessment)?;
        
        // Oblicz parametry decyzji
        let parameters = self.calculate_decision_parameters(&agent_responses, &risk_assessment)?;
        
        // Utwórz uzasadnienie
        let rationale = self.create_rationale(&agent_responses, &analysis)?;
        
        // Utwórz decyzję
        let decision = Decision {
            id: Uuid::new_v4(),
            decision_type: decision_type.clone(),
            confidence: DecisionConfidence::new(weighted_confidence),
            rationale,
            parameters,
            risk_assessment,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };
        
        // Aktualizuj statystyki
        let duration = start_time.elapsed();
        self.update_stats(&decision, duration.as_millis() as f64);
        
        info!("✅ Decision synthesized: {:?} with confidence {:.2}", 
              decision.decision_type, weighted_confidence);
        
        Ok(decision)
    }
    
    /// Analizuje odpowiedzi agentów
    fn analyze_agent_responses(&self, responses: &[AIResponse]) -> Result<AgentAnalysis> {
        let mut analysis = AgentAnalysis {
            total_responses: responses.len(),
            positive_sentiment: 0,
            negative_sentiment: 0,
            neutral_sentiment: 0,
            average_confidence: 0.0,
            consensus_level: 0.0,
        };
        
        let mut total_confidence = 0.0;
        
        for response in responses {
            total_confidence += response.confidence;
            
            // Analiza sentymentu na podstawie treści odpowiedzi
            if response.content.contains("buy") || response.content.contains("positive") {
                analysis.positive_sentiment += 1;
            } else if response.content.contains("sell") || response.content.contains("negative") {
                analysis.negative_sentiment += 1;
            } else {
                analysis.neutral_sentiment += 1;
            }
        }
        
        analysis.average_confidence = total_confidence / responses.len() as f64;
        
        // Oblicz poziom konsensusu
        let max_sentiment = analysis.positive_sentiment
            .max(analysis.negative_sentiment)
            .max(analysis.neutral_sentiment);
        analysis.consensus_level = max_sentiment as f64 / responses.len() as f64;
        
        Ok(analysis)
    }
    
    /// Oblicza ważoną pewność na podstawie wag agentów
    fn calculate_weighted_confidence(&self, responses: &[AIResponse]) -> Result<f64> {
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;
        
        for response in responses {
            let weight = self.agent_weights
                .get(&response.agent_type)
                .unwrap_or(&0.25); // Domyślna waga
            
            weighted_sum += response.confidence * weight;
            total_weight += weight;
        }
        
        if total_weight == 0.0 {
            return Err(anyhow!("No valid agent weights found"));
        }
        
        Ok(weighted_sum / total_weight)
    }
    
    /// Określa typ decyzji na podstawie odpowiedzi agentów
    fn determine_decision_type(
        &self,
        responses: &[AIResponse],
        risk_assessment: &RiskAssessment,
    ) -> Result<DecisionType> {
        // Prosta logika decyzyjna - można rozszerzyć
        let positive_count = responses.iter()
            .filter(|r| r.content.contains("buy") || r.content.contains("positive"))
            .count();
        
        let negative_count = responses.iter()
            .filter(|r| r.content.contains("sell") || r.content.contains("negative"))
            .count();
        
        // Sprawdź ograniczenia ryzyka
        if risk_assessment.risk_level.is_high() {
            return Ok(DecisionType::Wait);
        }
        
        if positive_count > negative_count {
            Ok(DecisionType::Buy)
        } else if negative_count > positive_count {
            Ok(DecisionType::Sell)
        } else {
            Ok(DecisionType::Hold)
        }
    }
    
    /// Oblicza parametry decyzji
    fn calculate_decision_parameters(
        &self,
        _responses: &[AIResponse],
        risk_assessment: &RiskAssessment,
    ) -> Result<DecisionParameters> {
        Ok(DecisionParameters {
            position_size: Some(risk_assessment.recommended_position_size),
            target_price: None,
            stop_loss: Some(risk_assessment.stop_loss_price),
            take_profit: Some(risk_assessment.take_profit_price),
            timeout_seconds: Some(300), // 5 minut
            additional: HashMap::new(),
        })
    }
    
    /// Tworzy uzasadnienie decyzji
    fn create_rationale(
        &self,
        responses: &[AIResponse],
        analysis: &AgentAnalysis,
    ) -> Result<DecisionRationale> {
        let primary_reason = format!(
            "Decision based on {} agent responses with {:.1}% consensus",
            responses.len(),
            analysis.consensus_level * 100.0
        );
        
        let supporting_factors = vec![
            format!("Average confidence: {:.2}", analysis.average_confidence),
            format!("Positive sentiment: {}", analysis.positive_sentiment),
            format!("Risk assessment completed", ),
        ];
        
        let mut agent_summaries = HashMap::new();
        for response in responses {
            agent_summaries.insert(
                response.agent_type.clone(),
                response.content.chars().take(100).collect::<String>() + "...",
            );
        }
        
        Ok(DecisionRationale {
            primary_reason,
            supporting_factors,
            risk_factors: vec!["Market volatility".to_string()],
            agent_weights: self.agent_weights.clone(),
            agent_summaries,
        })
    }
    
    /// Aktualizuje statystyki syntezy
    fn update_stats(&mut self, decision: &Decision, duration_ms: f64) {
        self.stats.total_decisions += 1;
        
        if decision.confidence.value() >= 0.7 {
            self.stats.high_confidence_decisions += 1;
        } else {
            self.stats.low_confidence_decisions += 1;
        }
        
        // Aktualizuj średni czas syntezy
        let total = self.stats.total_decisions as f64;
        self.stats.average_synthesis_time_ms = 
            (self.stats.average_synthesis_time_ms * (total - 1.0) + duration_ms) / total;
        
        // Aktualizuj statystyki według typu
        let decision_type_str = format!("{:?}", decision.decision_type);
        *self.stats.decisions_by_type
            .entry(decision_type_str)
            .or_insert(0) += 1;
    }
    
    /// Pobiera statystyki syntezy
    pub fn get_stats(&self) -> &SynthesisStats {
        &self.stats
    }
}

/// 📊 Analiza odpowiedzi agentów
#[derive(Debug, Clone)]
struct AgentAnalysis {
    pub total_responses: usize,
    pub positive_sentiment: usize,
    pub negative_sentiment: usize,
    pub neutral_sentiment: usize,
    pub average_confidence: f64,
    pub consensus_level: f64,
}
