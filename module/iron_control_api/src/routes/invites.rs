//! Invite link API endpoints
//!
//! Endpoints:
//! - POST `/api/v1/invites/generate`           — create invite link (Admin)
//! - GET  `/api/v1/invites/pending`            — list pending invite seats (Admin)
//! - GET  `/api/v1/invites/{token}`            — preview workspace via invite (Public)
//! - POST `/api/v1/invites/{token}/accept`     — accept invite, create account (Public)
//! - POST `/api/v1/invites/{token}/approve`    — approve pending member by token (Admin)
//! - POST `/api/v1/invites/seats/{id}/approve` — approve pending member by seat id (Admin)

use core::str::FromStr;
use std::sync::Arc;

use axum::{
  extract::{Path, State},
  http::StatusCode,
  response::{IntoResponse, Json},
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{pool::PoolConnection, Sqlite, SqlitePool};

use uuid::Uuid;

use crate::{
  error::{ApiError, ApiResult, JsonBody},
  jwt_auth::{AuthenticatedUser, JwtSecret},
  rbac::{Permission, PermissionChecker, Role},
};

/// `BCrypt` cost for new member passwords (matches `UserService::BCRYPT_COST`).
const BCRYPT_COST: u32 = 12;

/// State for invite endpoints.
#[derive(Clone)]
pub struct InviteState {
  /// Database pool.
  pub pool: SqlitePool,
  /// JWT signing secret for generating access tokens on invite accept.
  pub jwt_secret: Arc<JwtSecret>,
}

impl core::fmt::Debug for InviteState {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("InviteState")
      .field("pool", &"<SqlitePool>")
      .field("jwt_secret", &"<JwtSecret>")
      .finish()
  }
}

fn check_permission(role_str: &str, permission: Permission) -> Result<(), ApiError> {
  let role = Role::from_str(role_str)
    .map_err(|_| ApiError::Forbidden(format!("Invalid role: {role_str}")))?;
  let checker = PermissionChecker::new();
  if checker.has_permission(role, permission) {
    Ok(())
  } else {
    Err(ApiError::Forbidden(format!(
      "Permission {permission:?} required"
    )))
  }
}

/// Generate 32 random bytes encoded as URL-safe base64 (no padding).
fn generate_invite_token() -> String {
  let mut rng = rand::rng();
  let mut bytes = [0u8; 32];
  rng.fill(&mut bytes[..]);
  URL_SAFE_NO_PAD.encode(bytes)
}

/// SHA-256 of a token string, hex-encoded.
fn hash_token(token: &str) -> String {
  hex::encode(Sha256::digest(token.as_bytes()))
}

/// Current time in milliseconds since Unix epoch.
fn now_ms() -> i64 {
  std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map(|d| {
      let ms = d.as_millis();
      i64::try_from(ms).unwrap_or(i64::MAX)
    })
    .unwrap_or(0)
}

// Request / Response types ----------------------------------------------------

/// Request body for `POST /api/v1/invites/generate`.
#[derive(Debug, Deserialize)]
pub struct GenerateInviteRequest {
  /// Number of people who can use this link.
  pub seats: i64,
  /// Optional lifetime in hours (omit for no expiry).
  pub expires_in_hours: Option<i64>,
}

/// Response from `POST /api/v1/invites/generate`.
#[derive(Debug, Serialize)]
pub struct GenerateInviteResponse {
  /// Raw invite token to embed in the invite URL.
  pub token: String,
  /// Suggested join URL path (e.g. `/join/{token}`).
  pub url: String,
  /// Unix-ms expiry timestamp, or `null` if the link never expires.
  pub expires_at: Option<i64>,
  /// Total seats on this link.
  pub seats_total: i64,
}

/// Response from `GET /api/v1/invites/{token}`.
#[derive(Debug, Serialize)]
pub struct InvitePreviewResponse {
  /// Workspace display name.
  pub workspace_name: String,
  /// Primary domain.
  pub domain: String,
  /// Default AI model for the workspace, if configured.
  pub default_model: Option<String>,
  /// Remaining unused seats on this link.
  pub seats_remaining: i64,
}

