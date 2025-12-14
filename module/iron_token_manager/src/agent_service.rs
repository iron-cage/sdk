//! Agent management service
//!
//! Provides operations for agent lifecycle management: create, update, delete,
//! list agents, and get agent tokens. Authorization is handled at the service layer.

use std::collections::HashSet;

use chrono::DateTime;
use sqlx::{ Row, SqlitePool, sqlite::SqliteRow };
use crate::error::Result;
use tracing::error;

/// IC Token associated with an agent
#[ derive( Debug, Clone ) ]
pub struct ICToken
{
  /// Token ID
  pub id: String,
  /// Token value (for display, not the actual secret)
  pub token: String,
  /// Creation timestamp (ISO 8601 string)
  pub created_at: String,
  /// Last used timestamp (ISO 8601 string)
  pub last_used: Option< String >,
}

/// Agent data returned from database
#[ derive( Debug, Clone ) ]
pub struct Agent
{
  /// Agent ID (format: agent_<uuid>)
  pub id: String,
  /// Agent name
  pub name: String,
  /// Budget allocation in USD (from `agent_budgets` table)
  pub budget: f64,
  /// Amount spent in USD (from `agent_budgets` table)
  pub spent: f64,
  /// Remaining budget in USD (budget - spent)
  pub remaining: f64,
  /// Percentage of budget used (0.0 - 100.0)
  pub percent_used: f64,
  /// Allowed providers for this agent
  pub providers: Vec< String >,
  /// Agent description
  pub description: Option< String >,
  /// Tags for categorization
  pub tags: Option< Vec< String > >,
  /// Owner user ID
  pub user_id: String,
  /// Associated project ID
  pub project_id: Option< String >,
  /// IC Token for agent authentication
  pub ic_token: Option< ICToken >,
  /// Agent status (active, exhausted, inactive)
  pub status: String,
  /// Creation timestamp (ISO 8601 string)
  pub created_at: String,
  /// Last update timestamp (ISO 8601 string)
  pub updated_at: String,
}

/// Agent creation parameters
#[ derive( Debug, Clone ) ]
pub struct CreateAgentParams
{
  /// Agent name
  pub name: String,
  /// Budget allocation in USD
  pub budget: f64,
  /// Allowed providers
  pub providers: Option< Vec< String > >,
  /// Description
  pub description: Option< String >,
  /// Tags
  pub tags: Option< Vec< String > >,
  /// Associated project ID
  pub project_id: Option< String >,
}

/// Agent update parameters
#[ derive( Debug, Clone ) ]
pub struct UpdateAgentParams
{
  /// New name (optional)
  pub name: Option< String >,
  /// New description (optional)
  pub description: Option< String >,
  /// New tags (optional)
  pub tags: Option< Vec< String > >,
}

/// Token item for agent tokens listing
#[ derive( Debug, Clone ) ]
pub struct AgentTokenItem
{
  /// Token ID
  pub id: i64,
  /// User ID who owns the token
  pub user_id: String,
  /// Provider name
  pub provider: Option< String >,
  /// Token name
  pub name: Option< String >,
  /// Creation timestamp (milliseconds since epoch)
  pub created_at: i64,
  /// Last used timestamp (milliseconds since epoch)
  pub last_used_at: Option< i64 >,
  /// Whether token is active
  pub is_active: bool,
}

/// Provider item for agent providers listing
#[ derive( Debug, Clone ) ]
pub struct ProviderListItem
{
  /// Provider ID
  pub id: String,
  /// Provider name
  pub name: String,
  /// Base url endpoint type
  pub endpoint: String,
  /// List of supported models
  pub models: Vec< String >,
  /// Status (active/inactive)
  pub status: String,
}


