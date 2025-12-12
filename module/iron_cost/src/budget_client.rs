//! Budget Client for Protocol 005: Budget Control Protocol
//!
//! Handles communication with the Iron Cage dashboard for budget management:
//! - Handshake: Get lease and API key from dashboard
//! - Report: Report usage to dashboard (async, non-blocking)
//! - Refresh: Request more budget when running low
//! - Return: Return unused budget on shutdown

use crate::budget::{CostController, Reservation};
use crate::converter::usd_to_micros;
use crate::error::CostError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Errors that can occur in budget client operations
#[derive(Debug)]
pub enum BudgetClientError {
    /// HTTP request failed
    HttpError(reqwest::Error),
    /// Server returned error
    ServerError { status: u16, message: String },
    /// No active lease - call handshake() first
    NoLease,
    /// Handshake failed
    HandshakeFailed(String),
    /// Budget operation failed
    BudgetError(CostError),
    /// JSON parse error
    JsonError(serde_json::Error),
}

impl std::fmt::Display for BudgetClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HttpError(e) => write!(f, "HTTP request failed: {}", e),
            Self::ServerError { status, message } => {
                write!(f, "Server returned error: {} - {}", status, message)
            }
            Self::NoLease => write!(f, "No active lease - call handshake() first"),
            Self::HandshakeFailed(msg) => write!(f, "Handshake failed: {}", msg),
            Self::BudgetError(e) => write!(f, "Budget operation failed: {}", e),
            Self::JsonError(e) => write!(f, "JSON parse error: {}", e),
        }
    }
}

impl std::error::Error for BudgetClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::HttpError(e) => Some(e),
            Self::BudgetError(e) => Some(e),
            Self::JsonError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for BudgetClientError {
    fn from(e: reqwest::Error) -> Self {
        Self::HttpError(e)
    }
}

impl From<CostError> for BudgetClientError {
    fn from(e: CostError) -> Self {
        Self::BudgetError(e)
    }
}

impl From<serde_json::Error> for BudgetClientError {
    fn from(e: serde_json::Error) -> Self {
        Self::JsonError(e)
    }
}

/// Provider API key information
#[derive(Debug, Clone)]
pub struct ProviderKey {
    /// Provider type: "openai" or "anthropic"
    pub provider: String,
    /// The actual API key (decrypted)
    pub api_key: String,
    /// Optional custom base URL for the provider
    pub base_url: Option<String>,
}

/// Lease state from dashboard
#[derive(Debug, Clone)]
struct LeaseState {
    /// Lease ID from dashboard
    lease_id: String,
    /// Budget granted for this lease (USD)
    #[allow(dead_code)]
    budget_granted: f64,
}

/// Handshake request body
#[derive(Debug, Serialize)]
struct HandshakeRequest {
    ic_token: String,
    provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider_key_id: Option<i64>,
}

/// Handshake response from dashboard
#[derive(Debug, Deserialize)]
struct HandshakeResponse {
    ip_token: String,
    lease_id: String,
    budget_granted: f64,
    #[allow(dead_code)]
    budget_remaining: f64,
    #[allow(dead_code)]
    expires_at: Option<i64>,
}

/// Usage report request body
#[derive(Debug, Serialize)]
struct ReportRequest {
    lease_id: String,
    request_id: String,
    tokens: i64,
    cost_usd: f64,
    model: String,
    provider: String,
}

/// Usage report response from dashboard
#[derive(Debug, Deserialize)]
pub struct ReportResponse {
    pub success: bool,
    pub budget_remaining: f64,
}

/// Budget refresh request body
#[derive(Debug, Serialize)]
struct RefreshRequest {
    ic_token: String,
    current_lease_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    requested_budget: Option<f64>,
}

/// Budget refresh response from dashboard
#[derive(Debug, Deserialize)]
pub struct RefreshResponse {
    pub status: String,
    pub budget_granted: Option<f64>,
    pub budget_remaining: f64,
    pub lease_id: Option<String>,
    pub reason: Option<String>,
}

/// Budget return request body
#[derive(Debug, Serialize)]
struct ReturnRequest {
    lease_id: String,
}

/// Budget return response from dashboard
#[derive(Debug, Deserialize)]
pub struct ReturnResponse {
    pub success: bool,
    pub returned: f64,
}

/// Budget client configuration
pub struct BudgetClientConfig {
    /// Iron Cage server URL (e.g., "http://localhost:3000")
    pub server_url: String,
    /// IC Token for authentication
    pub ic_token: String,
    /// Provider to request (openai, anthropic)
    pub provider: String,
    /// Optional specific provider key ID
    pub provider_key_id: Option<i64>,
    /// Initial budget to request (USD)
    pub initial_budget: f64,
    /// HTTP request timeout
    pub timeout: Duration,
}

impl Default for BudgetClientConfig {
    fn default() -> Self {
        Self {
            server_url: String::new(),
            ic_token: String::new(),
            provider: "openai".to_string(),
            provider_key_id: None,
            initial_budget: 10.0,
            timeout: Duration::from_secs(30),
        }
    }
}