/// Request body for `POST /api/v1/invites/{token}/accept`.
#[derive(Debug, Deserialize)]
pub struct AcceptInviteRequest {
  /// Password for the new member account.
  pub password: String,
}

/// Response from `POST /api/v1/invites/{token}/accept`.
#[derive(Debug, Serialize)]
pub struct AcceptInviteResponse {
  /// Newly created user ID.
  pub user_id: String,
  /// JWT access token (30-day lifetime).
  pub access_token: String,
}

/// Request body for `POST /api/v1/invites/{token}/approve`.
#[derive(Debug, Deserialize)]
pub struct ApproveInviteRequest {
  /// ID of the pending member to approve.
  pub user_id: String,
}

// Handlers --------------------------------------------------------------------

/// `POST /api/v1/invites/generate`
///
/// Create an invite link for the workspace. Returns a raw token that must be
/// embedded in the join URL sent to prospective members.
///
/// # Errors
///
/// Returns `ApiError` on permission failure, missing workspace, or database error.
pub async fn post_invite_generate(
  State(state): State<InviteState>,
  AuthenticatedUser(claims): AuthenticatedUser,
  JsonBody(body): JsonBody<GenerateInviteRequest>,
) -> ApiResult<impl IntoResponse> {
  check_permission(&claims.role, Permission::ManageUsers)?;

  if body.seats < 1 {
    return Err(ApiError::BadRequest("seats must be at least 1".into()));
  }

  let token = generate_invite_token();
  let token_hash = hash_token(&token);
  let link_id = Uuid::new_v4().to_string();
  let now = now_ms();

  let expires_at: Option<i64> = body.expires_in_hours.map(|h| now + h * 3_600_000);

  sqlx::query(
    r"
      INSERT INTO invite_links (id, workspace_id, token_hash, seats_total, seats_used, expires_at, created_by)
      VALUES (?, 1, ?, ?, 0, ?, ?)
    ",
  )
  .bind(&link_id)
  .bind(&token_hash)
  .bind(body.seats)
  .bind(expires_at)
  .bind(&claims.sub)
  .execute(&state.pool)
  .await
  .map_err(|_| ApiError::Internal("Failed to create invite link".into()))?;

  let url = format!("/join/{token}");

  Ok((
    StatusCode::CREATED,
    Json(GenerateInviteResponse {
      token,
      url,
      expires_at,
      seats_total: body.seats,
    }),
  ))
}

/// DB row returned by the invite-preview query.
#[derive(sqlx::FromRow)]
struct InviteLinkRow {
  name: String,
  domain: String,
  default_model: Option<String>,
  seats_total: i64,
  seats_used: i64,
  expires_at: Option<i64>,
}

/// `GET /api/v1/invites/{token}`
///
/// Return a workspace preview for an invite link. Public - no authentication required.
///
/// # Errors
///
/// Returns `ApiError` if the token is unknown, expired, or the link is full.
pub async fn get_invite_preview(
  State(state): State<InviteState>,
  Path(token): Path<String>,
) -> ApiResult<impl IntoResponse> {
  let token_hash = hash_token(&token);

  let row: Option<InviteLinkRow> = sqlx::query_as(
    r"
      SELECT w.name, w.domain,
             wp.default_model,
             il.seats_total, il.seats_used, il.expires_at
      FROM invite_links il
      JOIN workspaces w ON w.id = il.workspace_id
      LEFT JOIN workspace_policy wp ON wp.workspace_id = il.workspace_id
      WHERE il.token_hash = ?
    ",
  )
  .bind(&token_hash)
  .fetch_optional(&state.pool)
  .await
  .map_err(|_| ApiError::Internal("Failed to query invite".into()))?;

  let row = row.ok_or_else(|| ApiError::NotFound("Invite not found".into()))?;

  if let Some(exp) = row.expires_at {
    if now_ms() > exp {
      return Err(ApiError::BadRequest("Invite link has expired".into()));
    }
  }

  let seats_remaining = row.seats_total - row.seats_used;
  if seats_remaining <= 0 {
    return Err(ApiError::BadRequest("Invite link is full".into()));
  }

  Ok((
    StatusCode::OK,
    Json(InvitePreviewResponse {
      workspace_name: row.name,
      domain: row.domain,
      default_model: row.default_model,
      seats_remaining,
    }),
  ))
}