/// Agent details
/// 
/// Represents an agent with additional details for communicating with endpoint providers
#[derive(Debug, Clone)]
pub struct AgentDetails
{
/// Agent ID (format: agent_<uuid>)
  pub id: String,
  /// Agent name
  pub name: String,
  /// Budget allocation in USD (from `agent_budgets` table)
  pub budget: f64,
  /// Amount spent in USD (from `agent_budgets` table)
  pub spent: f64,
  /// Remaining budget in USD (budget - spent)
  pub remaining: f64,
  /// Percentage of budget used (0.0 - 100.0)
  pub percent_used: f64,
  /// Allowed providers for this agent
  pub providers: Vec< ProviderListItem >,
  /// Agent description
  pub description: Option< String >,
  /// Tags for categorization
  pub tags: Option< Vec< String > >,
  /// Owner user ID
  pub user_id: String,
  /// Associated project ID
  pub project_id: Option< String >,
  /// IC Token for agent authentication
  pub ic_token: Option< ICToken >,
  /// Agent status (active, exhausted, inactive)
  pub status: String,
  /// Creation timestamp (ISO 8601 string)
  pub created_at: String,
  /// Last update timestamp (ISO 8601 string)
  pub updated_at: String,  
}

/// Sort direction
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum SortDirection
{
  /// Ascending sort
  Asc,
  /// Descending sort
  Desc,
}

/// Sort field for agents
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum AgentSortField
{
  /// Sort by agent name
  Name,
  /// Sort by agent budget
  Budget,
  /// Sort by agent creation time
  CreatedAt,
}

/// Agent listing filters
#[ derive( Debug, Clone, Default ) ]
pub struct ListAgentsFilters
{
  /// Filter by owner (None for admin to see all)
  pub user_id: Option< String >,
  /// Filter by name (partial match, case-insensitive)
  pub name: Option< String >,
  /// Filter by status
  pub status: Option< String >,
  /// Page number (1-indexed)
  pub page: Option< u32 >,
  /// Results per page
  pub per_page: Option< u32 >,
  /// Sort field
  pub sort_field: Option< AgentSortField >,
  /// Sort direction
  pub sort_direction: Option< SortDirection >,
}

/// Brief provider item for agent providers listing
#[derive(Debug, Clone)]
pub struct ProviderListItemBrief
{
  /// Provider ID
  pub id: String,
  /// Provider name
  pub name: String,
}

/// Result for paginated agent listing
#[ derive( Debug, Clone ) ]
pub struct ListAgentsResult
{
  /// List of agents
  pub agents: Vec< Agent >,
  /// Total count of matching agents
  pub total: u64,
}

/// Agent service error types
#[ derive( Debug, Clone, PartialEq, Eq ) ]
pub enum AgentServiceError
{
  /// Agent not found
  NotFound,
  /// Access forbidden
  Forbidden,
  /// Database error
  Database,
  /// JSON serialization error
  Json,
}

/// Agent management service
///
/// Handles agent lifecycle operations with authorization checks.
#[ derive( Debug, Clone ) ]
pub struct AgentService
{

  pool: SqlitePool,
}

impl AgentService
{
  /// Create new agent service
  ///
  /// # Arguments
  ///
  /// * `pool` - Database connection pool
  #[ must_use ]
  pub fn new( pool: SqlitePool ) -> Self
  {
    Self { pool }
  }

