//! tree_fmt-based formatter implementation
//!
//! Wrapper around tree_fmt library providing the same API as legacy Formatter.
//! Supports 4 output formats with improved features:
//! - Dynamic column widths (vs fixed 15-char)
//! - ANSI-aware alignment
//! - Professional table styling

use super::OutputFormat;
use crate::handlers::CliError;
use std::collections::HashMap;
use tree_fmt::{ RowBuilder, TableView };
use serde_json::Value;

/// Universal formatter using tree_fmt library
pub struct TreeFmtFormatter
{
  format: OutputFormat,
}

impl TreeFmtFormatter
{
  /// Create new formatter with specified format
  pub fn new( format: OutputFormat ) -> Self
  {
    Self { format }
  }

  /// Format a single item (key-value map)
  pub fn format_single( &self, data: &HashMap< String, String > ) -> String
  {
    match self.format
    {
      OutputFormat::Table => self.format_single_table( data ),
      OutputFormat::Expanded => self.format_single_expanded( data ),
      OutputFormat::Json => self.format_single_json( data ),
      OutputFormat::Yaml => self.format_single_yaml( data ),
    }
  }

  /// Format a list of items
  pub fn format_list( &self, items: &[ HashMap< String, String > ] ) -> String
  {
    match self.format
    {
      OutputFormat::Table => self.format_list_table( items ),
      OutputFormat::Expanded => self.format_list_expanded( items ),
      OutputFormat::Json => self.format_list_json( items ),
      OutputFormat::Yaml => self.format_list_yaml( items ),
    }
  }

  /// Format an error
  pub fn format_error( &self, error: &CliError ) -> String
  {
    match self.format
    {
      OutputFormat::Table | OutputFormat::Expanded => format!( "Error: {}", error ),
      OutputFormat::Json => self.format_error_json( error ),
      OutputFormat::Yaml => self.format_error_yaml( error ),
    }
  }

  /// Format a serde_json::Value (auto-detect array vs object)
  pub fn format_value( &self, value: &Value ) -> Result< String, String >
  {
    match value
    {
      Value::Array( items ) => self.format_value_array( items ),
      Value::Object( _ ) => self.format_value_object( value ),
      _ => Ok( value.to_string() ),
    }
  }

  /// Format a JSON object as single item
  fn format_value_object( &self, obj: &Value ) -> Result< String, String >
  {
    let obj_map = match obj.as_object()
    {
      Some( map ) => map,
      None => return match self.format
      {
        OutputFormat::Json => Ok( serde_json::to_string_pretty( obj ).unwrap_or_else( |_| "{}".to_string() ) ),
        OutputFormat::Yaml => Ok( serde_yaml::to_string( obj ).unwrap_or_else( |_| "{}".to_string() ) ),
        _ => Ok( obj.to_string() ),
      },
    };

    if obj_map.is_empty()
    {
      return Ok( "Empty object.".to_string() );
    }

    // Convert JSON object to HashMap<String, String>
    let data = convert_json_to_hashmap( obj_map );

    Ok( self.format_single( &data ) )
  }

  /// Format a JSON array as list
  fn format_value_array( &self, items: &[ Value ] ) -> Result< String, String >
  {
    if items.is_empty()
    {
      return Ok( "No results found.".to_string() );
    }

    // Convert JSON array to Vec<HashMap<String, String>>
    let data: Vec< HashMap< String, String > > = items
      .iter()
      .filter_map( |item| item.as_object().map( convert_json_to_hashmap ) )
      .collect();

    if data.is_empty()
    {
      // Array of non-objects, use JSON/YAML
      return match self.format
      {
        OutputFormat::Json => Ok( serde_json::to_string_pretty( items ).unwrap_or_else( |_| "[]".to_string() ) ),
        OutputFormat::Yaml => Ok( serde_yaml::to_string( items ).unwrap_or_else( |_| "[]".to_string() ) ),
        _ => Ok( format!( "[{} items]", items.len() ) ),
      };
    }

    Ok( self.format_list( &data ) )
  }

  // ============================================================================
  // Table format implementations
  // ============================================================================

  fn format_single_table( &self, data: &HashMap< String, String > ) -> String
  {
    if data.is_empty()
    {
      return String::new();
    }

    // Build table view: Key | Value
    let mut builder = RowBuilder::new( vec![ "Key".into(), "Value".into() ] );

    // Get keys in sorted order for consistent output
    let mut keys: Vec< _ > = data.keys().collect();
    keys.sort();

    for key in keys
    {
      if let Some( value ) = data.get( key.as_str() )
      {
        builder.add_row_mut( vec![ key.clone(), value.clone() ] );
      }
    }

    let view = builder.build_view();
    self.format_table_view( &view )
  }

  fn format_list_table( &self, items: &[ HashMap< String, String > ] ) -> String
  {
    if items.is_empty()
    {
      return "No items found".to_string();
    }

    // Get all unique keys across all items
    let mut all_keys = std::collections::HashSet::new();
    for item in items
    {
      for key in item.keys()
      {
        all_keys.insert( key.clone() );
      }
    }

    let mut keys: Vec< _ > = all_keys.into_iter().collect();
    keys.sort();

    // Build table view with dynamic columns
    let headers: Vec< String > = keys.iter().map( | k | k.to_string() ).collect();
    let mut builder = RowBuilder::new( headers );

    for item in items
    {
      let row: Vec< String > = keys
        .iter()
        .map( | k | item.get( k ).map( | s | s.to_string() ).unwrap_or_default() )
        .collect();
      builder.add_row_mut( row );
    }

    let view = builder.build_view();
    self.format_table_view( &view )
  }

