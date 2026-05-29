//! Workspace management API endpoints
//!
//! Endpoints:
//! - POST `/api/v1/workspace/budget` - upsert workspace spending policy (requires `ManageIcTokens`: Admin or Manager)
//! - GET  `/api/v1/me/workspace` - read the current user's workspace info

use axum::{
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::{
  error::{ApiError, ApiResult, JsonBody},
  freeform::usage_policy::{self, CapPeriod},
  jwt_auth::AuthenticatedUser,
  rbac::Permission,
};

fn period_to_str(period: &CapPeriod) -> &'static str {
  match period {
    CapPeriod::Day => "day",
    CapPeriod::Week => "week",
    CapPeriod::Month => "month",
  }
}

/// Joins a model list into the comma-separated form stored in `workspace_policy`.
/// Returns `None` for an empty list so the upsert preserves the existing value.
fn join_models(models: &[String]) -> Option<String> {
  if models.is_empty() {
    None
  } else {
    Some(models.join(","))
  }
}

/// Splits the stored comma-separated `requestable_models` column into a list.
fn split_models(stored: Option<&str>) -> Vec<String> {
  stored
    .map(|s| {
      s.split(',')
        .map(str::trim)
        .filter(|m| !m.is_empty())
        .map(str::to_string)
        .collect()
    })
    .unwrap_or_default()
}

/// Request body for `POST /api/v1/workspace/budget`.
///
/// Accepts either a raw freeform string or structured fields.
/// If `text` is provided, it is parsed via the usage-policy parser.
/// Otherwise `amount_cents` and `period` must both be present.
#[derive(Debug, Deserialize)]
pub struct BudgetRequest {
  /// Raw freeform string e.g. `"limit all users $100/week"` or `"$100/week"`.
  pub text: Option<String>,
  /// Budget ceiling in US cents (e.g. `$100` -> `10000`). Used when `text` is absent.
  pub amount_cents: Option<i64>,
  /// Billing period: `"day"`, `"week"`, or `"month"`. Used when `text` is absent.
  pub period: Option<String>,
  /// Optional default model for the workspace. When omitted, the existing
  /// value (if any) is preserved.
  pub default_model: Option<String>,
  /// Optional gated models members may request (structured input). When omitted,
  /// the existing value is preserved; `text` input uses the `requestable:` line.
  pub requestable_models: Option<Vec<String>>,
}

/// Response from `POST /api/v1/workspace/budget`.
#[derive(Debug, Serialize)]
pub struct WorkspaceBudgetResponse {
  /// Workspace the policy applies to (always `1` for single-workspace deployments).
  pub workspace_id: i64,
  /// Effective budget ceiling in US cents.
  pub amount_cents: i64,
  /// Effective billing period.
  pub period: String,
  /// Effective default model after the upsert (may be `None` if never set).
  pub default_model: Option<String>,
  /// Effective gated models members may request after the upsert.
  pub requestable_models: Vec<String>,
}