  /// List all agents with filtering, pagination, and sorting
  ///
  /// # Arguments
  ///
  /// * `filters` - Filters including `user_id`, `name`, `status`, pagination, and sorting
  ///
  /// # Returns
  ///
  /// Paginated result with agents and total count
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn list_agents( &self, filters: ListAgentsFilters ) -> Result< ListAgentsResult >
  {
    // Build WHERE clause
    let mut conditions = Vec::new();
    let mut bind_values: Vec< String > = Vec::new();

    if let Some( ref user_id ) = filters.user_id
    {
      conditions.push( "a.user_id = ?" );
      bind_values.push( user_id.clone() );
    }

    if let Some( ref name ) = filters.name
    {
      conditions.push( "LOWER(a.name) LIKE LOWER(?)" );
      bind_values.push( format!( "%{name}%" ) );
    }

    if let Some( ref status ) = filters.status
    {
      conditions.push( "a.status = ?" );
      bind_values.push( status.clone() );
    }

    let where_clause = if conditions.is_empty()
    {
      String::new()
    }
    else
    {
      format!( "WHERE {}", conditions.join( " AND " ) )
    };

    // Build ORDER BY clause
    let sort_field = filters.sort_field.unwrap_or( AgentSortField::CreatedAt );
    let sort_direction = filters.sort_direction.unwrap_or( SortDirection::Desc );

    let sort_column = match sort_field
    {
      AgentSortField::Name => "a.name",
      AgentSortField::Budget => "b.total_allocated",
      AgentSortField::CreatedAt => "a.created_at",
    };

    let sort_dir = match sort_direction
    {
      SortDirection::Asc => "ASC",
      SortDirection::Desc => "DESC",
    };

    let order_clause = format!( "ORDER BY {sort_column} {sort_dir}" );

    // Pagination
    let page = filters.page.unwrap_or( 1 ).max( 1 );
    let per_page = filters.per_page.unwrap_or( 50 ).min( 100 );
    let offset = ( page - 1 ) * per_page;

    // Count query
    let count_sql = format!( "SELECT COUNT(*) as count FROM agents a {where_clause}" );
    let mut count_query = sqlx::query_scalar::< _, i64 >( &count_sql );
    for value in &bind_values
    {
      count_query = count_query.bind( value );
    }
    let total: i64 = count_query
      .fetch_one( &self.pool )
      .await
      .map_err( |e| { error!( "Error counting agents: {}", e ); crate::error::TokenError::Generic } )?;

    // Data query
    let data_sql = format!(
      r#"
      SELECT
        a.id, a.name, a.providers, a.description, a.tags, a.user_id, a.project_id, a.status, a.created_at, a.updated_at,
        b.total_allocated as budget, b.total_spent as spent, b.budget_remaining as remaining
      FROM agents a
      LEFT JOIN agent_budgets b ON a.id = b.agent_id
      {where_clause}
      {order_clause}
      LIMIT ? OFFSET ?
      "#
    );

    let mut data_query = sqlx::query( &data_sql );
    for value in &bind_values
    {
      data_query = data_query.bind( value );
    }
    data_query = data_query.bind( per_page ).bind( offset );

    let rows = data_query
      .fetch_all( &self.pool )
      .await
      .map_err( |e| { error!( "Error listing agents: {}", e ); crate::error::TokenError::Generic } )?;

    let agents = rows.iter().map( |row| {
      Self::row_to_agent( row )
    } ).collect();

    Ok( ListAgentsResult {
      agents,
      total: total as u64,
    } )
  }

  /// Get a single agent by ID
  ///
  /// # Arguments
  ///
  /// * `id` - Agent ID (string format: agent_<uuid>)
  ///
  /// # Returns
  ///
  /// Agent data or error if not found
  ///
  /// # Errors
  ///
  /// Returns error if agent not found or database query fails
  pub async fn get_agent( &self, id: &str ) -> Result< Option< Agent > >
  {
    let row = sqlx::query(
      r#"
      SELECT
        a.id, a.name, a.providers, a.description, a.tags, a.user_id, a.project_id, a.status, a.created_at, a.updated_at,
        b.total_allocated as budget, b.total_spent as spent, b.budget_remaining as remaining
      FROM agents a
      LEFT JOIN agent_budgets b ON a.id = b.agent_id
      WHERE a.id = ?
      "#
    )
    .bind( id )
    .fetch_optional( &self.pool )
    .await
    .map_err( |e| { error!( "Error getting agent: {}", e ); crate::error::TokenError::Generic } )?;

    Ok( row.map( |row| Self::row_to_agent( &row ) ) )
  }

