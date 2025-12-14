// Extracted from iron_types/src/ids.rs inline tests

use iron_types::*;

#[test]
fn agent_id_generate_has_correct_prefix()
{
  let id = AgentId::generate();
  assert!( id.as_str().starts_with( "agent_" ) );
  assert_eq!( id.as_str().len(), "agent_".len() + 36 ); // prefix + UUID
}

#[test]
fn agent_id_parse_valid()
{
  let id_str = "agent_550e8400-e29b-41d4-a716-446655440000";
  let id = AgentId::parse( id_str ).unwrap();
  assert_eq!( id.as_str(), id_str );
}

#[test]
fn agent_id_parse_invalid_prefix()
{
  let result = AgentId::parse( "wrong_550e8400-e29b-41d4-a716-446655440000" );
  assert!( matches!( result, Err( IdError::InvalidPrefix { .. } ) ) );
}

#[test]
fn agent_id_parse_missing_uuid()
{
  let result = AgentId::parse( "agent_" );
  assert!( matches!( result, Err( IdError::MissingUuid ) ) );
}

#[test]
fn agent_id_parse_invalid_uuid()
{
  let result = AgentId::parse( "agent_not-a-valid-uuid" );
  assert!( matches!( result, Err( IdError::InvalidUuid( .. ) ) ) );
}

#[test]
fn agent_id_parse_empty()
{
  let result = AgentId::parse( "" );
  assert!( matches!( result, Err( IdError::EmptyId ) ) );
}

#[test]
fn provider_id_generate_has_correct_prefix()
{
  let id = ProviderId::generate();
  assert!( id.as_str().starts_with( "ip_" ) );
}

#[test]
fn project_id_generate_has_correct_prefix()
{
  let id = ProjectId::generate();
  assert!( id.as_str().starts_with( "proj_" ) );
}

#[test]
fn api_token_id_generate_has_correct_prefix()
{
  let id = ApiTokenId::generate();
  assert!( id.as_str().starts_with( "at_" ) );
}

#[test]
fn budget_request_id_generate_has_correct_prefix()
{
  let id = BudgetRequestId::generate();
  assert!( id.as_str().starts_with( "breq_" ) );
}

#[test]
fn lease_id_generate_has_correct_prefix()
{
  let id = LeaseId::generate();
  assert!( id.as_str().starts_with( "lease_" ) );
}

#[test]
fn request_id_generate_has_correct_prefix()
{
  let id = RequestId::generate();
  assert!( id.as_str().starts_with( "req_" ) );
}

#[test]
fn ic_token_id_generate_has_correct_prefix()
{
  let id = IcTokenId::generate();
  assert!( id.as_str().starts_with( "ic_" ) );
}

#[test]
fn all_ids_serialize_to_string()
{
  let agent_id = AgentId::generate();
  let json = serde_json::to_string( &agent_id ).unwrap();
  assert!( json.contains( "agent_" ) );
}

#[test]
fn all_ids_deserialize_from_string()
{
  let id_str = r#""agent_550e8400-e29b-41d4-a716-446655440000""#;
  let id: AgentId = serde_json::from_str( id_str ).unwrap();
  assert_eq!( id.as_str(), "agent_550e8400-e29b-41d4-a716-446655440000" );
}

#[test]
fn uuid_validation_rejects_uppercase()
{
  let result = AgentId::parse( "agent_550E8400-E29B-41D4-A716-446655440000" );
  assert!( matches!( result, Err( IdError::InvalidUuid( .. ) ) ) );
}

#[test]
fn uuid_validation_rejects_wrong_length()
{
  let result = AgentId::parse( "agent_550e8400-e29b-41d4" );
  assert!( matches!( result, Err( IdError::InvalidUuid( .. ) ) ) );
}

#[test]
fn from_str_trait_works()
{
  use std::str::FromStr;

  let id_str = "agent_550e8400-e29b-41d4-a716-446655440000";
  let id = AgentId::from_str( id_str ).unwrap();
  assert_eq!( id.as_str(), id_str );
}