  fn format_table_view( &self, view: &TableView ) -> String
  {
    use tree_fmt::{ TableFormatter, TableConfig, Format };

    let config = TableConfig::plain();
    let formatter = TableFormatter::with_config( config );

    Format::format( &formatter, view )
      .unwrap_or_else( | _ | "Format error".to_string() )
  }

  // ============================================================================
  // Expanded format implementations
  // ============================================================================

  fn format_single_expanded( &self, data: &HashMap< String, String > ) -> String
  {
    if data.is_empty()
    {
      return String::new();
    }

    let mut lines = Vec::new();

    // Get keys in sorted order
    let mut keys: Vec< _ > = data.keys().collect();
    keys.sort();

    for key in keys
    {
      if let Some( value ) = data.get( key.as_str() )
      {
        lines.push( format!( "{}: {}", key, value ) );
      }
    }

    lines.join( "\n" )
  }

  fn format_list_expanded( &self, items: &[ HashMap< String, String > ] ) -> String
  {
    if items.is_empty()
    {
      return "No items found".to_string();
    }

    let mut blocks = Vec::new();

    for ( i, item ) in items.iter().enumerate()
    {
      let mut lines = vec![ format!( "Item {}:", i + 1 ) ];

      // Get keys in sorted order
      let mut keys: Vec< _ > = item.keys().collect();
      keys.sort();

      for key in keys
      {
        if let Some( value ) = item.get( key.as_str() )
        {
          lines.push( format!( "  {}: {}", key, value ) );
        }
      }

      blocks.push( lines.join( "\n" ) );
    }

    blocks.join( "\n\n" )
  }

  // ============================================================================
  // JSON format implementations
  // ============================================================================

  fn format_single_json( &self, data: &HashMap< String, String > ) -> String
  {
    serde_json::to_string_pretty( data ).unwrap_or_else( | _ | "{}".to_string() )
  }

  fn format_list_json( &self, items: &[ HashMap< String, String > ] ) -> String
  {
    serde_json::to_string_pretty( items ).unwrap_or_else( | _ | "[]".to_string() )
  }

  fn format_error_json( &self, error: &CliError ) -> String
  {
    let error_msg = format!( "{}", error );
    let error_obj: HashMap< String, String > = [ ( "error".to_string(), error_msg ) ].iter().cloned().collect();
    serde_json::to_string_pretty( &error_obj ).unwrap_or_else( | _ | r#"{"error": "unknown"}"#.to_string() )
  }

  // ============================================================================
  // YAML format implementations
  // ============================================================================

  fn format_single_yaml( &self, data: &HashMap< String, String > ) -> String
  {
    serde_yaml::to_string( data ).unwrap_or_else( | _ | "{}".to_string() )
  }

  fn format_list_yaml( &self, items: &[ HashMap< String, String > ] ) -> String
  {
    serde_yaml::to_string( items ).unwrap_or_else( | _ | "[]".to_string() )
  }

  fn format_error_yaml( &self, error: &CliError ) -> String
  {
    let error_msg = format!( "{}", error );
    let error_obj: HashMap< String, String > = [ ( "error".to_string(), error_msg ) ].iter().cloned().collect();
    serde_yaml::to_string( &error_obj ).unwrap_or_else( | _ | "error: unknown".to_string() )
  }
}

// ============================================================================
// Helper functions for JSON conversion
// ============================================================================

/// Convert serde_json::Map to HashMap<String, String>
///
/// Handles nested structures by converting them to strings:
/// - Objects: Convert to JSON string
/// - Arrays: Convert to comma-separated list or JSON string
/// - Primitives: Convert to string representation
fn convert_json_to_hashmap( map: &serde_json::Map< String, Value > ) -> HashMap< String, String >
{
  map
    .iter()
    .map( |( key, value )|
    {
      let value_str = match value
      {
        Value::Null => "null".to_string(),
        Value::Bool( b ) => b.to_string(),
        Value::Number( n ) => n.to_string(),
        Value::String( s ) => s.clone(),
        Value::Array( arr ) =>
        {
          if arr.is_empty()
          {
            "[]".to_string()
          }
          else if arr.iter().all( |v| v.is_string() || v.is_number() || v.is_boolean() )
          {
            // Display array as comma-separated values for primitive types
            arr
              .iter()
              .map( |v| match v
              {
                Value::String( s ) => s.clone(),
                Value::Number( n ) => n.to_string(),
                Value::Bool( b ) => b.to_string(),
                _ => v.to_string(),
              } )
              .collect::< Vec< _ > >()
              .join( ", " )
          }
          else
          {
            // For complex arrays, show JSON
            serde_json::to_string( arr ).unwrap_or_else( |_| "[]".to_string() )
          }
        }
        Value::Object( _ ) =>
        {
          // For nested objects, show JSON
          serde_json::to_string( value ).unwrap_or_else( |_| "{}".to_string() )
        }
      };

      ( key.clone(), value_str )
    } )
    .collect()
}