  /// Create a new agent
  ///
  /// # Arguments
  ///
  /// * `params` - Agent creation parameters
  /// * `user_id` - ID of user who will own the agent
  ///
  /// # Returns
  ///
  /// Created agent
  ///
  /// # Errors
  ///
  /// Returns error if database insert fails
  pub async fn create_agent( &self, params: CreateAgentParams, user_id: &str ) -> Result< Agent >
  {
    let agent_id = format!( "agent_{}", uuid::Uuid::new_v4() );
    let providers_json = serde_json::to_string( &params.providers.clone().unwrap_or_default() )
      .map_err( |e| { error!( "Error serializing providers: {}", e ); crate::error::TokenError::Generic } )?;
    let tags_json = serde_json::to_string( &params.tags.clone().unwrap_or_default() )
      .map_err( |e| { error!( "Error serializing tags: {}", e ); crate::error::TokenError::Generic } )?;
    let now = chrono::Utc::now().timestamp();
    let status = "active".to_string();

    if let Some(providers) = &params.providers {
      for provider in providers {
        let provider = sqlx::query("SELECT id FROM ai_provider_keys WHERE id = ?")
          .bind(provider)
          .fetch_optional(&self.pool)
          .await
          .map_err(|e| { error!("Error getting provider: {}", e); crate::error::TokenError::Generic })?;

        if provider.is_none()  {
          return Err(crate::error::TokenError::Generic);
        }
      }
    }

    sqlx::query(
      r#"
      INSERT INTO agents (id, name, providers, description, tags, user_id, project_id, status, created_at, updated_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      "#
    )
    .bind( &agent_id )
    .bind( &params.name )
    .bind( &providers_json )
    .bind( &params.description )
    .bind( &tags_json )
    .bind( user_id )
    .bind( &params.project_id )
    .bind( &status )
    .bind( now )
    .bind( now )
    .execute( &self.pool )
    .await
    .map_err( |e| { error!( "Error creating agent: {}", e ); crate::error::TokenError::Generic } )?;

    sqlx::query(
      r#"
      INSERT INTO agent_budgets (agent_id, total_allocated, budget_remaining, created_at, updated_at) VALUES (?, ?, ?, ?, ?)
      "#
    )
    .bind(&agent_id)
    .bind(params.budget)
    .bind(params.budget)
    .bind(&now)
    .bind(&now)
    .execute(&self.pool)
    .await
    .map_err(|e| {
      error!("Error creating budget lease: {}", e);
      crate::error::TokenError::Generic 
    })?;

    self.get_agent( &agent_id )
      .await?
      .ok_or_else( || {
        error!( "Failed to fetch created agent: {}", agent_id );
        crate::error::TokenError::Generic 
      } )
  }

  /// Update an existing agent
  ///
  /// # Arguments
  ///
  /// * `id` - Agent ID to update (string format: agent_<uuid>)
  /// * `params` - Update parameters
  ///
  /// # Returns
  ///
  /// Updated agent or None if not found
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn update_agent( &self, id: &str, params: UpdateAgentParams ) -> Result< Option< Agent > >
  {
    // Check if agent exists
    let existing: Option< String > = sqlx::query_scalar( "SELECT id FROM agents WHERE id = ?" )
      .bind( id )
      .fetch_optional( &self.pool )
      .await
      .map_err( |e| { error!( "Error checking agent exists: {}", e ); crate::error::TokenError::Generic } )?;

    if existing.is_none()
    {
      return Ok( None );
    }

    let now = chrono::Utc::now().timestamp();

    // Update fields if provided
    if let Some( ref name ) = params.name
    {
      sqlx::query( "UPDATE agents SET name = ?, updated_at = ? WHERE id = ?" )
        .bind( name )
        .bind( now )
        .bind( id )
        .execute( &self.pool )
        .await
        .map_err( |e| { error!( "Error updating agent name: {}", e ); crate::error::TokenError::Generic } )?;
    }

    if let Some( ref description ) = params.description
    {
      sqlx::query( "UPDATE agents SET description = ?, updated_at = ? WHERE id = ?" )
        .bind( description )
        .bind( now )
        .bind( id )
        .execute( &self.pool )
        .await
        .map_err( |e| { error!( "Error updating agent description: {}", e ); crate::error::TokenError::Generic } )?;
    }

    if let Some( ref tags ) = params.tags
    {
      let tags_json = serde_json::to_string( tags )
        .map_err( |e| { error!( "Error serializing tags: {}", e ); crate::error::TokenError::Generic } )?;
      sqlx::query( "UPDATE agents SET tags = ?, updated_at = ? WHERE id = ?" )
        .bind( &tags_json )
        .bind( now )
        .bind( id )
        .execute( &self.pool )
        .await
        .map_err( |e| { error!( "Error updating agent tags: {}", e ); crate::error::TokenError::Generic } )?;
    }

    // Fetch and return updated agent
    self.get_agent( id ).await
  }

  /// Get agent details
  ///
  /// # Arguments
  ///
  /// * `id` - Agent ID to get details for (string format: agent_<uuid>)
  ///
  /// # Returns
  ///
  /// Agent details if found, None if not found
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn get_agent_details( &self, id: &str ) -> Result< Option< AgentDetails > >
  {
    let row = sqlx::query(
      r#"
      SELECT
        a.id, a.name, a.providers, a.description, a.tags, a.user_id, a.project_id, a.status, a.created_at, a.updated_at,
        b.total_allocated as budget, b.total_spent as spent, b.budget_remaining as remaining
      FROM agents a
      LEFT JOIN agent_budgets b ON a.id = b.agent_id
      WHERE a.id = ?
      "#
    )
    .bind( id )
    .fetch_optional( &self.pool )
    .await
    .map_err( |e| { error!( "Error getting agent: {}", e ); crate::error::TokenError::Generic } )?;

    if let Some(row) = row
    {
      let agent = Self::row_to_agent( &row );

      let mut providers: Vec<ProviderListItem> = Vec::new();

      for provider_id in agent.providers.iter()
      {
        let row = sqlx::query( "SELECT id, provider, base_url, models, is_enabled FROM ai_provider_keys WHERE id = ?")
          .bind( provider_id )
          .fetch_optional( &self.pool )
          .await
          .map_err( |e| { error!( "Error getting provider key: {}", e ); crate::error::TokenError::Generic } )?;

        if let Some(row) = row {
          providers.push(Self::row_to_provider_list_item( &row ));
        }
      }

      Ok( Some(AgentDetails {
        id: agent.id,
        name: agent.name,
        budget: agent.budget,
        spent: agent.spent,
        remaining: agent.remaining,
        percent_used: agent.percent_used,
        providers: providers,
        description: agent.description,
        tags: agent.tags,
        user_id: agent.user_id,
        project_id: agent.project_id,
        ic_token: agent.ic_token,
        status: agent.status,
        created_at: agent.created_at,
        updated_at: agent.updated_at,
      } ))
    }
    else
    {
      Ok( None )
    }
  }

  fn row_to_provider_list_item(row: &SqliteRow) -> ProviderListItem {
    let models_json: Option<String> = row.get("models");
    let models = models_json
      .as_ref()
      .and_then(|json| serde_json::from_str(json).ok())
      .unwrap_or_default();

    let is_enabled: bool = row.get("is_enabled");
    let status = if is_enabled { "active".to_string() } else { "inactive".to_string() };

    ProviderListItem {
      id: row.get("id"),
      name: row.get("provider"),
      endpoint: row.get("base_url"),
      models,
      status,
    }
  }

  /// Assign providers to an agent
  ///
  /// # Arguments
  ///
  /// * `id` - Agent ID to assign providers to (string format: agent_<uuid>)
  /// * `providers` - Vector of provider IDs to assign to the agent
  ///
  /// # Returns
  ///
  /// True if providers were assigned successfully, false if not found
  ///
  /// # Errors
  ///
  /// Returns error if database query fails

  pub async fn assign_providers_to_agent(&self, id: &str, providers: Vec<String>) -> Result <Option<Agent>> {
    // Validate providers exist    
    for provider in &providers {
      let provider = sqlx::query("SELECT id FROM ai_provider_keys WHERE id = ?")
        .bind(provider)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| { error!("Error getting provider: {}", e); crate::error::TokenError::Generic })?;

      if provider.is_none() {
        return Ok(None);
      }
    }

    // Deduplicate providers
    let providers = providers.into_iter().collect::<HashSet<_>>().into_iter().collect::<Vec<_>>();

    // Convert providers to JSON
    let providers_json = serde_json::to_string(&providers)
      .map_err(|e| { error!("Error serializing providers: {}", e); crate::error::TokenError::Generic })?;

    // Update agent providers
    sqlx::query("UPDATE agents SET providers = ? WHERE id = ?")
      .bind(providers_json)
      .bind(id)
      .execute(&self.pool)
      .await
      .map_err(|e| { error!("Error updating agent providers: {}", e); crate::error::TokenError::Generic })?;

    self.get_agent(id).await
  }




