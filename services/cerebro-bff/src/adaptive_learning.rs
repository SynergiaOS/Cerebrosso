//! 🧠 Adaptive Learning Engine - AI Parameter Optimization & Performance Analytics
//! 
//! System uczenia adaptacyjnego, który analizuje wyniki trading i optymalizuje
//! parametry AI agentów w czasie rzeczywistym dla maksymalizacji ROI.

use crate::config::Config;
use crate::ai_agent::{AIAgent, AgentType};
use crate::feedback_system::{FeedbackSystem, AgentPerformance, TradeFeedback};
use crate::paper_trading::{PaperTradingEngine, PortfolioPerformance};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn, debug, error};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LearningConfig {
    pub learning_rate: f64,
    pub min_samples_for_optimization: u32,
    pub optimization_interval_hours: u64,
    pub confidence_calibration_window: u32,
    pub performance_decay_factor: f64,
    pub risk_tolerance_adjustment_rate: f64,
    pub enable_auto_optimization: bool,
    pub enable_confidence_calibration: bool,
    pub enable_parameter_bounds: bool,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.1,
            min_samples_for_optimization: 50,
            optimization_interval_hours: 6,
            confidence_calibration_window: 100,
            performance_decay_factor: 0.95,
            risk_tolerance_adjustment_rate: 0.05,
            enable_auto_optimization: true,
            enable_confidence_calibration: true,
            enable_parameter_bounds: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentOptimizationState {
    pub agent_type: AgentType,
    pub current_parameters: HashMap<String, f64>,
    pub parameter_bounds: HashMap<String, (f64, f64)>,
    pub performance_history: Vec<PerformanceSnapshot>,
    pub last_optimization: DateTime<Utc>,
    pub optimization_count: u32,
    pub confidence_calibration: ConfidenceCalibration,
    pub learning_momentum: HashMap<String, f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PerformanceSnapshot {
    pub timestamp: DateTime<Utc>,
    pub roi: f64,
    pub win_rate: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub confidence_accuracy: f64,
    pub total_trades: u32,
    pub parameters: HashMap<String, f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfidenceCalibration {
    pub calibration_curve: Vec<CalibrationPoint>,
    pub brier_score: f64,
    pub reliability: f64,
    pub resolution: f64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CalibrationPoint {
    pub predicted_confidence: f64,
    pub actual_success_rate: f64,
    pub sample_count: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OptimizationResult {
    pub agent_type: AgentType,
    pub old_parameters: HashMap<String, f64>,
    pub new_parameters: HashMap<String, f64>,
    pub expected_improvement: f64,
    pub confidence: f64,
    pub optimization_method: String,
    pub timestamp: DateTime<Utc>,
}

pub struct AdaptiveLearningEngine {
    config: Arc<Config>,
    learning_config: LearningConfig,
    db_pool: PgPool,
    feedback_system: Arc<FeedbackSystem>,
    paper_trading: Arc<PaperTradingEngine>,
    ai_agent: Arc<AIAgent>,
    optimization_states: Arc<RwLock<HashMap<AgentType, AgentOptimizationState>>>,
    optimization_history: Arc<RwLock<Vec<OptimizationResult>>>,
    last_global_optimization: Arc<RwLock<Instant>>,
}

impl AdaptiveLearningEngine {
    /// 🚀 Initialize adaptive learning engine
    pub async fn new(
        config: Arc<Config>,
        db_pool: PgPool,
        feedback_system: Arc<FeedbackSystem>,
        paper_trading: Arc<PaperTradingEngine>,
        ai_agent: Arc<AIAgent>,
    ) -> Result<Self> {
        info!("🧠 Initializing Adaptive Learning Engine v2.0");
        
        let learning_config = LearningConfig::default();
        let optimization_states = Arc::new(RwLock::new(HashMap::new()));
        let optimization_history = Arc::new(RwLock::new(Vec::new()));
        let last_global_optimization = Arc::new(RwLock::new(Instant::now()));
        
        let engine = AdaptiveLearningEngine {
            config,
            learning_config,
            db_pool,
            feedback_system,
            paper_trading,
            ai_agent,
            optimization_states,
            optimization_history,
            last_global_optimization,
        };
        
        // Initialize optimization states for all agent types
        engine.initialize_optimization_states().await?;
        
        // Start background optimization task
        if engine.learning_config.enable_auto_optimization {
            engine.start_background_optimization().await;
        }
        
        info!("✅ Adaptive Learning Engine initialized successfully");
        Ok(engine)
    }
    
    /// 🎯 Process new feedback and trigger learning if needed
    pub async fn process_feedback(&self, feedback: &TradeFeedback) -> Result<()> {
        debug!("🎯 Processing feedback for agent: {:?}", feedback.agent_type);
        
        // Update performance history
        self.update_performance_history(feedback).await?;
        
        // Update confidence calibration
        if self.learning_config.enable_confidence_calibration {
            self.update_confidence_calibration(feedback).await?;
        }
        
        // Check if optimization is needed
        if self.should_optimize_agent(feedback.agent_type.clone()).await? {
            self.optimize_agent_parameters(feedback.agent_type.clone()).await?;
        }
        
        Ok(())
    }
    
    /// 📊 Update performance history for agent
    async fn update_performance_history(&self, feedback: &TradeFeedback) -> Result<()> {
        let mut states = self.optimization_states.write().await;
        
        if let Some(state) = states.get_mut(&feedback.agent_type) {
            // Get current agent performance
            let agent_performance = self.feedback_system
                .get_agent_performance(feedback.agent_type.clone())
                .await?
                .unwrap_or_else(|| AgentPerformance {
                    agent_type: feedback.agent_type.clone(),
                    success_rate: 0.0,
                    avg_roi: 0.0,
                    total_trades: 0,
                    profitable_trades: 0,
                    avg_latency_ms: 0.0,
                    confidence_calibration: 0.0,
                    sharpe_ratio: None,
                    max_drawdown: 0.0,
                    optimal_parameters: HashMap::new(),
                    last_updated: chrono::Utc::now(),
                });
            
            // Create performance snapshot
            let snapshot = PerformanceSnapshot {
                timestamp: Utc::now(),
                roi: agent_performance.avg_roi,
                win_rate: agent_performance.success_rate,
                sharpe_ratio: agent_performance.sharpe_ratio.unwrap_or(0.0),
                max_drawdown: agent_performance.max_drawdown,
                confidence_accuracy: agent_performance.confidence_calibration,
                total_trades: agent_performance.total_trades as u32,
                parameters: state.current_parameters.clone(),
            };
            
            // Add to history with decay
            state.performance_history.push(snapshot);
            
            // Keep only recent history (sliding window)
            let max_history = 1000;
            if state.performance_history.len() > max_history {
                state.performance_history.remove(0);
            }
            
            debug!("📊 Updated performance history for {:?}: {} samples", 
                   feedback.agent_type, state.performance_history.len());
        }
        
        Ok(())
    }
    
    /// 🎯 Update confidence calibration
    async fn update_confidence_calibration(&self, feedback: &TradeFeedback) -> Result<()> {
        let mut states = self.optimization_states.write().await;
        
        if let Some(state) = states.get_mut(&feedback.agent_type) {
            let predicted_confidence = feedback.performance_metrics.confidence_accuracy;
            let actual_success = feedback.performance_metrics.pnl > 0.0;
            
            // Find or create calibration bin
            let confidence_bin = (predicted_confidence * 10.0).floor() / 10.0;
            
            let mut found_bin = false;
            for point in &mut state.confidence_calibration.calibration_curve {
                if (point.predicted_confidence - confidence_bin).abs() < 0.05 {
                    // Update existing bin
                    let total_samples = point.sample_count + 1;
                    let new_success_rate = (point.actual_success_rate * point.sample_count as f64 + 
                                          if actual_success { 1.0 } else { 0.0 }) / total_samples as f64;
                    
                    point.actual_success_rate = new_success_rate;
                    point.sample_count = total_samples;
                    found_bin = true;
                    break;
                }
            }
            
            if !found_bin {
                // Create new calibration point
                state.confidence_calibration.calibration_curve.push(CalibrationPoint {
                    predicted_confidence: confidence_bin,
                    actual_success_rate: if actual_success { 1.0 } else { 0.0 },
                    sample_count: 1,
                });
            }
            
            // Update calibration metrics
            self.calculate_calibration_metrics(&mut state.confidence_calibration);
            state.confidence_calibration.last_updated = Utc::now();
            
            debug!("🎯 Updated confidence calibration for {:?}", feedback.agent_type);
        }
        
        Ok(())
    }
    
    /// 📈 Calculate calibration metrics (Brier score, reliability, resolution)
    fn calculate_calibration_metrics(&self, calibration: &mut ConfidenceCalibration) {
        if calibration.calibration_curve.is_empty() {
            return;
        }
        
        let mut brier_score = 0.0;
        let mut reliability = 0.0;
        let mut resolution = 0.0;
        let mut total_samples = 0;
        let mut weighted_success_rate = 0.0;
        
        // Calculate overall success rate
        for point in &calibration.calibration_curve {
            total_samples += point.sample_count;
            weighted_success_rate += point.actual_success_rate * point.sample_count as f64;
        }
        
        if total_samples > 0 {
            weighted_success_rate /= total_samples as f64;
            
            // Calculate Brier score and reliability
            for point in &calibration.calibration_curve {
                let weight = point.sample_count as f64 / total_samples as f64;
                
                // Brier score component
                brier_score += weight * (point.predicted_confidence - point.actual_success_rate).powi(2);
                
                // Reliability component
                reliability += weight * (point.actual_success_rate - point.predicted_confidence).powi(2);
                
                // Resolution component
                resolution += weight * (point.actual_success_rate - weighted_success_rate).powi(2);
            }
        }
        
        calibration.brier_score = brier_score;
        calibration.reliability = reliability;
        calibration.resolution = resolution;
    }
    
    /// 🤔 Check if agent should be optimized
    async fn should_optimize_agent(&self, agent_type: AgentType) -> Result<bool> {
        let states = self.optimization_states.read().await;
        
        if let Some(state) = states.get(&agent_type) {
            // Check minimum samples
            if state.performance_history.len() < self.learning_config.min_samples_for_optimization as usize {
                return Ok(false);
            }
            
            // Check time since last optimization
            let time_since_optimization = Utc::now() - state.last_optimization;
            if time_since_optimization.num_hours() < self.learning_config.optimization_interval_hours as i64 {
                return Ok(false);
            }
            
            // Check if performance is declining
            let recent_performance = self.calculate_recent_performance(state);
            let historical_performance = self.calculate_historical_performance(state);
            
            // Trigger optimization if performance declined by more than 5%
            let performance_decline = historical_performance - recent_performance;
            if performance_decline > 0.05 {
                info!("🤔 Performance decline detected for {:?}: {:.2}%", 
                      agent_type, performance_decline * 100.0);
                return Ok(true);
            }
            
            // Also optimize periodically even if performance is stable
            if time_since_optimization.num_hours() >= (self.learning_config.optimization_interval_hours * 2) as i64 {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// 📊 Calculate recent performance (last 20% of samples)
    fn calculate_recent_performance(&self, state: &AgentOptimizationState) -> f64 {
        let recent_count = (state.performance_history.len() as f64 * 0.2).max(10.0) as usize;
        let recent_samples = state.performance_history.iter()
            .rev()
            .take(recent_count)
            .collect::<Vec<_>>();
        
        if recent_samples.is_empty() {
            return 0.0;
        }
        
        // Weighted performance score
        let mut total_score = 0.0;
        for sample in recent_samples {
            let score = sample.roi * 0.4 + 
                       sample.win_rate * 0.3 + 
                       sample.sharpe_ratio * 0.2 + 
                       (1.0 - sample.max_drawdown) * 0.1;
            total_score += score;
        }
        
        total_score / recent_count as f64
    }
    
    /// 📈 Calculate historical performance (excluding recent samples)
    fn calculate_historical_performance(&self, state: &AgentOptimizationState) -> f64 {
        let recent_count = (state.performance_history.len() as f64 * 0.2).max(10.0) as usize;
        let historical_samples = state.performance_history.iter()
            .rev()
            .skip(recent_count)
            .take(100) // Last 100 historical samples
            .collect::<Vec<_>>();
        
        if historical_samples.is_empty() {
            return 0.0;
        }
        
        // Weighted performance score
        let mut total_score = 0.0;
        for sample in &historical_samples {
            let score = sample.roi * 0.4 + 
                       sample.win_rate * 0.3 + 
                       sample.sharpe_ratio * 0.2 + 
                       (1.0 - sample.max_drawdown) * 0.1;
            total_score += score;
        }
        
        total_score / historical_samples.len() as f64
    }

    /// 🎯 Optimize agent parameters using gradient-based approach
    pub async fn optimize_agent_parameters(&self, agent_type: AgentType) -> Result<OptimizationResult> {
        info!("🎯 Starting parameter optimization for agent: {:?}", agent_type);

        let mut states = self.optimization_states.write().await;
        let state = states.get_mut(&agent_type)
            .ok_or_else(|| anyhow::anyhow!("Agent state not found: {:?}", agent_type))?;

        let old_parameters = state.current_parameters.clone();

        // Use different optimization strategies based on agent type
        let optimization_result = match agent_type {
            AgentType::FastDecision => self.optimize_fast_decision_agent(state).await?,
            AgentType::ContextAnalysis => self.optimize_context_analysis_agent(state).await?,
            AgentType::RiskAssessment => self.optimize_risk_assessment_agent(state).await?,
            AgentType::DeepAnalysis => self.optimize_deep_analysis_agent(state).await?,
        };

        // Apply parameter bounds if enabled
        if self.learning_config.enable_parameter_bounds {
            self.apply_parameter_bounds(state);
        }

        // Update optimization state
        state.last_optimization = Utc::now();
        state.optimization_count += 1;

        // Create optimization result
        let result = OptimizationResult {
            agent_type: agent_type.clone(),
            old_parameters,
            new_parameters: state.current_parameters.clone(),
            expected_improvement: optimization_result.expected_improvement,
            confidence: optimization_result.confidence,
            optimization_method: optimization_result.method,
            timestamp: Utc::now(),
        };

        // Store optimization history
        self.optimization_history.write().await.push(result.clone());

        // Apply new parameters to AI agent
        self.apply_parameters_to_agent(agent_type.clone(), &state.current_parameters).await?;

        info!("✅ Parameter optimization completed for {:?}: expected improvement {:.2}%",
              agent_type, result.expected_improvement * 100.0);

        Ok(result)
    }

    /// ⚡ Optimize Fast Decision Agent parameters
    async fn optimize_fast_decision_agent(&self, state: &mut AgentOptimizationState) -> Result<InternalOptimizationResult> {
        // Fast Decision Agent focuses on speed and confidence thresholds
        let mut new_params = state.current_parameters.clone();

        // Analyze confidence vs success rate correlation
        let confidence_correlation = self.calculate_confidence_success_correlation(state);

        // Adjust confidence threshold based on calibration
        if let Some(current_threshold) = new_params.get_mut("confidence_threshold") {
            if confidence_correlation > 0.7 {
                // High correlation - can be more aggressive
                *current_threshold = (*current_threshold * 0.95).max(0.3);
            } else if confidence_correlation < 0.3 {
                // Low correlation - be more conservative
                *current_threshold = (*current_threshold * 1.05).min(0.8);
            }
        }

        // Adjust urgency multiplier based on win rate
        let recent_win_rate = self.calculate_recent_win_rate(state);
        if let Some(urgency_multiplier) = new_params.get_mut("urgency_multiplier") {
            if recent_win_rate > 0.6 {
                *urgency_multiplier = (*urgency_multiplier * 1.02).min(2.0);
            } else if recent_win_rate < 0.4 {
                *urgency_multiplier = (*urgency_multiplier * 0.98).max(0.5);
            }
        }

        // Update momentum
        self.update_learning_momentum(state, &new_params);
        state.current_parameters = new_params;

        Ok(InternalOptimizationResult {
            expected_improvement: (recent_win_rate - 0.5).abs() * 0.1,
            confidence: confidence_correlation,
            method: "confidence_threshold_optimization".to_string(),
        })
    }

    /// 🧠 Optimize Context Analysis Agent parameters
    async fn optimize_context_analysis_agent(&self, state: &mut AgentOptimizationState) -> Result<InternalOptimizationResult> {
        let mut new_params = state.current_parameters.clone();

        // Analyze context quality vs performance correlation
        let context_performance = self.analyze_context_performance_correlation(state);

        // Adjust context window size
        if let Some(context_window) = new_params.get_mut("context_window_size") {
            if context_performance.correlation > 0.6 {
                // Good context correlation - can expand window
                *context_window = (*context_window * 1.1).min(4096.0);
            } else {
                // Poor correlation - reduce window for focus
                *context_window = (*context_window * 0.9).max(512.0);
            }
        }

        // Adjust sentiment weight based on market conditions
        let market_volatility = self.calculate_recent_market_volatility(state);
        if let Some(sentiment_weight) = new_params.get_mut("sentiment_weight") {
            if market_volatility > 0.05 {
                // High volatility - reduce sentiment weight
                *sentiment_weight = (*sentiment_weight * 0.95).max(0.1);
            } else {
                // Low volatility - increase sentiment weight
                *sentiment_weight = (*sentiment_weight * 1.05).min(0.8);
            }
        }

        self.update_learning_momentum(state, &new_params);
        state.current_parameters = new_params;

        Ok(InternalOptimizationResult {
            expected_improvement: context_performance.correlation * 0.15,
            confidence: context_performance.confidence,
            method: "context_window_optimization".to_string(),
        })
    }

    /// ⚠️ Optimize Risk Assessment Agent parameters
    async fn optimize_risk_assessment_agent(&self, state: &mut AgentOptimizationState) -> Result<InternalOptimizationResult> {
        let mut new_params = state.current_parameters.clone();

        // Analyze risk vs actual losses correlation
        let risk_accuracy = self.calculate_risk_prediction_accuracy(state);

        // Adjust risk thresholds based on accuracy
        if let Some(risk_threshold) = new_params.get_mut("risk_threshold") {
            if risk_accuracy > 0.7 {
                // Good risk prediction - can be more aggressive
                *risk_threshold = (*risk_threshold * 1.02).min(0.9);
            } else {
                // Poor risk prediction - be more conservative
                *risk_threshold = (*risk_threshold * 0.98).max(0.3);
            }
        }

        // Adjust position sizing based on drawdown history
        let max_drawdown = self.calculate_recent_max_drawdown(state);
        if let Some(position_multiplier) = new_params.get_mut("position_size_multiplier") {
            if max_drawdown > 0.1 {
                // High drawdown - reduce position sizes
                *position_multiplier = (*position_multiplier * 0.9).max(0.1);
            } else if max_drawdown < 0.03 {
                // Low drawdown - can increase position sizes
                *position_multiplier = (*position_multiplier * 1.05).min(2.0);
            }
        }

        self.update_learning_momentum(state, &new_params);
        state.current_parameters = new_params;

        Ok(InternalOptimizationResult {
            expected_improvement: risk_accuracy * 0.2,
            confidence: risk_accuracy,
            method: "risk_threshold_optimization".to_string(),
        })
    }

    /// 🔬 Optimize Deep Analysis Agent parameters
    async fn optimize_deep_analysis_agent(&self, state: &mut AgentOptimizationState) -> Result<InternalOptimizationResult> {
        let mut new_params = state.current_parameters.clone();

        // Analyze long-term strategy effectiveness
        let strategy_effectiveness = self.calculate_strategy_effectiveness(state);

        // Adjust analysis depth based on effectiveness
        if let Some(analysis_depth) = new_params.get_mut("analysis_depth") {
            if strategy_effectiveness > 0.6 {
                // Effective analysis - can go deeper
                *analysis_depth = (*analysis_depth * 1.05).min(10.0);
            } else {
                // Less effective - simplify analysis
                *analysis_depth = (*analysis_depth * 0.95).max(3.0);
            }
        }

        // Adjust long-term outlook weight
        let long_term_accuracy = self.calculate_long_term_prediction_accuracy(state);
        if let Some(outlook_weight) = new_params.get_mut("long_term_outlook_weight") {
            if long_term_accuracy > 0.5 {
                *outlook_weight = (*outlook_weight * 1.02).min(0.8);
            } else {
                *outlook_weight = (*outlook_weight * 0.98).max(0.2);
            }
        }

        self.update_learning_momentum(state, &new_params);
        state.current_parameters = new_params;

        Ok(InternalOptimizationResult {
            expected_improvement: strategy_effectiveness * 0.1,
            confidence: long_term_accuracy,
            method: "strategy_depth_optimization".to_string(),
        })
    }

    /// 📊 Helper methods for performance analysis
    fn calculate_confidence_success_correlation(&self, state: &AgentOptimizationState) -> f64 {
        if state.performance_history.len() < 10 {
            return 0.5; // Default neutral correlation
        }

        // Calculate correlation between confidence and actual success
        let recent_samples = state.performance_history.iter()
            .rev()
            .take(50)
            .collect::<Vec<_>>();

        let mut correlation_sum = 0.0;
        for sample in recent_samples {
            let confidence_score = sample.confidence_accuracy;
            let success_score = if sample.roi > 0.0 { 1.0 } else { 0.0 };
            correlation_sum += confidence_score * success_score;
        }

        (correlation_sum / 50.0).min(1.0).max(0.0)
    }

    fn calculate_recent_win_rate(&self, state: &AgentOptimizationState) -> f64 {
        let recent_samples = state.performance_history.iter()
            .rev()
            .take(20)
            .collect::<Vec<_>>();

        if recent_samples.is_empty() {
            return 0.5;
        }

        recent_samples.iter().map(|s| s.win_rate).sum::<f64>() / recent_samples.len() as f64
    }

    fn analyze_context_performance_correlation(&self, state: &AgentOptimizationState) -> ContextPerformanceAnalysis {
        // Simplified analysis - in real implementation would analyze context quality metrics
        let recent_performance = self.calculate_recent_performance(state);

        ContextPerformanceAnalysis {
            correlation: recent_performance.min(1.0).max(0.0),
            confidence: if state.performance_history.len() > 30 { 0.8 } else { 0.5 },
        }
    }

    fn calculate_recent_market_volatility(&self, state: &AgentOptimizationState) -> f64 {
        let recent_samples = state.performance_history.iter()
            .rev()
            .take(10)
            .collect::<Vec<_>>();

        if recent_samples.len() < 2 {
            return 0.02; // Default volatility
        }

        let roi_values: Vec<f64> = recent_samples.iter().map(|s| s.roi).collect();
        let mean_roi = roi_values.iter().sum::<f64>() / roi_values.len() as f64;

        let variance = roi_values.iter()
            .map(|roi| (roi - mean_roi).powi(2))
            .sum::<f64>() / roi_values.len() as f64;

        variance.sqrt()
    }

    fn calculate_risk_prediction_accuracy(&self, state: &AgentOptimizationState) -> f64 {
        // Analyze how well risk predictions matched actual outcomes
        let recent_samples = state.performance_history.iter()
            .rev()
            .take(30)
            .collect::<Vec<_>>();

        if recent_samples.is_empty() {
            return 0.5;
        }

        let mut accuracy_sum = 0.0;
        for sample in &recent_samples {
            // High risk should correlate with low ROI and vice versa
            let risk_prediction = 1.0 - sample.confidence_accuracy; // Inverse of confidence
            let actual_risk = if sample.roi < 0.0 { 1.0 } else { 0.0 };

            // Calculate accuracy (1 - absolute difference)
            accuracy_sum += 1.0 - (risk_prediction - actual_risk).abs();
        }

        (accuracy_sum / recent_samples.len() as f64).min(1.0).max(0.0)
    }

    fn calculate_recent_max_drawdown(&self, state: &AgentOptimizationState) -> f64 {
        state.performance_history.iter()
            .rev()
            .take(20)
            .map(|s| s.max_drawdown)
            .fold(0.0, f64::max)
    }

    fn calculate_strategy_effectiveness(&self, state: &AgentOptimizationState) -> f64 {
        if state.performance_history.len() < 20 {
            return 0.5;
        }

        // Compare recent performance to historical average
        let recent_performance = self.calculate_recent_performance(state);
        let historical_performance = self.calculate_historical_performance(state);

        // Effectiveness is how much better recent performance is
        let effectiveness = (recent_performance / historical_performance.max(0.01)).min(2.0);
        (effectiveness - 1.0).max(0.0).min(1.0)
    }

    fn calculate_long_term_prediction_accuracy(&self, state: &AgentOptimizationState) -> f64 {
        // Simplified - would analyze long-term predictions vs actual outcomes
        let recent_sharpe = state.performance_history.iter()
            .rev()
            .take(10)
            .map(|s| s.sharpe_ratio)
            .sum::<f64>() / 10.0;

        (recent_sharpe + 1.0) / 3.0 // Normalize Sharpe ratio to 0-1 range
    }

    /// 🔄 Update learning momentum for parameter changes
    fn update_learning_momentum(&self, state: &mut AgentOptimizationState, new_params: &HashMap<String, f64>) {
        for (param_name, new_value) in new_params {
            if let Some(old_value) = state.current_parameters.get(param_name) {
                let change = new_value - old_value;
                let momentum = state.learning_momentum.entry(param_name.clone()).or_insert(0.0);

                // Update momentum with decay
                *momentum = *momentum * 0.9 + change * self.learning_config.learning_rate;

                // Apply momentum to parameter change
                if momentum.abs() > 0.001 {
                    let momentum_adjustment = *momentum * 0.1;
                    if let Some(param) = state.current_parameters.get_mut(param_name) {
                        *param += momentum_adjustment;
                    }
                }
            }
        }
    }

    /// 🎯 Apply parameter bounds to prevent extreme values
    fn apply_parameter_bounds(&self, state: &mut AgentOptimizationState) {
        for (param_name, value) in &mut state.current_parameters {
            if let Some((min_val, max_val)) = state.parameter_bounds.get(param_name) {
                *value = value.max(*min_val).min(*max_val);
            }
        }
    }

    /// 🔧 Apply optimized parameters to AI agent
    async fn apply_parameters_to_agent(&self, agent_type: AgentType, parameters: &HashMap<String, f64>) -> Result<()> {
        // This would update the AI agent's configuration
        // For now, we'll just log the changes
        info!("🔧 Applying optimized parameters to {:?}:", agent_type);
        for (param, value) in parameters {
            debug!("  {}: {:.4}", param, value);
        }

        // In a real implementation, this would call methods on the AI agent
        // to update its internal parameters

        Ok(())
    }

    /// 🚀 Initialize optimization states for all agent types
    async fn initialize_optimization_states(&self) -> Result<()> {
        let mut states = self.optimization_states.write().await;

        for agent_type in [AgentType::FastDecision, AgentType::ContextAnalysis,
                          AgentType::RiskAssessment, AgentType::DeepAnalysis] {
            let (default_params, param_bounds) = self.get_default_parameters(&agent_type);

            let state = AgentOptimizationState {
                agent_type: agent_type.clone(),
                current_parameters: default_params,
                parameter_bounds: param_bounds,
                performance_history: Vec::new(),
                last_optimization: Utc::now(),
                optimization_count: 0,
                confidence_calibration: ConfidenceCalibration {
                    calibration_curve: Vec::new(),
                    brier_score: 0.0,
                    reliability: 0.0,
                    resolution: 0.0,
                    last_updated: Utc::now(),
                },
                learning_momentum: HashMap::new(),
            };

            states.insert(agent_type, state);
        }

        info!("🚀 Initialized optimization states for {} agents", states.len());
        Ok(())
    }

    /// 📋 Get default parameters and bounds for each agent type
    fn get_default_parameters(&self, agent_type: &AgentType) -> (HashMap<String, f64>, HashMap<String, (f64, f64)>) {
        let mut params = HashMap::new();
        let mut bounds = HashMap::new();

        match agent_type {
            AgentType::FastDecision => {
                params.insert("confidence_threshold".to_string(), 0.6);
                params.insert("urgency_multiplier".to_string(), 1.0);
                params.insert("speed_weight".to_string(), 0.8);

                bounds.insert("confidence_threshold".to_string(), (0.3, 0.9));
                bounds.insert("urgency_multiplier".to_string(), (0.5, 2.0));
                bounds.insert("speed_weight".to_string(), (0.5, 1.0));
            }
            AgentType::ContextAnalysis => {
                params.insert("context_window_size".to_string(), 2048.0);
                params.insert("sentiment_weight".to_string(), 0.4);
                params.insert("trend_weight".to_string(), 0.6);

                bounds.insert("context_window_size".to_string(), (512.0, 4096.0));
                bounds.insert("sentiment_weight".to_string(), (0.1, 0.8));
                bounds.insert("trend_weight".to_string(), (0.2, 0.9));
            }
            AgentType::RiskAssessment => {
                params.insert("risk_threshold".to_string(), 0.7);
                params.insert("position_size_multiplier".to_string(), 1.0);
                params.insert("volatility_weight".to_string(), 0.5);

                bounds.insert("risk_threshold".to_string(), (0.3, 0.9));
                bounds.insert("position_size_multiplier".to_string(), (0.1, 2.0));
                bounds.insert("volatility_weight".to_string(), (0.2, 0.8));
            }
            AgentType::DeepAnalysis => {
                params.insert("analysis_depth".to_string(), 5.0);
                params.insert("long_term_outlook_weight".to_string(), 0.3);
                params.insert("strategy_complexity".to_string(), 0.7);

                bounds.insert("analysis_depth".to_string(), (3.0, 10.0));
                bounds.insert("long_term_outlook_weight".to_string(), (0.1, 0.8));
                bounds.insert("strategy_complexity".to_string(), (0.3, 1.0));
            }
        }

        (params, bounds)
    }

    /// 🔄 Start background optimization task
    async fn start_background_optimization(&self) {
        let optimization_states = self.optimization_states.clone();
        let learning_config = self.learning_config.clone();
        let last_global_optimization = self.last_global_optimization.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(3600)); // Check every hour

            loop {
                interval.tick().await;

                let last_opt = *last_global_optimization.read().await;
                let time_since_last = last_opt.elapsed();

                if time_since_last >= Duration::from_secs(learning_config.optimization_interval_hours * 3600) {
                    debug!("🔄 Background optimization check triggered");

                    // Check each agent for optimization needs
                    let states = optimization_states.read().await;
                    for (agent_type, state) in states.iter() {
                        if state.performance_history.len() >= learning_config.min_samples_for_optimization as usize {
                            info!("🔄 Agent {:?} ready for background optimization", agent_type);
                        }
                    }

                    *last_global_optimization.write().await = Instant::now();
                }
            }
        });
    }

    /// 📊 Get optimization statistics
    pub async fn get_optimization_stats(&self) -> OptimizationStats {
        let states = self.optimization_states.read().await;
        let history = self.optimization_history.read().await;

        let total_optimizations = history.len();
        let avg_improvement = if !history.is_empty() {
            history.iter().map(|r| r.expected_improvement).sum::<f64>() / history.len() as f64
        } else {
            0.0
        };

        let agents_optimized = states.len();
        let total_performance_samples = states.values()
            .map(|s| s.performance_history.len())
            .sum::<usize>();

        OptimizationStats {
            total_optimizations,
            avg_improvement,
            agents_optimized,
            total_performance_samples,
            last_optimization: history.last().map(|r| r.timestamp),
        }
    }
}

// Helper structs
#[derive(Debug)]
struct InternalOptimizationResult {
    expected_improvement: f64,
    confidence: f64,
    method: String,
}

#[derive(Debug)]
struct ContextPerformanceAnalysis {
    correlation: f64,
    confidence: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptimizationStats {
    pub total_optimizations: usize,
    pub avg_improvement: f64,
    pub agents_optimized: usize,
    pub total_performance_samples: usize,
    pub last_optimization: Option<DateTime<Utc>>,
}
