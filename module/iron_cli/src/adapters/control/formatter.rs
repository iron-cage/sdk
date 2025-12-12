//! Output formatting for CLI responses
//!
//! Provides table, JSON, and YAML formatters for API responses.
//!
//! ## Table Formatting
//!
//! - **Lists**: Display as aligned tables with headers
//! - **Single Objects**: Display as key-value pairs
//! - **Nested Data**: Flatten or display as JSON strings
//!
//! ## Usage
//!
//! ```rust,no_run
//! use serde_json::json;
//! use iron_cli::adapters::control::format_output;
//!
//! let response = json!({"id": "agent_123", "name": "test-agent"});
//! let output = format_output(&response, "table").unwrap();
//! ```

use serde_json::Value;

/// Format API response according to requested format
///
/// ## Supported Formats
///
/// - `json`: Pretty-printed JSON
/// - `yaml`: YAML format
/// - `table`: ASCII table (default)
///
/// ## Examples
///
/// ```rust,no_run
/// use serde_json::json;
/// use iron_cli::adapters::control::format_output;
///
/// // List response
/// let response = json!([
///   {"id": "agent_1", "name": "Agent One", "budget": 1000},
///   {"id": "agent_2", "name": "Agent Two", "budget": 2000}
/// ]);
/// let table = format_output(&response, "table").unwrap();
/// // Output:
/// // ID       | NAME       | BUDGET
/// // ---------|------------|-------
/// // agent_1  | Agent One  | 1000
/// // agent_2  | Agent Two  | 2000
///
/// // Single object response
/// let response = json!({"id": "agent_1", "name": "Agent One", "budget": 1000});
/// let table = format_output(&response, "table").unwrap();
/// // Output:
/// // ID     : agent_1
/// // NAME   : Agent One
/// // BUDGET : 1000
/// ```
pub fn format_output(
  response: &Value,
  format: &str,
) -> Result< String, String >
{
  match format
  {
    "json" => format_json( response ),
    "yaml" => format_yaml( response ),
    "table" => format_table( response ),
    _ => format_table( response ),
  }
}

/// Format response as pretty-printed JSON
pub fn format_json( response: &Value ) -> Result< String, String >
{
  serde_json::to_string_pretty( response )
    .map_err( |e| format!( "JSON formatting failed: {}", e ) )
}

/// Format response as YAML
pub fn format_yaml( response: &Value ) -> Result< String, String >
{
  serde_yaml::to_string( response )
    .map_err( |e| format!( "YAML formatting failed: {}", e ) )
}

/// Format response as ASCII table
pub fn format_table( response: &Value ) -> Result< String, String >
{
  match response
  {
    Value::Array( items ) =>
    {
      if items.is_empty()
      {
        return Ok( "No results found.".to_string() );
      }
      format_list_table( items )
    }
    Value::Object( _ ) =>
    {
      format_object_table( response )
    }
    _ =>
    {
      // For simple values, just display as-is
      Ok( response.to_string() )
    }
  }
}

/// Format array of objects as table with rows
pub fn format_list_table( items: &[ Value ] ) -> Result< String, String >
{
  if items.is_empty()
  {
    return Ok( "No results found.".to_string() );
  }

  // Extract column names from first object
  let columns = extract_columns( &items[ 0 ] );

  if columns.is_empty()
  {
    // Fallback to JSON if no columns
    return format_json( &Value::Array( items.to_vec() ) );
  }

  // Calculate column widths
  let widths = calculate_column_widths( &columns, items );

  // Build table
  let mut output = String::new();

  // Header row
  output.push_str( &format_header_row( &columns, &widths ) );
  output.push( '\n' );

  // Separator row
  output.push_str( &format_separator_row( &widths ) );
  output.push( '\n' );

  // Data rows
  for item in items
  {
    output.push_str( &format_data_row( item, &columns, &widths ) );
    output.push( '\n' );
  }

  Ok( output )
}

/// Format single object as key-value pairs
pub fn format_object_table( obj: &Value ) -> Result< String, String >
{
  let obj_map = match obj.as_object()
  {
    Some( map ) => map,
    None => return format_json( obj ),
  };

  if obj_map.is_empty()
  {
    return Ok( "Empty object.".to_string() );
  }

  // Calculate max key width for alignment
  let max_key_width = obj_map
    .keys()
    .map( |k| k.len() )
    .max()
    .unwrap_or( 0 );

  let mut output = String::new();

  for ( key, value ) in obj_map
  {
    let key_upper = key.to_uppercase();
    let value_str = format_value_for_display( value );

    output.push_str( &format!(
      "{:width$} : {}\n",
      key_upper,
      value_str,
      width = max_key_width
    ) );
  }

  Ok( output.trim_end().to_string() )
}

/// Extract column names from object (top-level keys only)
fn extract_columns( obj: &Value ) -> Vec< String >
{
  match obj.as_object()
  {
    Some( map ) => map.keys().cloned().collect(),
    None => vec![],
  }
}

/// Calculate column widths based on header and data
pub fn calculate_column_widths(
  columns: &[ String ],
  items: &[ Value ],
) -> Vec< usize >
{
  columns
    .iter()
    .map( |col|
    {
      // Start with header width
      let mut max_width = col.len();

      // Check all data values
      for item in items
      {
        if let Some( value ) = item.get( col )
        {
          let value_str = format_value_for_display( value );
          max_width = max_width.max( value_str.len() );
        }
      }

      // Limit max width to 50 characters
      max_width.min( 50 )
    } )
    .collect()
}

/// Format header row
fn format_header_row(
  columns: &[ String ],
  widths: &[ usize ],
) -> String
{
  columns
    .iter()
    .zip( widths )
    .map( |( col, width )|
    {
      let col_upper = col.to_uppercase();
      format!( "{:width$}", col_upper, width = width )
    } )
    .collect::< Vec< _ > >()
    .join( " | " )
}

/// Format separator row
fn format_separator_row( widths: &[ usize ] ) -> String
{
  widths
    .iter()
    .map( |width| "-".repeat( *width ) )
    .collect::< Vec< _ > >()
    .join( "-|-" )
}

/// Format data row
pub fn format_data_row(
  item: &Value,
  columns: &[ String ],
  widths: &[ usize ],
) -> String
{
  columns
    .iter()
    .zip( widths )
    .map( |( col, width )|
    {
      let value = item.get( col ).unwrap_or( &Value::Null );
      let value_str = format_value_for_display( value );

      // Truncate if too long (handle UTF-8 properly)
      let display_str = if value_str.chars().count() > *width
      {
        if *width < 4
        {
          // Width too narrow for "..." suffix
          value_str.chars().take( *width ).collect()
        }
        else
        {
          // Safe UTF-8 truncation using chars
          let truncated: String = value_str.chars().take( *width - 3 ).collect();
          format!( "{}...", truncated )
        }
      }
      else
      {
        value_str
      };

      format!( "{:width$}", display_str, width = width )
    } )
    .collect::< Vec< _ > >()
    .join( " | " )
}

/// Format value for display in table
pub fn format_value_for_display( value: &Value ) -> String
{
  match value
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
      else
      {
        // Display array as comma-separated values for primitive types
        if arr.iter().all( |v| v.is_string() || v.is_number() || v.is_boolean() )
        {
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
          // For complex arrays, show count
          format!( "[{} items]", arr.len() )
        }
      }
    }
    Value::Object( obj ) =>
    {
      if obj.is_empty()
      {
        "{}".to_string()
      }
      else
      {
        // For nested objects, show JSON representation (compact)
        serde_json::to_string( value ).unwrap_or_else( |_| "{...}".to_string() )
      }
    }
  }
}
