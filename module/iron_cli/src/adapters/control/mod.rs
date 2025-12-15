//! Control API adapters
//!
//! Bridge CLI commands to Control API HTTP endpoints.
//!
//! ## Architecture
//!
//! ```text
//! CLI Command → Adapter → Handler (validation) → HTTP Client → REST API
//!       ↓          ↓            ↓                     ↓            ↓
//!   Parameters  Extract    Validate params      POST/GET    API Response
//! ```
//!
//! ## Adapter Responsibilities
//!
//! 1. Extract parameters from command
//! 2. Call handler for validation (pure, sync)
//! 3. Check dry-run mode
//! 4. Make HTTP request to Control API
//! 5. Format response
//!
//! ## HTTP Client
//!
//! Uses reqwest for async HTTP calls to Control API endpoints.
//! Base URL configured via environment variable: IRON_CONTROL_API_URL
//! Default: http://localhost:8080
//!
//! ## Authentication
//!
//! API token passed via Authorization header: Bearer <token>
//! Token loaded from: `iron_config` layers or `IRON_CONTROL_API_TOKEN` env var
//!
//! ## Error Handling
//!
//! - Validation errors: Returned from handlers before HTTP call
//! - Network errors: Wrapped in AdapterError::NetworkError
//! - API errors: Parsed from response, wrapped in AdapterError::ApiError
//!
//! ## Dry Run Mode
//!
//! When dry::1 is set:
//! - Handler validation runs normally
//! - HTTP request is NOT made
//! - Simulated response returned to user
//!
//! ## Phase 2 Implementation Status
//!
//! Current: Basic infrastructure only (HTTP client, config)
//! Phase 3: Will implement all 46 adapter functions
//! Phase 4: Will add comprehensive error handling and retries

pub mod http_client;
pub mod config;
pub mod formatter;
pub mod agent_adapters;
pub mod provider_adapters;
pub mod analytics_adapters;
pub mod budget_limit_adapters;
pub mod api_token_adapters;
pub mod project_adapters;
pub mod budget_request_adapters;
pub mod user_adapters;

pub use http_client::ControlApiClient;
pub use config::ControlApiConfig;
pub use formatter::format_output;
