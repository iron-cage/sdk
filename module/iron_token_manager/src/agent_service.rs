//! Agent management service
//!
//! Provides agent management operations for managing agents:
//! - Create agents
//! - List agents
//! - Get agent details
//! - Update agent details
//! - Delete agents
//! - Get agent status
//! - Stop agent
//!
//! All endpoints require Admin role (ManageAgents permission).

use sqlx::SqlitePool;

/// Agent management service
/// 
/// Provides agent management operations for managing agents:
/// - Create agents
/// - List agents
/// - Get agent details
/// - Update agent details
/// - Delete agents
/// - Get agent status
/// - Stop agent
///
#[ derive( Debug, Clone ) ]
pub struct AgentService 
{
    db_pool: SqlitePool
}

impl AgentService {
    /// Create new agent service
    pub fn new(db_pool: SqlitePool) -> Self {
        Self { db_pool }
    }

    /// Create a new agent
    pub async fn create_agent(
        &self, 
        name: &str, 
        user_id: &str,
        budget: f64,
        status: &str, 
        providers: Vec<String>, 
        description: Option<String>, 
        tags: Option<Vec<String>>
    ) -> Result<String, sqlx::Error> 
    {
        let created_at = chrono::Utc::now().timestamp();
        let mut agent_prefix = "agent_".to_string();
        let agent_id = uuid::Uuid::new_v4().to_string();
        agent_prefix.push_str(&agent_id);
        let tags_string = match tags {
            Some(t) => serde_json::to_string(&t).unwrap_or("[]".to_string()),
            None => "[]".to_string(),
        };

        let result = sqlx::query!(
            "
            INSERT INTO agents (id, name, user_id, budget, status, description, tags, created_at)
            VALUES (?, ?, ?, ?, ?, strftime('%s', 'now'))
            "
        )
        .bind(&agent_prefix)
        .bind(name)
        .bind(user_id)
        .bind(budget)
        .bind(status)
        .bind(tags_string)
        .bind(description)
        .bind(created_at)
        .execute(&self.db_pool)
        .await?;

        // TODO: Create audit log

        return Ok( agent_prefix)
    }

}