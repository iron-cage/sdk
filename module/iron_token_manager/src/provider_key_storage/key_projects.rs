//! Provider key project assignment operations
//!
//! Assign, unassign, and query key-to-project mappings.

use std::collections::HashMap;

use crate::error::{Result, TokenError};

use super::{current_time_ms, ProviderKeyStorage};

impl ProviderKeyStorage {
  /// Assign a key to a project
  ///
  /// # Errors
  ///
  /// Returns error if database insert fails
  pub async fn assign_to_project(&self, key_id: i64, project_id: &str) -> Result<()> {
    let now_ms = current_time_ms();
    sqlx::query(
      "INSERT OR REPLACE INTO project_provider_key_assignments \
       ( project_id, provider_key_id, assigned_at ) VALUES ( $1, $2, $3 )",
    )
    .bind(project_id)
    .bind(key_id)
    .bind(now_ms)
    .execute(&self.pool)
    .await
    .map_err(TokenError::Database)?;
    Ok(())
  }

  /// Remove key assignment from a project
  ///
  /// # Errors
  ///
  /// Returns error if database delete fails
  pub async fn unassign_from_project(&self, key_id: i64, project_id: &str) -> Result<()> {
    sqlx::query(
      "DELETE FROM project_provider_key_assignments \
       WHERE project_id = $1 AND provider_key_id = $2",
    )
    .bind(project_id)
    .bind(key_id)
    .execute(&self.pool)
    .await
    .map_err(TokenError::Database)?;
    Ok(())
  }

  /// Get key assigned to a project
  ///
  /// Returns the most recently assigned key for the project, or `None` if no key is assigned.
  /// Uses `ORDER BY assigned_at DESC LIMIT 1` to give deterministic results when multiple
  /// assignments exist.
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn get_project_key(&self, project_id: &str) -> Result<Option<i64>> {
    let row: Option<(i64,)> = sqlx::query_as(
      "SELECT provider_key_id FROM project_provider_key_assignments \
       WHERE project_id = $1 ORDER BY assigned_at DESC LIMIT 1",
    )
    .bind(project_id)
    .fetch_optional(&self.pool)
    .await
    .map_err(TokenError::Database)?;

    Ok(row.map(|r| r.0))
  }

  /// Get all project assignments for a key
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn get_key_projects(&self, key_id: i64) -> Result<Vec<String>> {
    let rows: Vec<(String,)> = sqlx::query_as(
      "SELECT project_id FROM project_provider_key_assignments WHERE provider_key_id = $1",
    )
    .bind(key_id)
    .fetch_all(&self.pool)
    .await
    .map_err(TokenError::Database)?;

    Ok(rows.into_iter().map(|r| r.0).collect())
  }

  /// Verify that a user owns a project by checking the `api_tokens` table.
  ///
  /// Returns `true` if a row exists in `api_tokens` for the given project and user,
  /// `false` otherwise.
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn verify_project_owner(&self, project_id: &str, user_id: &str) -> Result<bool> {
    let exists: bool = sqlx::query_scalar(
      "SELECT EXISTS(SELECT 1 FROM api_tokens WHERE project_id = $1 AND user_id = $2)",
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_one(&self.pool)
    .await
    .map_err(TokenError::Database)?;

    Ok(exists)
  }

  /// Get all project assignments for multiple keys in a single query
  ///
  /// Returns a map from key ID to list of project IDs.
  /// Keys with no assignments are absent from the map.
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn get_all_key_projects(&self, key_ids: &[i64]) -> Result<HashMap<i64, Vec<String>>> {
    if key_ids.is_empty() {
      return Ok(HashMap::new());
    }
    // SQLite limits bind parameters to 999 per statement; chunk to stay within that.
    if key_ids.len() > 999 {
      let mut result: HashMap<i64, Vec<String>> = HashMap::new();
      for chunk in key_ids.chunks(999) {
        let partial = self.get_all_key_projects_batch(chunk).await?;
        for (k, v) in partial {
          result.entry(k).or_default().extend(v);
        }
      }
      return Ok(result);
    }
    self.get_all_key_projects_batch(key_ids).await
  }

  /// Inner helper: run a single batched query for up to 999 key IDs at a time.
  async fn get_all_key_projects_batch(
    &self,
    key_ids: &[i64],
  ) -> Result<HashMap<i64, Vec<String>>> {
    // Build parameterized IN clause
    let placeholders = key_ids
      .iter()
      .enumerate()
      .map(|(i, _)| format!("${}", i + 1))
      .collect::<Vec<_>>()
      .join(", ");
    let sql = format!(
      "SELECT provider_key_id, project_id \
       FROM project_provider_key_assignments \
       WHERE provider_key_id IN ({placeholders})"
    );
    let mut query = sqlx::query_as::<_, (i64, String)>(&sql);
    for id in key_ids {
      query = query.bind(id);
    }
    let rows = query.fetch_all(&self.pool).await.map_err(TokenError::Database)?;
    let mut map: HashMap<i64, Vec<String>> = HashMap::new();
    // qqq: [Low] result order within each key's project list is non-deterministic — add ORDER BY project_id if stable ordering matters
    for (key_id, project_id) in rows {
      map.entry(key_id).or_default().push(project_id);
    }
    Ok(map)
  }
}
