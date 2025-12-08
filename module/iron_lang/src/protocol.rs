//! Protocol message type definitions for IronLang.
//!
//! This module defines all message types used for communication between AI agents
//! and data sources. Messages are serialized as NDJSON over STDIN/STDOUT.

use serde::{ Deserialize, Serialize };
use uuid::Uuid;
use std::collections::HashMap;

// =============================================================================
// Core Message Envelope
// =============================================================================

/// Top-level message envelope for all IronLang protocol messages.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
#[ serde( tag = "type" ) ]
pub enum IronMessage
{
  /// Request to read data from a source
  #[ serde( rename = "READ" ) ]
  Read( ReadMessage ),

  /// Request to write data to a destination
  #[ serde( rename = "WRITE" ) ]
  Write( WriteMessage ),

  /// Query metadata or filter data
  #[ serde( rename = "QUERY" ) ]
  Query( QueryMessage ),

  /// Request schema information
  #[ serde( rename = "SCHEMA" ) ]
  Schema( SchemaMessage ),

  /// Authenticate an agent
  #[ serde( rename = "AUTH" ) ]
  Auth( AuthMessage ),

  /// Acknowledge successful operation
  #[ serde( rename = "ACK" ) ]
  Ack( AckMessage ),

  /// Report operation failure
  #[ serde( rename = "ERROR" ) ]
  Error( ErrorMessage ),

  /// Diagnostic logging message
  #[ serde( rename = "LOG" ) ]
  Log( LogMessage ),

  /// Performance and usage metrics
  #[ serde( rename = "METRICS" ) ]
  Metrics( MetricsMessage ),
}

impl IronMessage
{
  /// Get the type name of this message.
  pub fn message_type( &self ) -> &'static str
  {
    match self
    {
      IronMessage::Read( _ ) => "READ",
      IronMessage::Write( _ ) => "WRITE",
      IronMessage::Query( _ ) => "QUERY",
      IronMessage::Schema( _ ) => "SCHEMA",
      IronMessage::Auth( _ ) => "AUTH",
      IronMessage::Ack( _ ) => "ACK",
      IronMessage::Error( _ ) => "ERROR",
      IronMessage::Log( _ ) => "LOG",
      IronMessage::Metrics( _ ) => "METRICS",
    }
  }

  /// Check if this is a request message.
  pub fn is_request( &self ) -> bool
  {
    matches!
    (
      self,
      IronMessage::Read( _ )
        | IronMessage::Write( _ )
        | IronMessage::Query( _ )
        | IronMessage::Schema( _ )
        | IronMessage::Auth( _ )
    )
  }

  /// Check if this is a response message.
  pub fn is_response( &self ) -> bool
  {
    matches!( self, IronMessage::Ack( _ ) | IronMessage::Error( _ ) )
  }
}

// =============================================================================
// READ Messages
// =============================================================================

/// Request to read data from a source.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct ReadMessage
{
  /// Unique identifier for this request
  pub request_id : Uuid,
  /// Name of the data source
  pub source : String,
  /// Type of read operation to perform
  pub operation : ReadOperation,
  /// Optional configuration
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub options : Option< ReadOptions >,
}

/// Type of read operation to execute.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
#[ serde( tag = "type" ) ]
pub enum ReadOperation
{
  /// Execute SQL query
  #[ serde( rename = "sql" ) ]
  Sql( SqlQuery ),
  /// Read file
  #[ serde( rename = "file" ) ]
  File( FileRead ),
  /// Make HTTP request
  #[ serde( rename = "http" ) ]
  Http( HttpRequest ),
  /// Get from cache
  #[ serde( rename = "cache" ) ]
  Cache( CacheGet ),
  /// Get object from storage
  #[ serde( rename = "object" ) ]
  Object( ObjectGet ),
}

/// SQL query operation.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct SqlQuery
{
  /// SQL query string
  pub query : String,
  /// Optional parameters
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub parameters : Option< Vec< SqlParameter > >,
}