  /// Remove provider from agent
  /// 
  /// # Arguments
  ///
  /// * `id` - Agent ID to remove provider from (string format: agent_<uuid>)
  /// * `provider_id` - Provider ID to remove from the agent (string format: provider_<uuid>)
  ///
  /// # Returns
  ///
  /// True if provider was removed, false if not found
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn remove_provider_from_agent(&self, id: &str, provider_id: &str) -> Result<Vec<ProviderListItemBrief>> {
    let providers = sqlx::query("SELECT providers FROM agents WHERE id = ?")
      .bind(id)
      .fetch_optional(&self.pool)
      .await
      .map_err(|e| { error!("Error removing provider from agent: {}", e); crate::error::TokenError::Generic })?;

    if let Some(row) = providers {
      let providers_json: String = row.get("providers");
      let mut providers: Vec<String> = serde_json::from_str(&providers_json)
        .map_err(|e| { error!("Error parsing providers: {}", e); crate::error::TokenError::Generic })?;

      if let Some(pos) = providers.iter().position(|x| x == provider_id) {
        providers.remove(pos);

        let providers_json = serde_json::to_string(&providers)
          .map_err(|e| { error!("Error serializing providers: {}", e); crate::error::TokenError::Generic })?;

        sqlx::query("UPDATE agents SET providers = ? WHERE id = ?")
          .bind(providers_json)
          .bind(id)
          .execute(&self.pool)
          .await
          .map_err(|e| { error!("Error updating agent providers: {}", e); crate::error::TokenError::Generic })?;
      }
    }

    let remaining_providers = self.get_agent_details(id).await?;

    let remaining_providers = remaining_providers.map(|agent| {
      agent.providers.iter().map(|provider| {
        ProviderListItemBrief {
          id: provider.id.clone(),
          name: provider.name.clone(),
        }
      }).collect() 
    });

    match remaining_providers {
      Some(providers) => Ok(providers),
      None => Err(crate::error::TokenError::Generic ),
    }
  }