/// Client for Protocol 005 budget operations.
///
/// Manages budget lease lifecycle and cost tracking:
/// 1. Handshake: Get lease + API key from dashboard
/// 2. Reserve/Commit: Track costs locally with atomic reservation
/// 3. Report: Async report to dashboard (non-blocking)
/// 4. Refresh: Get more budget when low
/// 5. Return: Return unused on shutdown
pub struct BudgetClient {
    /// Configuration
    config: BudgetClientConfig,
    /// HTTP client
    http_client: Client,
    /// Local cost controller with reservation support
    cost_controller: CostController,
    /// Lease state (set after handshake)
    lease: RwLock<Option<LeaseState>>,
    /// Provider API key (set after handshake)
    provider_key: RwLock<Option<ProviderKey>>,
}

impl BudgetClient {
    /// Create a new budget client (does not perform handshake)
    pub fn new(config: BudgetClientConfig) -> Result<Self, BudgetClientError> {
        let http_client = Client::builder()
            .timeout(config.timeout)
            .build()?;

        // Start with 0 budget - will be set after handshake
        let cost_controller = CostController::new(0.0);

        Ok(Self {
            config,
            http_client,
            cost_controller,
            lease: RwLock::new(None),
            provider_key: RwLock::new(None),
        })
    }

    /// Perform handshake with dashboard to get lease and API key.
    ///
    /// Must be called before using the client for requests.
    pub async fn handshake(&self) -> Result<(), BudgetClientError> {
        let url = format!("{}/api/budget/handshake", self.config.server_url);

        let request = HandshakeRequest {
            ic_token: self.config.ic_token.clone(),
            provider: self.config.provider.clone(),
            provider_key_id: self.config.provider_key_id,
        };

        let response = self
            .http_client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            return Err(BudgetClientError::ServerError { status, message });
        }

        let data: HandshakeResponse = response.json().await?;

        // Decrypt IP token to get provider API key
        // For now, treat ip_token as the raw key (dashboard handles encryption)
        let provider_key = ProviderKey {
            provider: self.config.provider.clone(),
            api_key: data.ip_token,
            base_url: None,
        };

        // Set budget limit from lease
        self.cost_controller.set_budget(data.budget_granted);

        // Store lease state
        {
            let mut lease_guard = self.lease.write().await;
            *lease_guard = Some(LeaseState {
                lease_id: data.lease_id,
                budget_granted: data.budget_granted,
            });
        }

        // Store provider key
        {
            let mut key_guard = self.provider_key.write().await;
            *key_guard = Some(provider_key);
        }

