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
use tree_fmt::{ RowBuilder, TableView, Format };

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
    use tree_fmt::{ TableFormatter, TableConfig };

    let config = TableConfig::plain();
    let formatter = TableFormatter::with_config( config );

    formatter.format( view )
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

    // Build table view for expanded formatter
    let mut builder = RowBuilder::new( vec![ "Key".into(), "Value".into() ] );

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
    self.format_expanded_view( &view )
  }

  fn format_list_expanded( &self, items: &[ HashMap< String, String > ] ) -> String
  {
    if items.is_empty()
    {
      return "No items found".to_string();
    }

    // Format each item as a separate expanded block
    let blocks: Vec< String > = items
      .iter()
      .map( | item | self.format_single_expanded( item ) )
      .collect();

    blocks.join( "\n\n" )
  }

  fn format_expanded_view( &self, view: &TableView ) -> String
  {
    use tree_fmt::{ ExpandedFormatter, ExpandedConfig };

    let config = ExpandedConfig::postgres();
    let formatter = ExpandedFormatter::with_config( config );

    formatter.format( view )
      .unwrap_or_else( | _ | "Format error".to_string() )
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