/// Re-check the seat limit and claim one seat, all on a single connection.
///
/// Must be called between `BEGIN IMMEDIATE` and `COMMIT` on `conn`: the
/// `seats_used` re-check, the user insert, the seat insert, and the counter
/// increment must be atomic to prevent two concurrent accepts from both
/// claiming the last seat (TOCTOU race).
async fn claim_invite_seat(
  conn: &mut PoolConnection<Sqlite>,
  token_hash: &str,
  user_id: &str,
  username: &str,
  password_hash: &str,
  user_email: &str,
) -> Result<(), ApiError> {
  let row = sqlx::query_as::<_, (String, i64, i64, Option<i64>)>(
    r"
      SELECT id, seats_total, seats_used, expires_at
      FROM invite_links
      WHERE token_hash = ?
    ",
  )
  .bind(token_hash)
  .fetch_optional(conn.as_mut())
  .await
  .map_err(|_| ApiError::Internal("Failed to query invite".into()))?;

  let (link_id, seats_total, seats_used, expires_at) =
    row.ok_or_else(|| ApiError::NotFound("Invite not found".into()))?;

  if let Some(exp) = expires_at {
    if now_ms() > exp {
      return Err(ApiError::BadRequest("Invite link has expired".into()));
    }
  }

  if seats_used >= seats_total {
    return Err(ApiError::BadRequest("Invite link is full".into()));
  }

  sqlx::query(
    r"
      INSERT INTO users (id, username, password_hash, email, role, is_active, created_at)
      VALUES (?, ?, ?, ?, 'developer', 1, ?)
    ",
  )
  .bind(user_id)
  .bind(username)
  .bind(password_hash)
  .bind(user_email)
  .bind(now_ms())
  .execute(conn.as_mut())
  .await
  .map_err(|_| ApiError::Internal("Failed to create member account".into()))?;

  sqlx::query(
    r"
      INSERT INTO invite_seats (invite_link_id, user_id, accepted_at)
      VALUES (?, ?, ?)
    ",
  )
  .bind(&link_id)
  .bind(user_id)
  .bind(now_ms())
  .execute(conn.as_mut())
  .await
  .map_err(|_| ApiError::Internal("Failed to record invite seat".into()))?;

  sqlx::query(
    r"
      UPDATE invite_links SET seats_used = seats_used + 1 WHERE id = ?
    ",
  )
  .bind(&link_id)
  .execute(conn.as_mut())
  .await
  .map_err(|_| ApiError::Internal("Failed to update seat count".into()))?;

  Ok(())
}

