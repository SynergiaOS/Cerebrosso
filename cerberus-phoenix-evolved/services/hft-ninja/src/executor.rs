//! ⚡ Transaction execution engine

use crate::config::Config;
use anyhow::{anyhow, Result};
use bincode;
use cerberus_core_types::{Decision, DecisionAction, ExecutionResult};
use reqwest::Client;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    system_instruction,
    transaction::Transaction,
};
use std::{str::FromStr, sync::Arc, time::Instant};
use tracing::{error, info, warn};

pub struct TransactionExecutor {
    config: Arc<Config>,
    http_client: Client,
    keypair: Option<Keypair>, // TODO: Load from Vault
}

impl TransactionExecutor {
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_millis(config.solana.timeout_ms))
            .build()?;

        // TODO: Load keypair from Vault
        let keypair = None;

        Ok(Self {
            config,
            http_client,
            keypair,
        })
    }

    pub async fn execute_decision(&self, decision: &Decision) -> Result<ExecutionResult> {
        let start = Instant::now();

        match &decision.action {
            DecisionAction::Snipe { amount_sol, slippage } => {
                self.execute_snipe(decision, *amount_sol, *slippage).await
            }
            DecisionAction::Dump { percentage } => {
                self.execute_dump(decision, *percentage).await
            }
            DecisionAction::Hold => {
                info!("🤚 Decision is HOLD, no execution needed");
                Ok(ExecutionResult::success(
                    decision.id,
                    start.elapsed().as_millis() as u64,
                    0.0,
                    "hold".to_string(),
                ))
            }
            DecisionAction::Ignore => {
                info!("🚫 Decision is IGNORE, skipping execution");
                Ok(ExecutionResult::success(
                    decision.id,
                    start.elapsed().as_millis() as u64,
                    0.0,
                    "ignored".to_string(),
                ))
            }
        }
    }

    async fn execute_snipe(
        &self,
        decision: &Decision,
        amount_sol: f64,
        slippage: f32,
    ) -> Result<ExecutionResult> {
        info!("🎯 Executing SNIPE: {} SOL with {}% slippage", amount_sol, slippage * 100.0);

        // For MVP: simulate transaction
        let simulated_tx_hash = format!("sim_{}", uuid::Uuid::new_v4());
        let simulated_profit = amount_sol * 0.05; // 5% profit simulation

        tokio::time::sleep(std::time::Duration::from_millis(50)).await; // Simulate execution time

        Ok(ExecutionResult::success(
            decision.id,
            50, // Simulated latency
            simulated_profit,
            simulated_tx_hash,
        ))
    }

    async fn execute_dump(
        &self,
        decision: &Decision,
        percentage: f32,
    ) -> Result<ExecutionResult> {
        info!("💰 Executing DUMP: {}% of position", percentage * 100.0);

        // For MVP: simulate transaction
        let simulated_tx_hash = format!("dump_{}", uuid::Uuid::new_v4());
        let simulated_profit = 0.1; // Simulated profit

        tokio::time::sleep(std::time::Duration::from_millis(30)).await;

        Ok(ExecutionResult::success(
            decision.id,
            30,
            simulated_profit,
            simulated_tx_hash,
        ))
    }

    async fn get_latest_blockhash(&self) -> Result<String> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getLatestBlockhash",
            "params": [
                {
                    "commitment": self.config.solana.commitment
                }
            ]
        });

        let response: serde_json::Value = self
            .http_client
            .post(&self.config.solana.rpc_url)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        if let Some(error) = response.get("error") {
            return Err(anyhow!("RPC error: {}", error));
        }

        let blockhash = response["result"]["value"]["blockhash"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid blockhash response"))?;

        Ok(blockhash.to_string())
    }

    async fn send_transaction(&self, transaction: &Transaction) -> Result<String> {
        let serialized = bs58::encode(bincode::serialize(transaction)?).into_string();

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendTransaction",
            "params": [
                serialized,
                {
                    "skipPreflight": true,
                    "preflightCommitment": self.config.solana.commitment,
                    "encoding": "base58"
                }
            ]
        });

        let response: serde_json::Value = self
            .http_client
            .post(&self.config.solana.rpc_url)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        if let Some(error) = response.get("error") {
            return Err(anyhow!("Transaction failed: {}", error));
        }

        let signature = response["result"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid transaction response"))?;

        Ok(signature.to_string())
    }
}