/// SQL parameter.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
#[ serde( tag = "type", content = "value" ) ]
pub enum SqlParameter
{
  /// String value
  #[ serde( rename = "string" ) ]
  String( String ),
  /// Integer value
  #[ serde( rename = "int" ) ]
  Int( i64 ),
  /// Float value
  #[ serde( rename = "float" ) ]
  Float( f64 ),
  /// Boolean value
  #[ serde( rename = "bool" ) ]
  Bool( bool ),
  /// NULL value
  #[ serde( rename = "null" ) ]
  Null,
}

/// File read operation.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct FileRead
{
  /// Path to file
  pub path : String,
  /// Format of file content
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub format : Option< String >,
  /// Byte range to read
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub range : Option< ByteRange >,
}

/// Byte range.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct ByteRange
{
  /// Start offset (inclusive)
  pub start : u64,
  /// End offset (exclusive)
  pub end : u64,
}

/// HTTP request operation.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct HttpRequest
{
  /// HTTP method
  pub method : String,
  /// URL path
  pub path : String,
  /// Query parameters
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub query_params : Option< HashMap< String, String > >,
  /// Request headers
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub headers : Option< HashMap< String, String > >,
  /// Request body
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub body : Option< String >,
}

/// Cache get operation.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct CacheGet
{
  /// Cache key
  pub key : String,
}

/// Object storage get operation.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct ObjectGet
{
  /// Object key/path
  pub key : String,
  /// Bucket name
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub bucket : Option< String >,
  /// Byte range to download
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub range : Option< ByteRange >,
}

/// Options for read operations.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct ReadOptions
{
  /// Timeout in milliseconds
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub timeout_ms : Option< u64 >,
  /// Maximum retries
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub max_retries : Option< u32 >,
}

// =============================================================================
// WRITE Messages
// =============================================================================

/// Request to write data to a destination.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct WriteMessage
{
  /// Unique identifier
  pub request_id : Uuid,
  /// Name of the destination
  pub destination : String,
  /// Type of write operation
  pub operation : WriteOperation,
  /// Optional configuration
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub options : Option< WriteOptions >,
}

/// Type of write operation.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
#[ serde( tag = "type" ) ]
pub enum WriteOperation
{
  /// Execute SQL statement
  #[ serde( rename = "sql" ) ]
  Sql( SqlStatement ),
  /// Write file
  #[ serde( rename = "file" ) ]
  File( FileWrite ),
  /// Make HTTP request
  #[ serde( rename = "http" ) ]
  Http( HttpWrite ),
  /// Set in cache
  #[ serde( rename = "cache" ) ]
  Cache( CacheSet ),
  /// Put object to storage
  #[ serde( rename = "object" ) ]
  Object( ObjectPut ),
}

/// SQL statement.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct SqlStatement
{
  /// SQL statement
  pub statement : String,
  /// Optional parameters
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub parameters : Option< Vec< SqlParameter > >,
}

/// File write operation.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct FileWrite
{
  /// Path to file
  pub path : String,
  /// File content
  pub content : String,
  /// Format
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub format : Option< String >,
  /// Create parent directories
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub create_dirs : Option< bool >,
}

/// HTTP write operation.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct HttpWrite
{
  /// HTTP method
  pub method : String,
  /// URL path
  pub path : String,
  /// Query parameters
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub query_params : Option< HashMap< String, String > >,
  /// Request headers
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub headers : Option< HashMap< String, String > >,
  /// Request body
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub body : Option< String >,
}

/// Cache set operation.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct CacheSet
{
  /// Cache key
  pub key : String,
  /// Value to store
  pub value : String,
  /// Time-to-live in seconds
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub ttl_seconds : Option< u64 >,
}

/// Object storage put operation.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct ObjectPut
{
  /// Object key/path
  pub key : String,
  /// Object content
  pub content : String,
  /// Bucket name
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub bucket : Option< String >,
  /// Content type
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub content_type : Option< String >,
  /// Object metadata
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub metadata : Option< HashMap< String, String > >,
}

/// Options for write operations.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct WriteOptions
{
  /// Timeout in milliseconds
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub timeout_ms : Option< u64 >,
  /// Maximum retries
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub max_retries : Option< u32 >,
  /// Perform write atomically
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub atomic : Option< bool >,
}

// =============================================================================
// QUERY Messages
// =============================================================================