/// `POST /api/v1/invites/{token}/accept`
///
/// Create a member account and claim a seat on this invite link.
/// Returns a JWT access token the new member can use immediately.
/// Public - no authentication required.
///
/// # Errors
///
/// Returns `ApiError` if the token is invalid, expired, full, password is too
/// short, or the database operation fails.
pub async fn post_invite_accept(
  State(state): State<InviteState>,
  Path(token): Path<String>,
  JsonBody(body): JsonBody<AcceptInviteRequest>,
) -> ApiResult<impl IntoResponse> {
  if body.password.len() < 8 {
    return Err(ApiError::BadRequest(
      "Password must be at least 8 characters".into(),
    ));
  }

  let token_hash = hash_token(&token);

  // Generate the identity and hash the password BEFORE opening the write
  // transaction: bcrypt is deliberately expensive, and we must not hold the
  // SQLite write lock for its duration.
  let user_id = Uuid::new_v4().to_string();
  let username = format!("member_{}", &user_id[..8]);
  let user_email = format!("{user_id}@invite.local");
  let password_hash = bcrypt::hash(&body.password, BCRYPT_COST)
    .map_err(|_| ApiError::Internal("Failed to hash password".into()))?;

  // Claim the seat inside a single BEGIN IMMEDIATE transaction so the
  // seats_used check and the counter increment are atomic. pool.begin() emits
  // BEGIN (deferred); we need BEGIN IMMEDIATE to block concurrent accepts from
  // both passing the check on a full link (TOCTOU race).
  let mut conn = state
    .pool
    .acquire()
    .await
    .map_err(|_| ApiError::Internal("Failed to acquire connection".into()))?;
  sqlx::query("BEGIN IMMEDIATE")
    .execute(conn.as_mut())
    .await
    .map_err(|_| ApiError::Internal("Failed to begin transaction".into()))?;

  let claim = claim_invite_seat(
    &mut conn,
    &token_hash,
    &user_id,
    &username,
    &password_hash,
    &user_email,
  )
  .await;

  match claim {
    Ok(()) => {
      sqlx::query("COMMIT")
        .execute(conn.as_mut())
        .await
        .map_err(|_| ApiError::Internal("Failed to commit transaction".into()))?;
    }
    Err(e) => {
      // Best-effort rollback; surface the original error.
      let _ = sqlx::query("ROLLBACK").execute(conn.as_mut()).await;
      return Err(e);
    }
  }

  let token_jti = Uuid::new_v4().to_string();
  let access_token = state
    .jwt_secret
    .generate_access_token(&user_id, &user_email, "developer", &token_jti)
    .map_err(|_| ApiError::Internal("Failed to generate access token".into()))?;

  Ok((
    StatusCode::CREATED,
    Json(AcceptInviteResponse {
      user_id,
      access_token,
    }),
  ))
}

/// `POST /api/v1/invites/{token}/approve`
///
/// Mark a pending invite-seat member as approved. The seat is identified by
/// the invite link token and the member's user ID.
///
/// # Errors
///
/// Returns `ApiError` on permission failure, unknown token, unknown user, or
/// database error.
pub async fn post_invite_approve(
  State(state): State<InviteState>,
  AuthenticatedUser(claims): AuthenticatedUser,
  Path(token): Path<String>,
  JsonBody(body): JsonBody<ApproveInviteRequest>,
) -> ApiResult<impl IntoResponse> {
  check_permission(&claims.role, Permission::ManageUsers)?;

  let token_hash = hash_token(&token);

  let link_id: Option<String> = sqlx::query_scalar(
    r"
      SELECT id FROM invite_links WHERE token_hash = ?
    ",
  )
  .bind(&token_hash)
  .fetch_optional(&state.pool)
  .await
  .map_err(|_| ApiError::Internal("Failed to query invite link".into()))?;

  let link_id = link_id.ok_or_else(|| ApiError::NotFound("Invite not found".into()))?;

  let approved_at = now_ms();
  let rows_affected = sqlx::query(
    r"
      UPDATE invite_seats
      SET approved_at = ?, approved_by = ?
      WHERE invite_link_id = ? AND user_id = ? AND approved_at IS NULL
    ",
  )
  .bind(approved_at)
  .bind(&claims.sub)
  .bind(&link_id)
  .bind(&body.user_id)
  .execute(&state.pool)
  .await
  .map_err(|_| ApiError::Internal("Failed to approve invite seat".into()))?
  .rows_affected();

  if rows_affected == 0 {
    return Err(ApiError::NotFound(
      "Pending seat not found for this user".into(),
    ));
  }

  Ok(StatusCode::OK)
}

/// One row in the pending-invites listing.
#[derive(Debug, Serialize)]
pub struct PendingInviteSeat {
  /// Seat ID - stable identifier used to approve via `/invites/seats/{id}/approve`.
  pub seat_id: i64,
  /// User ID of the pending member.
  pub user_id: String,
  /// Member username (generated by the accept handler).
  pub username: String,
  /// Member email (generated by the accept handler).
  pub email: String,
  /// Unix-ms timestamp of when the member accepted the invite.
  pub accepted_at: Option<i64>,
  /// Workspace name the invite belongs to.
  pub workspace_name: String,
  /// Invite link ID this seat belongs to.
  pub invite_link_id: String,
}