  /// Delete an agent
  ///
  /// # Arguments
  ///
  /// * `id` - Agent ID to delete (string format: agent_<uuid>)
  ///
  /// # Returns
  ///
  /// True if agent was deleted, false if not found
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn delete_agent( &self, id: &str ) -> Result< bool >
  {
    let result = sqlx::query( "DELETE FROM agents WHERE id = ?" )
      .bind( id )
      .execute( &self.pool )
      .await
      .map_err( |e| { error!( "Error deleting agent: {}", e ); crate::error::TokenError::Generic } )?;

    Ok( result.rows_affected() > 0 )
  }

  /// Get agent owner ID
  ///
  /// # Arguments
  ///
  /// * `id` - Agent ID (string format: agent_<uuid>)
  ///
  /// # Returns
  ///
  /// Owner ID or None if agent not found
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn get_agent_owner( &self, id: &str ) -> Result< Option< String > >
  {
    let owner: Option< String > = sqlx::query_scalar( "SELECT user_id FROM agents WHERE id = ?" )
      .bind( id )
      .fetch_optional( &self.pool )
      .await
      .map_err( |e| { error!( "Error getting agent owner: {}", e ); crate::error::TokenError::Generic } )?;

    Ok( owner )
  }

  /// Get all tokens for an agent
  ///
  /// # Arguments
  ///
  /// * `agent_id` - Agent ID (string format: agent_<uuid>)
  /// * `user_filter` - Optional user ID to filter tokens (None for all tokens)
  ///
  /// # Returns
  ///
  /// Vector of tokens for the agent
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn get_agent_tokens( &self, agent_id: &str, user_filter: Option< &str > ) -> Result< Vec< AgentTokenItem > >
  {
    let rows = if let Some( user_id ) = user_filter
    {
      // Filter by user
      sqlx::query(
        r#"
        SELECT id, user_id, provider, name, created_at, last_used_at, is_active
        FROM api_tokens
        WHERE agent_id = ? AND user_id = ?
        ORDER BY created_at DESC
        "#
      )
      .bind( agent_id )
      .bind( user_id )
      .fetch_all( &self.pool )
      .await
      .map_err( |e| { error!( "Error getting agent tokens: {}", e ); crate::error::TokenError::Generic } )?
    }
    else
    {
      // Return all tokens for agent
      sqlx::query(
        r#"
        SELECT id, user_id, provider, name, created_at, last_used_at, is_active
        FROM api_tokens
        WHERE agent_id = ?
        ORDER BY created_at DESC
        "#
      )
      .bind( agent_id )
      .fetch_all( &self.pool )
      .await
      .map_err( |e| { error!( "Error getting agent tokens: {}", e ); crate::error::TokenError::Generic } )?
    };

    let tokens = rows.iter().map( |row| AgentTokenItem {
      id: row.get( "id" ),
      user_id: row.get( "user_id" ),
      provider: row.get( "provider" ),
      name: row.get( "name" ),
      created_at: row.get( "created_at" ),
      last_used_at: row.get( "last_used_at" ),
      is_active: row.get( "is_active" ),
    } ).collect();

    Ok( tokens )
  }

  

