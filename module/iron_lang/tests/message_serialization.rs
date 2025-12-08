//! Integration tests for protocol message serialization.

use iron_lang::protocol::*;

#[ test ]
fn test_read_sql_message_roundtrip()
{
  let original = IronMessage::Read( ReadMessage
  {
    request_id : new_request_id(),
    source : "test_db".to_string(),
    operation : ReadOperation::Sql( SqlQuery
    {
      query : "SELECT * FROM users WHERE id = $1".to_string(),
      parameters : Some( vec![ SqlParameter::Int( 42 ) ] ),
    }),
    options : Some( ReadOptions
    {
      timeout_ms : Some( 5000 ),
      max_retries : Some( 3 ),
    }),
  });

  let json = serde_json::to_string( &original ).expect( "serialization failed" );
  let deserialized : IronMessage = serde_json::from_str( &json )
    .expect( "deserialization failed" );

  assert_eq!( original, deserialized );
  assert_eq!( deserialized.message_type(), "READ" );
  assert!( deserialized.is_request() );
}

#[ test ]
fn test_write_file_message_roundtrip()
{
  let original = IronMessage::Write( WriteMessage
  {
    request_id : new_request_id(),
    destination : "local_fs".to_string(),
    operation : WriteOperation::File( FileWrite
    {
      path : "/tmp/test.json".to_string(),
      content : r#"{"key":"value"}"#.to_string(),
      format : Some( "json".to_string() ),
      create_dirs : Some( true ),
    }),
    options : None,
  });

  let json = serde_json::to_string( &original ).unwrap();
  let deserialized : IronMessage = serde_json::from_str( &json ).unwrap();

  assert_eq!( original, deserialized );
}

#[ test ]
fn test_ack_message_roundtrip()
{
  let original = IronMessage::Ack( AckMessage
  {
    request_id : new_request_id(),
    status : Status::Success,
    data : Some( serde_json::json!
    ({
      "rows" : [ { "id" : 1, "name" : "Alice" } ],
      "count" : 1,
    })),
    metadata : None,
  });

  let json = serde_json::to_string( &original ).unwrap();
  let deserialized : IronMessage = serde_json::from_str( &json ).unwrap();

  assert_eq!( original, deserialized );
  assert!( deserialized.is_response() );
}

#[ test ]
fn test_error_message_roundtrip()
{
  let original = IronMessage::Error
  (
    ErrorMessage::new
    (
      "SQL_ERROR".to_string(),
      "Table 'users' does not exist".to_string(),
    )
    .with_severity( ErrorSeverity::Error )
    .with_details( "Database: production\nQuery: SELECT * FROM users".to_string() )
  );

  let json = serde_json::to_string( &original ).unwrap();
  let deserialized : IronMessage = serde_json::from_str( &json ).unwrap();

  assert_eq!( original, deserialized );
}

#[ test ]
fn test_log_message_roundtrip()
{
  let original = IronMessage::Log
  (
    LogMessage::new
    (
      LogLevel::Info,
      "iron_connector_sql".to_string(),
      "Connection pool initialized".to_string(),
    )
    .with_context( serde_json::json!({ "pool_size" : 10 }) )
  );

  let json = serde_json::to_string( &original ).unwrap();
  let deserialized : IronMessage = serde_json::from_str( &json ).unwrap();

  assert_eq!( original, deserialized );
}

#[ test ]
fn test_ndjson_stream()
{
  let messages = vec!
  [
    IronMessage::Read( ReadMessage
    {
      request_id : new_request_id(),
      source : "db1".to_string(),
      operation : ReadOperation::Sql( SqlQuery
      {
        query : "SELECT 1".to_string(),
        parameters : None,
      }),
      options : None,
    }),
    IronMessage::Log( LogMessage::new
    (
      LogLevel::Debug,
      "runtime".to_string(),
      "Processing request".to_string(),
    )),
  ];

  let ndjson : String = messages
    .iter()
    .map( |msg| serde_json::to_string( msg ).unwrap() )
    .collect::< Vec< _ > >()
    .join( "\n" );

  let deserialized : Vec< IronMessage > = ndjson
    .lines()
    .map( |line| serde_json::from_str( line ).unwrap() )
    .collect();

  assert_eq!( messages.len(), deserialized.len() );
  assert_eq!( messages, deserialized );
}
