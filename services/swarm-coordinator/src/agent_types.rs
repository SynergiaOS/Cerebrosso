//! 🤖 Agent Types & Roles Definition
//! 
//! Definicje typów agentów w architekturze Hive Mind

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 🎯 Główne typy agentów w systemie Swarmagentic
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentType {
    /// 👑 Agent-Strateg (CEO) - Główny decydent i koordynator
    Strateg,
    /// 🔬 Agent-Analityk - Analiza jakościowa i sentiment
    Analityk,
    /// 🧮 Agent-Quant - Analiza ilościowa i modelowanie
    Quant,
    /// 🛡️ Agent-Nadzorca (Guardian) - Bezpieczeństwo i monitoring
    Nadzorca,
}

/// 🎭 Role agenta w konkretnym zadaniu
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentRole {
    /// Lider zadania
    Leader,
    /// Współpracownik
    Collaborator,
    /// Konsultant
    Consultant,
    /// Observer
    Observer,
}

/// 🧠 Wyspecjalizowane funkcje agenta
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentCapability {
    /// Analiza danych
    Analysis,
    /// Podejmowanie decyzji
    DecisionMaking,
    /// Komunikacja z innymi agentami
    Communication,
    /// Uczenie się z feedbacku
    Learning,
    /// Monitorowanie bezpieczeństwa
    SecurityMonitoring,
    /// Optymalizacja wydajności
    PerformanceOptimization,
    /// Zarządzanie ryzykiem
    RiskManagement,
    /// Analiza sentymentu
    SentimentAnalysis,
    /// Modelowanie matematyczne
    MathematicalModeling,
    /// Wykrywanie anomalii
    AnomalyDetection,
}

/// 🤖 Trait dla wyspecjalizowanych agentów
pub trait SpecializedAgent {
    /// Typ agenta
    fn agent_type(&self) -> AgentType;
    
    /// Główne funkcje agenta
    fn capabilities(&self) -> Vec<AgentCapability>;
    
    /// Waga decyzyjna agenta (0.0 - 1.0)
    fn decision_weight(&self) -> f64;
    
    /// Maksymalna liczba równoczesnych zadań
    fn max_concurrent_tasks(&self) -> usize;
    
    /// Sprawdź czy agent może obsłużyć zadanie
    fn can_handle_task(&self, task_type: &str) -> bool;
}

/// 👑 Agent-Strateg Implementation
#[derive(Debug, Clone)]
pub struct AgentStrateg {
    pub id: Uuid,
    pub performance_score: f64,
}

impl SpecializedAgent for AgentStrateg {
    fn agent_type(&self) -> AgentType {
        AgentType::Strateg
    }
    
    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::DecisionMaking,
            AgentCapability::Communication,
            AgentCapability::Learning,
            AgentCapability::RiskManagement,
        ]
    }
    
    fn decision_weight(&self) -> f64 {
        0.4 // 40% wagi w końcowej decyzji
    }
    
    fn max_concurrent_tasks(&self) -> usize {
        10
    }
    
    fn can_handle_task(&self, task_type: &str) -> bool {
        matches!(task_type, 
            "goal_decomposition" | 
            "task_delegation" | 
            "decision_synthesis" |
            "strategy_planning"
        )
    }
}

/// 🔬 Agent-Analityk Implementation
#[derive(Debug, Clone)]
pub struct AgentAnalityk {
    pub id: Uuid,
    pub performance_score: f64,
}

impl SpecializedAgent for AgentAnalityk {
    fn agent_type(&self) -> AgentType {
        AgentType::Analityk
    }
    
    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::Analysis,
            AgentCapability::SentimentAnalysis,
            AgentCapability::Communication,
        ]
    }
    
    fn decision_weight(&self) -> f64 {
        0.25 // 25% wagi w końcowej decyzji
    }
    
    fn max_concurrent_tasks(&self) -> usize {
        5
    }
    
    fn can_handle_task(&self, task_type: &str) -> bool {
        matches!(task_type,
            "sentiment_analysis" |
            "whitepaper_evaluation" |
            "team_credibility_analysis" |
            "qualitative_assessment"
        )
    }
}

/// 🧮 Agent-Quant Implementation
#[derive(Debug, Clone)]
pub struct AgentQuant {
    pub id: Uuid,
    pub performance_score: f64,
}

impl SpecializedAgent for AgentQuant {
    fn agent_type(&self) -> AgentType {
        AgentType::Quant
    }
    
    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::Analysis,
            AgentCapability::MathematicalModeling,
            AgentCapability::PerformanceOptimization,
            AgentCapability::RiskManagement,
        ]
    }
    
    fn decision_weight(&self) -> f64 {
        0.3 // 30% wagi w końcowej decyzji
    }
    
    fn max_concurrent_tasks(&self) -> usize {
        8
    }
    
    fn can_handle_task(&self, task_type: &str) -> bool {
        matches!(task_type,
            "tokenomics_analysis" |
            "risk_modeling" |
            "fee_optimization" |
            "backtesting" |
            "quantitative_analysis"
        )
    }
}

/// 🛡️ Agent-Nadzorca Implementation
#[derive(Debug, Clone)]
pub struct AgentNadzorca {
    pub id: Uuid,
    pub performance_score: f64,
}

impl SpecializedAgent for AgentNadzorca {
    fn agent_type(&self) -> AgentType {
        AgentType::Nadzorca
    }
    
    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::SecurityMonitoring,
            AgentCapability::AnomalyDetection,
            AgentCapability::RiskManagement,
        ]
    }
    
    fn decision_weight(&self) -> f64 {
        0.05 // 5% wagi, ale może zawetować decyzję
    }
    
    fn max_concurrent_tasks(&self) -> usize {
        3
    }
    
    fn can_handle_task(&self, task_type: &str) -> bool {
        matches!(task_type,
            "threat_monitoring" |
            "security_analysis" |
            "anomaly_detection" |
            "policy_enforcement"
        )
    }
}

impl fmt::Display for AgentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentType::Strateg => write!(f, "👑 Agent-Strateg"),
            AgentType::Analityk => write!(f, "🔬 Agent-Analityk"),
            AgentType::Quant => write!(f, "🧮 Agent-Quant"),
            AgentType::Nadzorca => write!(f, "🛡️ Agent-Nadzorca"),
        }
    }
}