/// POST /api/v1/workspace/budget
///
/// Upsert the workspace spending policy. Accepts either a raw freeform string
/// or structured `amount_cents` + `period` fields.
///
/// # Errors
///
/// Returns `ApiError` on permission failure, parse error, missing fields,
/// or database error.
pub async fn post_workspace_budget(
  State(pool): State<SqlitePool>,
  AuthenticatedUser(claims): AuthenticatedUser,
  JsonBody(body): JsonBody<BudgetRequest>,
) -> ApiResult<impl IntoResponse> {
  super::check_permission(&claims.role, Permission::ManageIcTokens)?;

  let (amount_cents, period_str, default_model, requestable_str) = if let Some(ref raw) = body.text
  {
    let policy = usage_policy::parse(raw).map_err(|errors| {
      let msg = errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ");
      ApiError::BadRequest(msg)
    })?;

    let cap = policy.spending_cap.ok_or_else(|| {
      ApiError::BadRequest(
        "No spending cap found in text. Expected: 'limit all users $N/<period>'".into(),
      )
    })?;

    let cents = cap
      .amount_cents
      .try_into()
      .map_err(|_| ApiError::BadRequest("Budget amount too large".into()))?;

    (
      cents,
      period_to_str(&cap.period).to_string(),
      policy.default_model,
      join_models(&policy.requestable_models),
    )
  } else {
    let amount = body.amount_cents.ok_or_else(|| {
      ApiError::BadRequest("Either 'text' or 'amount_cents' + 'period' required".into())
    })?;
    let period = body.period.ok_or_else(|| {
      ApiError::BadRequest("Either 'text' or 'amount_cents' + 'period' required".into())
    })?;

    let valid_period = match period.to_lowercase().as_str() {
      "day" | "week" | "month" => period.to_lowercase(),
      _ => {
        return Err(ApiError::BadRequest(format!(
          "Invalid period '{period}' - supported: day, week, month"
        )))
      }
    };

    if amount < 0 {
      return Err(ApiError::BadRequest(
        "amount_cents must be non-negative".into(),
      ));
    }

    (
      amount,
      valid_period,
      body.default_model.clone(),
      body.requestable_models.as_deref().and_then(join_models),
    )
  };

  // COALESCE preserves existing default_model / requestable_models when this
  // request does not specify them (e.g. admin edits only the budget amount).
  let row = sqlx::query_as::<_, (i64, Option<String>, Option<String>)>(
    r"
      INSERT INTO workspace_policy
        (workspace_id, budget_amount_cents, budget_period, default_model, requestable_models, updated_at)
      VALUES (1, ?, ?, ?, ?, strftime('%s', 'now') * 1000)
      ON CONFLICT(workspace_id) DO UPDATE SET
        budget_amount_cents = excluded.budget_amount_cents,
        budget_period       = excluded.budget_period,
        default_model       = COALESCE(excluded.default_model, workspace_policy.default_model),
        requestable_models  = COALESCE(excluded.requestable_models, workspace_policy.requestable_models),
        updated_at          = excluded.updated_at
      RETURNING workspace_id, default_model, requestable_models
    ",
  )
  .bind(amount_cents)
  .bind(&period_str)
  .bind(&default_model)
  .bind(&requestable_str)
  .fetch_one(&pool)
  .await
  .map_err(|_| ApiError::Internal("Failed to upsert workspace policy".into()))?;

  Ok((
    StatusCode::OK,
    Json(WorkspaceBudgetResponse {
      workspace_id: row.0,
      amount_cents,
      period: period_str,
      default_model: row.1,
      requestable_models: split_models(row.2.as_deref()),
    }),
  ))
}

/// Response from `GET /api/v1/me/workspace`.
#[derive(Debug, Serialize)]
pub struct MeWorkspaceResponse {
  /// Workspace the authenticated user belongs to.
  pub workspace_id: i64,
  /// Display name of the workspace.
  pub workspace_name: String,
  /// Company domain (e.g. `acme.com`).
  pub domain: String,
  /// Default model used by the workspace, if configured.
  pub default_model: Option<String>,
  /// Workspace budget ceiling in US cents, if configured.
  pub budget_amount_cents: Option<i64>,
  /// Workspace billing period (`day`, `week`, `month`), if configured.
  pub budget_period: Option<String>,
  /// Gated models members may request access to, if configured.
  pub requestable_models: Vec<String>,
}

/// DB row returned by the `/me/workspace` query.
#[derive(sqlx::FromRow)]
struct MeWorkspaceRow {
  id: i64,
  name: String,
  domain: String,
  default_model: Option<String>,
  budget_amount_cents: Option<i64>,
  budget_period: Option<String>,
  requestable_models: Option<String>,
}

/// `GET /api/v1/me/workspace`
///
/// Return the current user's workspace info: name, domain, and policy summary.
/// Any authenticated user (Admin or Member) may call this.
///
/// # Errors
///
/// Returns `ApiError` if no workspace exists or the database query fails.
pub async fn get_me_workspace(
  State(pool): State<SqlitePool>,
  AuthenticatedUser(_claims): AuthenticatedUser,
) -> ApiResult<impl IntoResponse> {
  let row = sqlx::query_as::<_, MeWorkspaceRow>(
    r"
      SELECT w.id, w.name, w.domain,
             wp.default_model,
             wp.budget_amount_cents,
             wp.budget_period,
             wp.requestable_models
      FROM workspaces w
      LEFT JOIN workspace_policy wp ON wp.workspace_id = w.id
      WHERE w.id = 1
    ",
  )
  .fetch_optional(&pool)
  .await
  .map_err(|_| ApiError::Internal("Failed to query workspace".into()))?;

  let row = row.ok_or_else(|| ApiError::NotFound("Workspace not configured".into()))?;

  Ok((
    StatusCode::OK,
    Json(MeWorkspaceResponse {
      workspace_id: row.id,
      workspace_name: row.name,
      domain: row.domain,
      default_model: row.default_model,
      budget_amount_cents: row.budget_amount_cents,
      budget_period: row.budget_period,
      requestable_models: split_models(row.requestable_models.as_deref()),
    }),
  ))
}
