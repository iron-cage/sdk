//! Control API command handlers
//!
//! Pure business logic handlers for the `iron` control API CLI.
//! These handlers support all 46 control API commands across 8 resource categories.
//!
//! ## Architecture
//!
//! Handlers are pure functions: `HashMap<String, String> → Result<String, CliError>`
//! - No I/O operations (no async, no .await)
//! - No external service calls
//! - Only validation and formatting
//!
//! ## Resource Categories (46 handlers total)
//!
//! - **Agents** (8): list, create, get, update, delete, assign_providers, list_providers, remove_provider
//! - **Providers** (8): list, create, get, update, delete, assign_agents, list_agents, remove_agent
//! - **Analytics** (8): usage, spending, metrics, usage_by_agent, usage_by_provider, spending_by_period, export_usage, export_spending
//! - **Budget Limits** (2): get, set (admin only)
//! - **API Tokens** (4): list, create, get, revoke
//! - **Projects** (2): list, get (read-only in Pilot)
//! - **Budget Requests** (6): list, create, get, approve, reject, cancel
//! - **Users** (8): list, create, get, update, delete, set_role, reset_password, get_permissions
//!
//! ## Handler Signature
//!
//! ```rust,ignore
//! pub fn handler_name(
//!     params: &HashMap<String, String>,
//! ) -> Result<String, CliError>
//! {
//!     // 1. Validate required parameters (MissingParameter)
//!     // 2. Validate formats/patterns (InvalidParameter)
//!     // 3. Format output string
//!     // 4. Return result
//! }
//! ```
//!
//! ## Common Patterns
//!
//! - **Dry run validation**: All mutation commands support `dry::1` parameter
//! - **Format validation**: All commands support `format::table|json|yaml`
//! - **Verbosity**: List commands support `v::0-5` for verbosity control
//! - **Pagination**: List commands support `page` and `limit` parameters
//!
//! ## Notes
//!
//! - User handlers named `control_user_handlers` to distinguish from token CLI's `user_handlers`
//! - All handlers return placeholder strings; actual data loading happens in adapter layer

pub mod agent_handlers;
pub mod provider_handlers;
pub mod analytics_handlers;
pub mod budget_limit_handlers;
pub mod api_token_handlers;
pub mod project_handlers;
pub mod budget_request_handlers;
pub mod control_user_handlers;
