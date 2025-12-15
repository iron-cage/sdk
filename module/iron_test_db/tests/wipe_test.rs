use iron_test_db::*;
use std::collections::HashMap;

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
    .expect( "LOUD FAILURE: Should sort successfully" );

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
    .expect( "LOUD FAILURE: Should sort successfully" );

  // usage -> tokens -> projects -> users (deletion order)
  assert_eq!( sorted[ 0 ], "usage" );
  assert_eq!( sorted[ 3 ], "users" );

  // Verify tokens comes before projects and users
  let tokens_idx = sorted.iter().position( |t| t == "tokens" ).unwrap();
  let projects_idx = sorted.iter().position( |t| t == "projects" ).unwrap();
  let users_idx = sorted.iter().position( |t| t == "users" ).unwrap();

  assert!( tokens_idx < projects_idx, "Tokens table should be deleted before projects (child before parent)" );
  assert!( tokens_idx < users_idx, "Tokens table should be deleted before users (child before parent)" );
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