#[test]
fn from_str_trait_via_parse_method()
{
  // Ergonomic syntax enabled by FromStr
  let id: AgentId = "agent_550e8400-e29b-41d4-a716-446655440000".parse().unwrap();
  assert!( id.as_str().starts_with( "agent_" ) );
}

#[test]
fn test_fixture_generates_sequential_ids()
{
  let id1 = AgentId::test_fixture( 1 );
  let id2 = AgentId::test_fixture( 2 );
  let id3 = AgentId::test_fixture( 3 );

  assert_eq!( id1.as_str(), "agent_00000000-0000-0000-0000-000000000001" );
  assert_eq!( id2.as_str(), "agent_00000000-0000-0000-0000-000000000002" );
  assert_eq!( id3.as_str(), "agent_00000000-0000-0000-0000-000000000003" );
}

#[test]
fn test_fixture_ids_are_valid()
{
  let id = AgentId::test_fixture( 42 );

  // Can round-trip through parse
  let parsed = AgentId::parse( id.as_str() ).unwrap();
  assert_eq!( parsed.as_str(), id.as_str() );
}

#[test]
fn from_uuid_creates_valid_id()
{
  let uuid = uuid::Uuid::parse_str( "550e8400-e29b-41d4-a716-446655440000" ).unwrap();
  let id = AgentId::from_uuid( uuid );

  assert_eq!( id.as_str(), "agent_550e8400-e29b-41d4-a716-446655440000" );

  // Can round-trip
  let parsed = AgentId::parse( id.as_str() ).unwrap();
  assert_eq!( parsed.as_str(), id.as_str() );
}

#[test]
fn test_with_suffix_creates_invalid_id()
{
  // Create intentionally invalid ID for error testing
  let bad_id = AgentId::test_with_suffix( "not-a-valid-uuid" );

  assert_eq!( bad_id.as_str(), "agent_not-a-valid-uuid" );

  // Should fail validation
  let result = AgentId::parse( bad_id.as_str() );
  assert!( result.is_err() );
}

#[test]
fn all_id_types_have_test_fixtures()
{
  // Verify all ID types support test utilities
  let _ = AgentId::test_fixture( 1 );
  let _ = ProviderId::test_fixture( 1 );
  let _ = ProjectId::test_fixture( 1 );
  let _ = ApiTokenId::test_fixture( 1 );
  let _ = BudgetRequestId::test_fixture( 1 );
  let _ = LeaseId::test_fixture( 1 );
  let _ = RequestId::test_fixture( 1 );
  let _ = IcTokenId::test_fixture( 1 );
}

// parse_flexible tests - backward compatibility

#[test]
fn parse_flexible_accepts_current_underscore_format()
{
  let id_str = "agent_550e8400-e29b-41d4-a716-446655440000";
  let id = AgentId::parse_flexible( id_str ).unwrap();
  assert_eq!( id.as_str(), id_str );
}

#[test]
fn parse_flexible_accepts_legacy_hyphen_format()
{
  let legacy_id = "agent-550e8400-e29b-41d4-a716-446655440000";
  let id = AgentId::parse_flexible( legacy_id ).unwrap();

  // Normalized to underscore format
  assert_eq!( id.as_str(), "agent_550e8400-e29b-41d4-a716-446655440000" );
}

#[test]
fn parse_flexible_normalizes_legacy_to_current()
{
  let legacy_id = "agent-550e8400-e29b-41d4-a716-446655440000";
  let current_id = "agent_550e8400-e29b-41d4-a716-446655440000";

  let id1 = AgentId::parse_flexible( legacy_id ).unwrap();
  let id2 = AgentId::parse_flexible( current_id ).unwrap();

  // Both produce same normalized ID
  assert_eq!( id1.as_str(), id2.as_str() );
  assert_eq!( id1.as_str(), "agent_550e8400-e29b-41d4-a716-446655440000" );
}

