//! Budget Control Protocol REST API endpoints
//!
//! Protocol 005: Budget Control Protocol
//! Protocol 012: Budget Request Workflow API
//!
//! Endpoints:
//! - POST /api/budget/handshake - IC Token → IP Token exchange with budget lease
//! - POST /api/budget/report - Report LLM usage cost to Control Panel
//! - POST /api/budget/refresh - Request additional budget when running low
//! - POST /api/budget/return - Return unused budget when runtime shuts down
//! - POST /api/v1/budget/requests - Create budget change request (Protocol 012)
//! - GET /api/v1/budget/requests/:id - Get budget request details
//! - GET /api/v1/budget/requests - List budget requests with filtering
//! - PATCH /api/v1/budget/requests/:id/approve - Approve budget request
//! - PATCH /api/v1/budget/requests/:id/reject - Reject budget request

// Module declarations
pub mod state;
pub mod handshake;
pub mod usage;
pub mod refresh;
pub mod request_workflow;

// Re-export shared state
pub use state::BudgetState;

// Re-export handshake types and endpoint
pub use handshake::
{
  HandshakeRequest,
  HandshakeResponse,
  handshake,
};

// Re-export usage types and endpoints
pub use usage::
{
  UsageReportRequest,
  UsageReportResponse,
  report_usage,
  BudgetReturnRequest,
  BudgetReturnResponse,
  return_budget,
};

// Re-export refresh types and endpoint
pub use refresh::
{
  BudgetRefreshRequest,
  BudgetRefreshResponse,
  refresh_budget,
};

// Re-export request workflow types and endpoints
pub use request_workflow::
{
  CreateBudgetRequestRequest,
  CreateBudgetRequestResponse,
  create_budget_request,
  GetBudgetRequestResponse,
  get_budget_request,
  ListBudgetRequestsQuery,
  ListBudgetRequestsResponse,
  list_budget_requests,
  ApproveBudgetRequestResponse,
  approve_budget_request,
  RejectBudgetRequestResponse,
  reject_budget_request,
};