/// Request to query metadata.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct QueryMessage
{
  /// Unique identifier
  pub request_id : Uuid,
  /// Name of the source
  pub source : String,
  /// Type of query
  pub query_type : QueryType,
  /// Optional filters
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub filters : Option< HashMap< String, String > >,
}

/// Type of metadata query.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
#[ serde( tag = "type" ) ]
pub enum QueryType
{
  /// List tables
  #[ serde( rename = "list_tables" ) ]
  ListTables,
  /// List files
  #[ serde( rename = "list_files" ) ]
  ListFiles { path : String },
  /// List keys
  #[ serde( rename = "list_keys" ) ]
  ListKeys { pattern : String },
  /// List objects
  #[ serde( rename = "list_objects" ) ]
  ListObjects
  {
    bucket : String,
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    prefix : Option< String >,
  },
  /// Check existence
  #[ serde( rename = "exists" ) ]
  Exists { resource : String },
}

// =============================================================================
// SCHEMA Messages
// =============================================================================

/// Request schema information.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct SchemaMessage
{
  /// Unique identifier
  pub request_id : Uuid,
  /// Name of the source
  pub source : String,
  /// Optional resource identifier
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub resource : Option< String >,
}

/// Schema definition.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct Schema
{
  /// Schema format
  pub format : String,
  /// Schema content
  pub content : serde_json::Value,
}

// =============================================================================
// AUTH Messages
// =============================================================================

/// Authentication request.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct AuthMessage
{
  /// Unique identifier
  pub request_id : Uuid,
  /// Agent identifier
  pub agent_id : String,
  /// Credentials
  pub credentials : Credentials,
}

/// Supported credential types.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
#[ serde( tag = "type" ) ]
pub enum Credentials
{
  /// API key
  #[ serde( rename = "api_key" ) ]
  ApiKey { key : String },
  /// JWT token
  #[ serde( rename = "token" ) ]
  Token { token : String },
  /// Certificate
  #[ serde( rename = "certificate" ) ]
  Certificate { certificate : String },
  /// Password
  #[ serde( rename = "password" ) ]
  Password { username : String, password : String },
}

/// Session information.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct AgentSession
{
  /// Session ID
  pub session_id : Uuid,
  /// Agent ID
  pub agent_id : String,
  /// Session token
  pub token : String,
  /// Expiration timestamp
  pub expires_at : i64,
  /// Granted permissions
  pub permissions : Vec< Permission >,
}

/// Permission granted to agent.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct Permission
{
  /// Source name
  pub source : String,
  /// Allowed operations
  pub operations : Vec< String >,
}

// =============================================================================
// ACK Messages
// =============================================================================

/// Acknowledgment of successful operation.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct AckMessage
{
  /// Request ID this ACK responds to
  pub request_id : Uuid,
  /// Operation status
  pub status : Status,
  /// Response data
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub data : Option< serde_json::Value >,
  /// Optional metadata
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub metadata : Option< HashMap< String, String > >,
}

/// Operation status.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
#[ serde( rename_all = "UPPERCASE" ) ]
pub enum Status
{
  /// Operation completed successfully
  Success,
  /// Operation partially succeeded
  Partial,
  /// Operation completed with warnings
  Warning,
}

// =============================================================================
// ERROR Messages
// =============================================================================

/// Error response.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct ErrorMessage
{
  /// Request ID
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub request_id : Option< Uuid >,
  /// Error code
  pub error_code : String,
  /// Human-readable message
  pub message : String,
  /// Error severity
  pub severity : ErrorSeverity,
  /// Optional details
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub details : Option< String >,
  /// Timestamp
  pub timestamp : i64,
}

/// Error severity levels.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
#[ serde( rename_all = "UPPERCASE" ) ]
pub enum ErrorSeverity
{
  /// Informational
  Info,
  /// Warning
  Warning,
  /// Error
  Error,
  /// Fatal
  Fatal,
}

impl ErrorMessage
{
  /// Create a new error message.
  pub fn new( error_code : String, message : String ) -> Self
  {
    Self
    {
      request_id : None,
      error_code,
      message,
      severity : ErrorSeverity::Error,
      details : None,
      timestamp : chrono::Utc::now().timestamp(),
    }
  }