        Ok(())
    }

    /// Report usage to dashboard (async, fire-and-forget style).
    ///
    /// This is meant to be called after each LLM request completes.
    /// The report is non-blocking to avoid adding latency.
    pub async fn report(
        &self,
        cost_usd: f64,
        tokens: i64,
        model: &str,
        request_id: &str,
    ) -> Result<ReportResponse, BudgetClientError> {
        let lease = self.lease.read().await;
        let lease = lease.as_ref().ok_or(BudgetClientError::NoLease)?;

        let url = format!("{}/api/budget/report", self.config.server_url);

        let request = ReportRequest {
            lease_id: lease.lease_id.clone(),
            request_id: request_id.to_string(),
            tokens,
            cost_usd,
            model: model.to_string(),
            provider: self.config.provider.clone(),
        };

        let response = self
            .http_client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            return Err(BudgetClientError::ServerError { status, message });
        }

        let data: ReportResponse = response.json().await?;
        Ok(data)
    }

    /// Request more budget when running low.
    ///
    /// If approved, updates the local cost controller with new budget.
    pub async fn refresh(&self, requested_budget: Option<f64>) -> Result<RefreshResponse, BudgetClientError> {
        let lease = self.lease.read().await;
        let lease = lease.as_ref().ok_or(BudgetClientError::NoLease)?;

        let url = format!("{}/api/budget/refresh", self.config.server_url);

        let request = RefreshRequest {
            ic_token: self.config.ic_token.clone(),
            current_lease_id: lease.lease_id.clone(),
            requested_budget,
        };

        let response = self
            .http_client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            return Err(BudgetClientError::ServerError { status, message });
        }

        let data: RefreshResponse = response.json().await?;

        // If approved, update lease and budget
        if data.status == "approved" {
            if let (Some(new_lease_id), Some(new_budget)) = (&data.lease_id, data.budget_granted) {
                // Update lease state
                {
                    let mut lease_guard = self.lease.write().await;
                    *lease_guard = Some(LeaseState {
                        lease_id: new_lease_id.clone(),
                        budget_granted: new_budget,
                    });
                }

                // Reset cost controller with new budget
                // Note: This resets spent to 0 for the new lease
                self.cost_controller.set_budget(new_budget);
            }
        }

        Ok(data)
    }

    /// Return unused budget to dashboard on shutdown.
    ///
    /// Should be called when the LLM router is stopping.
    pub async fn return_unused(&self) -> Result<ReturnResponse, BudgetClientError> {
        let lease = self.lease.read().await;
        let lease = lease.as_ref().ok_or(BudgetClientError::NoLease)?;

        let url = format!("{}/api/budget/return", self.config.server_url);

        let request = ReturnRequest {
            lease_id: lease.lease_id.clone(),
        };

        let response = self
            .http_client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            return Err(BudgetClientError::ServerError { status, message });
        }

        let data: ReturnResponse = response.json().await?;
        Ok(data)
    }

    // =========================================================================
    // Delegation methods to CostController
    // =========================================================================

    /// Reserve budget atomically before an LLM call.
    ///
    /// This must be called before making an LLM request to prevent concurrent overspend.
    pub fn reserve(&self, max_cost_micros: u64) -> Result<Reservation, CostError> {
        self.cost_controller.reserve(max_cost_micros)
    }

    /// Reserve budget in USD (convenience wrapper).
    pub fn reserve_usd(&self, max_cost_usd: f64) -> Result<Reservation, CostError> {
        self.cost_controller.reserve(usd_to_micros(max_cost_usd))
    }

    /// Commit a reservation with actual cost.
    ///
    /// Call after an LLM request completes successfully.
    pub fn commit(&self, reservation: Reservation, actual_cost_micros: u64) {
        self.cost_controller.commit(reservation, actual_cost_micros);
    }

    /// Commit a reservation with actual cost in USD.
    pub fn commit_usd(&self, reservation: Reservation, actual_cost_usd: f64) {
        self.cost_controller.commit_usd(reservation, actual_cost_usd);
    }

    /// Cancel a reservation without adding cost.
    ///
    /// Call if an LLM request fails or is cancelled.
    pub fn cancel(&self, reservation: Reservation) {
        self.cost_controller.cancel(reservation);
    }

    /// Check if budget is available (includes reserved amounts).
    pub fn check_budget(&self) -> Result<(), CostError> {
        self.cost_controller.check_budget()
    }

    /// Add spend directly (without reservation).
    ///
    /// Use this for cases where reservation is not needed.
    pub fn add_spend(&self, cost_usd: f64) {
        self.cost_controller.add_spend(cost_usd);
    }

    /// Add spend in microdollars (without reservation).
    pub fn add_spend_micros(&self, cost_micros: u64) {
        self.cost_controller.add_spend_micros(cost_micros);
    }

    // =========================================================================
    // Getters
    // =========================================================================

    /// Get the provider API key (after handshake).
    pub async fn get_provider_key(&self) -> Option<ProviderKey> {
        self.provider_key.read().await.clone()
    }

    /// Get the current lease ID (after handshake).
    pub async fn get_lease_id(&self) -> Option<String> {
        self.lease.read().await.as_ref().map(|l| l.lease_id.clone())
    }

    /// Get budget status: (spent_usd, limit_usd).
    pub fn get_status(&self) -> (f64, f64) {
        self.cost_controller.get_status()
    }

    /// Get full budget status: (spent_usd, reserved_usd, limit_usd).
    pub fn get_full_status(&self) -> (f64, f64, f64) {
        self.cost_controller.get_full_status()
    }

    /// Get available budget in USD.
    pub fn available(&self) -> f64 {
        self.cost_controller.available()
    }

    /// Get total spent in USD.
    pub fn total_spent(&self) -> f64 {
        self.cost_controller.total_spent()
    }

    /// Get budget limit in USD.
    pub fn budget_limit(&self) -> f64 {
        self.cost_controller.budget_limit()
    }

    /// Check if handshake has been completed.
    pub async fn is_connected(&self) -> bool {
        self.lease.read().await.is_some()
    }

    /// Get reference to cost controller (for advanced usage).
    pub fn cost_controller(&self) -> &CostController {
        &self.cost_controller
    }
}

/// Builder for BudgetClient configuration
pub struct BudgetClientBuilder {
    config: BudgetClientConfig,
}

impl BudgetClientBuilder {
    pub fn new() -> Self {
        Self {
            config: BudgetClientConfig::default(),
        }
    }

    pub fn server_url(mut self, url: impl Into<String>) -> Self {
        self.config.server_url = url.into();
        self
    }

    pub fn ic_token(mut self, token: impl Into<String>) -> Self {
        self.config.ic_token = token.into();
        self
    }

    pub fn provider(mut self, provider: impl Into<String>) -> Self {
        self.config.provider = provider.into();
        self
    }

    pub fn provider_key_id(mut self, id: i64) -> Self {
        self.config.provider_key_id = Some(id);
        self
    }

    pub fn initial_budget(mut self, budget: f64) -> Self {
        self.config.initial_budget = budget;
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    pub fn build(self) -> Result<BudgetClient, BudgetClientError> {
        BudgetClient::new(self.config)
    }
}

impl Default for BudgetClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a static budget client that doesn't connect to dashboard.
///
/// Useful for testing or standalone mode.
pub fn create_static_client(budget_usd: f64) -> Arc<CostController> {
    Arc::new(CostController::new(budget_usd))
}