  /// Get database pool for test verification
  ///
  /// **Warning:** Test-only method for accessing internal state
  #[ must_use ]
  pub fn pool( &self ) -> &SqlitePool
  {
    &self.pool
  }

  /// Convert a database row to an Agent struct
  fn row_to_agent( row: &sqlx::sqlite::SqliteRow ) -> Agent
  {
    let providers_json: Option< String > = row.get( "providers" );
    let providers = providers_json
      .as_ref()
      .and_then( |json| serde_json::from_str( json ).ok() )
      .unwrap_or_else( Vec::new );

    let tags_json: Option< String > = row.get( "tags" );
    let tags = tags_json
      .as_ref()
      .and_then( |json| serde_json::from_str( json ).ok() );

    let budget: f64 = row.get( "budget" );
    let spent: f64 = row.get( "spent" );
    let remaining: f64 = row.get( "remaining" );
    let percent_used = if budget > 0.0 { (spent / budget) * 100.0 } else { 0.0 };

    let ts = row.get( "created_at" );
    let dt = &DateTime::from_timestamp(ts, 0).unwrap_or_default();
    let created_at = dt.to_rfc3339();

    let ts = row.get( "updated_at" );
    let dt = &DateTime::from_timestamp(ts, 0).unwrap_or_default();
    let updated_at = dt.to_rfc3339();

    Agent {
      id: row.get( "id" ),
      name: row.get( "name" ),
      budget,
      providers,
      description: row.get( "description" ),
      tags,
      user_id: row.get( "user_id" ),
      project_id: row.get( "project_id" ),
      ic_token: None, // IC tokens are loaded separately if needed
      status: row.get( "status" ),
      created_at,
      updated_at,
      percent_used,
      spent,
      remaining,
    }
  }   
}
