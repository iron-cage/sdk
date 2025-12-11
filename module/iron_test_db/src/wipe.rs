//! Automatic table wiping with dependency detection

use crate::error::{ Result, TestDbError };
use sqlx::SqlitePool;
use std::collections::{ HashMap, HashSet };

/// Discover table dependencies via foreign key introspection
///
/// Returns tables in reverse dependency order (safe for deletion).
/// Children are listed before parents.
pub async fn discover_table_dependencies( pool: &SqlitePool ) -> Result< Vec< String > >
{
  // Get all non-system tables
  let tables: Vec< String > = sqlx::query_scalar(
    "SELECT name FROM sqlite_master
     WHERE type = 'table'
     AND substr( name, 1, 1 ) != '_'
     AND name != 'sqlite_sequence'
     ORDER BY name"
  )
  .fetch_all( pool )
  .await?;

  if tables.is_empty()
  {
    return Ok( Vec::new() );
  }

  // Build dependency graph (table -> tables it depends on)
  let mut dependencies: HashMap< String, Vec< String > > = HashMap::new();

  for table in &tables
  {
    // Get foreign keys for this table
    let foreign_keys: Vec< String > = sqlx::query_scalar(
      "SELECT DISTINCT [table] FROM pragma_foreign_key_list( ? )"
    )
    .bind( table )
    .fetch_all( pool )
    .await?;

    dependencies.insert( table.clone(), foreign_keys );
  }

  // Topological sort (reverse order for deletion)
  topological_sort_reverse( &tables, &dependencies )
}

/// Topological sort in reverse order (children before parents)
fn topological_sort_reverse(
  tables: &[ String ],
  dependencies: &HashMap< String, Vec< String > >,
) -> Result< Vec< String > >
{
  let mut sorted = Vec::new();
  let mut visited = HashSet::new();
  let mut visiting = HashSet::new();

  for table in tables
  {
    if !visited.contains( table )
    {
      visit( table, dependencies, &mut visited, &mut visiting, &mut sorted )?;
    }
  }

  // Reverse for deletion order (children first)
  sorted.reverse();
  Ok( sorted )
}

fn visit(
  table: &str,
  dependencies: &HashMap< String, Vec< String > >,
  visited: &mut HashSet< String >,
  visiting: &mut HashSet< String >,
  sorted: &mut Vec< String >,
) -> Result< () >
{
  // Cycle detection
  if visiting.contains( table )
  {
    return Err( TestDbError::DependencyCycle(
      format!( "Cycle detected involving table: {}", table )
    ) );
  }

  if visited.contains( table )
  {
    return Ok( () );
  }

  visiting.insert( table.to_string() );

  // Visit dependencies first (parents)
  if let Some( deps ) = dependencies.get( table )
  {
    for dep in deps
    {
      visit( dep, dependencies, visited, visiting, sorted )?;
    }
  }

  visiting.remove( table );
  visited.insert( table.to_string() );
  sorted.push( table.to_string() );

  Ok( () )
}

/// Wipe all tables in dependency-safe order
///
/// Automatically detects foreign key relationships and deletes
/// in correct order (children before parents).
pub async fn wipe_all_tables( pool: &SqlitePool ) -> Result< () >
{
  let tables = discover_table_dependencies( pool ).await?;

  for table in tables
  {
    sqlx::query( &format!( "DELETE FROM {}", table ) )
      .execute( pool )
      .await?;
  }

  Ok( () )
}

#[ cfg( test ) ]
mod tests
{
  use super::*;

  #[ test ]
  fn test_topological_sort_simple()
  {
    let tables = vec![
      "parent".to_string(),
      "child".to_string(),
    ];

    let mut dependencies = HashMap::new();
    dependencies.insert( "parent".to_string(), vec![] );
    dependencies.insert( "child".to_string(), vec![ "parent".to_string() ] );

    let sorted = topological_sort_reverse( &tables, &dependencies )
      .expect( "Should sort successfully" );

    // Child should come before parent (for deletion)
    assert_eq!( sorted, vec![ "child", "parent" ] );
  }

  #[ test ]
  fn test_topological_sort_complex()
  {
    let tables = vec![
      "users".to_string(),
      "projects".to_string(),
      "tokens".to_string(),
      "usage".to_string(),
    ];

    let mut dependencies = HashMap::new();
    dependencies.insert( "users".to_string(), vec![] );
    dependencies.insert( "projects".to_string(), vec![ "users".to_string() ] );
    dependencies.insert( "tokens".to_string(), vec![ "projects".to_string(), "users".to_string() ] );
    dependencies.insert( "usage".to_string(), vec![ "tokens".to_string() ] );

    let sorted = topological_sort_reverse( &tables, &dependencies )
      .expect( "Should sort successfully" );

    // usage -> tokens -> projects -> users (deletion order)
    assert_eq!( sorted[ 0 ], "usage" );
    assert_eq!( sorted[ 3 ], "users" );

    // Verify tokens comes before projects and users
    let tokens_idx = sorted.iter().position( |t| t == "tokens" ).unwrap();
    let projects_idx = sorted.iter().position( |t| t == "projects" ).unwrap();
    let users_idx = sorted.iter().position( |t| t == "users" ).unwrap();

    assert!( tokens_idx < projects_idx );
    assert!( tokens_idx < users_idx );
  }

  #[ test ]
  fn test_cycle_detection()
  {
    let tables = vec![
      "a".to_string(),
      "b".to_string(),
    ];

    let mut dependencies = HashMap::new();
    dependencies.insert( "a".to_string(), vec![ "b".to_string() ] );
    dependencies.insert( "b".to_string(), vec![ "a".to_string() ] );

    let result = topological_sort_reverse( &tables, &dependencies );

    assert!( result.is_err() );
    let err = result.unwrap_err();
    assert!( matches!( err, TestDbError::DependencyCycle( _ ) ) );
  }
}