#[test]
fn parse_flexible_works_for_all_entity_types()
{
  // Agent ID
  assert!( AgentId::parse_flexible( "agent-abc123def456ghi789jkl012mno345pq" ).is_err() );
  assert!( AgentId::parse_flexible( "agent_550e8400-e29b-41d4-a716-446655440000" ).is_ok() );
  assert!( AgentId::parse_flexible( "agent-550e8400-e29b-41d4-a716-446655440000" ).is_ok() );

  // Provider ID
  assert!( ProviderId::parse_flexible( "ip_550e8400-e29b-41d4-a716-446655440000" ).is_ok() );
  assert!( ProviderId::parse_flexible( "ip-550e8400-e29b-41d4-a716-446655440000" ).is_ok() );

  // Project ID
  assert!( ProjectId::parse_flexible( "proj_550e8400-e29b-41d4-a716-446655440000" ).is_ok() );
  assert!( ProjectId::parse_flexible( "proj-550e8400-e29b-41d4-a716-446655440000" ).is_ok() );

  // API Token ID
  assert!( ApiTokenId::parse_flexible( "at_550e8400-e29b-41d4-a716-446655440000" ).is_ok() );
  assert!( ApiTokenId::parse_flexible( "at-550e8400-e29b-41d4-a716-446655440000" ).is_ok() );
}

#[test]
fn parse_flexible_rejects_invalid_uuids_in_both_formats()
{
  // Invalid UUID in current format
  let result1 = AgentId::parse_flexible( "agent_not-a-valid-uuid" );
  assert!( matches!( result1, Err( IdError::InvalidUuid( .. ) ) ) );

  // Invalid UUID in legacy format
  let result2 = AgentId::parse_flexible( "agent-not-a-valid-uuid" );
  assert!( matches!( result2, Err( IdError::InvalidUuid( .. ) ) ) );
}

#[test]
fn parse_flexible_rejects_wrong_prefix()
{
  let result = AgentId::parse_flexible( "wrong_550e8400-e29b-41d4-a716-446655440000" );
  assert!( matches!( result, Err( IdError::InvalidPrefix { .. } ) ) );
}

#[test]
fn parse_flexible_rejects_empty_string()
{
  let result = AgentId::parse_flexible( "" );
  assert!( matches!( result, Err( IdError::EmptyId ) ) );
}

#[test]
fn parse_flexible_rejects_missing_uuid_current_format()
{
  let result = AgentId::parse_flexible( "agent_" );
  assert!( matches!( result, Err( IdError::MissingUuid ) ) );
}

#[test]
fn parse_flexible_rejects_missing_uuid_legacy_format()
{
  let result = AgentId::parse_flexible( "agent-" );
  assert!( matches!( result, Err( IdError::MissingUuid ) ) );
}

#[test]
fn parse_flexible_preserves_uuid_exactly()
{
  let uuid = "550e8400-e29b-41d4-a716-446655440000";

  // Legacy format input
  let legacy_input = format!( "agent-{}", uuid );
  let id = AgentId::parse_flexible( &legacy_input ).unwrap();

  // UUID part preserved exactly (not modified)
  assert!( id.as_str().ends_with( uuid ) );
}

#[test]
fn parse_flexible_roundtrip_with_parse()
{
  // Start with legacy format
  let legacy_id = "agent-550e8400-e29b-41d4-a716-446655440000";
  let id = AgentId::parse_flexible( legacy_id ).unwrap();

  // Normalized ID can be parsed with strict parse()
  let reparsed = AgentId::parse( id.as_str() ).unwrap();
  assert_eq!( reparsed.as_str(), id.as_str() );
}

#[test]
fn parse_flexible_ic_token_no_normalization_needed()
{
  // IC tokens already use underscore (no legacy format)
  let ic_id = "ic_550e8400-e29b-41d4-a716-446655440000";
  let id = IcTokenId::parse_flexible( ic_id ).unwrap();
  assert_eq!( id.as_str(), ic_id );

  // Hyphen format for IC tokens is NOT legacy (it never existed)
  let fake_legacy = "ic-550e8400-e29b-41d4-a716-446655440000";
  let id2 = IcTokenId::parse_flexible( fake_legacy ).unwrap();

  // Still normalizes for consistency
  assert_eq!( id2.as_str(), "ic_550e8400-e29b-41d4-a716-446655440000" );
}