/// Response from `GET /api/v1/invites/pending`.
#[derive(Debug, Serialize)]
pub struct PendingInvitesResponse {
  /// List of pending invite seats awaiting admin approval.
  pub pending: Vec<PendingInviteSeat>,
}

/// DB row for the pending invites query.
#[derive(sqlx::FromRow)]
struct PendingInviteSeatRow {
  seat_id: i64,
  user_id: Option<String>,
  username: Option<String>,
  email: Option<String>,
  accepted_at: Option<i64>,
  workspace_name: String,
  invite_link_id: String,
}

/// `GET /api/v1/invites/pending`
///
/// List all invite seats that have been accepted but not yet approved by an
/// admin. Returned seats can be approved via `POST /invites/seats/{id}/approve`.
///
/// # Errors
///
/// Returns `ApiError` on permission failure or database error.
pub async fn get_pending_invites(
  State(state): State<InviteState>,
  AuthenticatedUser(claims): AuthenticatedUser,
) -> ApiResult<impl IntoResponse> {
  check_permission(&claims.role, Permission::ManageUsers)?;

  let rows: Vec<PendingInviteSeatRow> = sqlx::query_as(
    r"
      SELECT s.id AS seat_id,
             s.user_id,
             u.username,
             u.email,
             s.accepted_at,
             w.name AS workspace_name,
             s.invite_link_id
      FROM invite_seats s
      JOIN invite_links il ON il.id = s.invite_link_id
      JOIN workspaces w ON w.id = il.workspace_id
      LEFT JOIN users u ON u.id = s.user_id
      WHERE s.approved_at IS NULL AND s.user_id IS NOT NULL
      ORDER BY s.accepted_at DESC
    ",
  )
  .fetch_all(&state.pool)
  .await
  .map_err(|_| ApiError::Internal("Failed to query pending invites".into()))?;

  let pending = rows
    .into_iter()
    .map(|r| PendingInviteSeat {
      seat_id: r.seat_id,
      user_id: r.user_id.unwrap_or_default(),
      username: r.username.unwrap_or_default(),
      email: r.email.unwrap_or_default(),
      accepted_at: r.accepted_at,
      workspace_name: r.workspace_name,
      invite_link_id: r.invite_link_id,
    })
    .collect();

  Ok((StatusCode::OK, Json(PendingInvitesResponse { pending })))
}

/// `POST /api/v1/invites/seats/{seat_id}/approve`
///
/// Mark a pending invite-seat as approved. Identifies the seat directly by its
/// primary key, which is more practical for admin UIs than the token-scoped
/// `POST /invites/{token}/approve` endpoint (since raw tokens are not
/// retrievable after generation).
///
/// # Errors
///
/// Returns `ApiError` on permission failure, unknown seat, already-approved
/// seat, or database error.
pub async fn post_invite_seat_approve(
  State(state): State<InviteState>,
  AuthenticatedUser(claims): AuthenticatedUser,
  Path(seat_id): Path<i64>,
) -> ApiResult<impl IntoResponse> {
  check_permission(&claims.role, Permission::ManageUsers)?;

  let approved_at = now_ms();
  let rows_affected = sqlx::query(
    r"
      UPDATE invite_seats
      SET approved_at = ?, approved_by = ?
      WHERE id = ? AND approved_at IS NULL
    ",
  )
  .bind(approved_at)
  .bind(&claims.sub)
  .bind(seat_id)
  .execute(&state.pool)
  .await
  .map_err(|_| ApiError::Internal("Failed to approve invite seat".into()))?
  .rows_affected();

  if rows_affected == 0 {
    return Err(ApiError::NotFound(
      "Pending seat not found or already approved".into(),
    ));
  }

  Ok(StatusCode::OK)
}