  /// Set request ID.
  pub fn with_request_id( mut self, request_id : Uuid ) -> Self
  {
    self.request_id = Some( request_id );
    self
  }

  /// Set severity.
  pub fn with_severity( mut self, severity : ErrorSeverity ) -> Self
  {
    self.severity = severity;
    self
  }

  /// Add details.
  pub fn with_details( mut self, details : String ) -> Self
  {
    self.details = Some( details );
    self
  }
}

// =============================================================================
// LOG Messages
// =============================================================================

/// Diagnostic log message.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct LogMessage
{
  /// Log level
  pub level : LogLevel,
  /// Log message content
  pub message : String,
  /// Component that emitted the log
  pub component : String,
  /// Timestamp
  pub timestamp : i64,
  /// Optional context fields
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub context : Option< serde_json::Value >,
}

/// Log severity levels.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
#[ serde( rename_all = "UPPERCASE" ) ]
pub enum LogLevel
{
  /// Trace
  Trace,
  /// Debug
  Debug,
  /// Info
  Info,
  /// Warn
  Warn,
  /// Error
  Error,
}

impl LogMessage
{
  /// Create a new log message.
  pub fn new( level : LogLevel, component : String, message : String ) -> Self
  {
    Self
    {
      level,
      message,
      component,
      timestamp : chrono::Utc::now().timestamp(),
      context : None,
    }
  }

  /// Add context.
  pub fn with_context( mut self, context : serde_json::Value ) -> Self
  {
    self.context = Some( context );
    self
  }
}

// =============================================================================
// METRICS Messages
// =============================================================================

/// Performance and usage metrics.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct MetricsMessage
{
  /// Metric name
  pub name : String,
  /// Metric value
  pub value : MetricValue,
  /// Metric type
  pub metric_type : MetricType,
  /// Component that emitted the metric
  pub component : String,
  /// Timestamp
  pub timestamp : i64,
  /// Optional tags
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub tags : Option< HashMap< String, String > >,
}

/// Metric value types.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
#[ serde( tag = "type", content = "value" ) ]
pub enum MetricValue
{
  /// Counter value
  #[ serde( rename = "counter" ) ]
  Counter( u64 ),
  /// Gauge value
  #[ serde( rename = "gauge" ) ]
  Gauge( f64 ),
  /// Histogram value
  #[ serde( rename = "histogram" ) ]
  Histogram( f64 ),
  /// Summary value
  #[ serde( rename = "summary" ) ]
  Summary( f64 ),
}

/// Metric types.
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
#[ serde( rename_all = "lowercase" ) ]
pub enum MetricType
{
  /// Counter metric
  Counter,
  /// Gauge metric
  Gauge,
  /// Histogram metric
  Histogram,
  /// Summary metric
  Summary,
}

impl MetricsMessage
{
  /// Create a counter metric.
  pub fn counter( name : String, component : String, value : u64 ) -> Self
  {
    Self
    {
      name,
      value : MetricValue::Counter( value ),
      metric_type : MetricType::Counter,
      component,
      timestamp : chrono::Utc::now().timestamp(),
      tags : None,
    }
  }

  /// Create a gauge metric.
  pub fn gauge( name : String, component : String, value : f64 ) -> Self
  {
    Self
    {
      name,
      value : MetricValue::Gauge( value ),
      metric_type : MetricType::Gauge,
      component,
      timestamp : chrono::Utc::now().timestamp(),
      tags : None,
    }
  }

  /// Add tags.
  pub fn with_tags( mut self, tags : HashMap< String, String > ) -> Self
  {
    self.tags = Some( tags );
    self
  }
}

// =============================================================================
// Utilities
// =============================================================================

/// Generate a new request ID.
pub fn new_request_id() -> Uuid
{
  Uuid::new_v4()
}

/// Get current timestamp as Unix epoch seconds.
pub fn current_timestamp() -> i64
{
  chrono::Utc::now().timestamp()
}

/// Get current timestamp as Unix epoch milliseconds.
pub fn current_timestamp_ms() -> i64
{
  chrono::Utc::now().timestamp_millis()
}
