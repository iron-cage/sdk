//! Configuration layer abstraction
//!
//! Implements the 5-layer configuration precedence system:
//! 1. Environment variables (highest priority)
//! 2. Project-specific config (`{workspace}/config/{module}.{env}.toml`)
//! 3. User config (`~/.config/iron/{module}.toml`)
//! 4. Workspace defaults (`{workspace}/config/{module}.default.toml`)
//! 5. Crate defaults (lowest priority)

use crate::error::{ ConfigError, Result };
use std::collections::HashMap;
use std::path::PathBuf;

/// Configuration value from a specific layer
#[ derive( Debug, Clone ) ]
pub struct ConfigValue
{
  /// The actual value
  pub value: toml::Value,
  /// Source layer for debugging
  pub source: String,
}

/// Trait for configuration layers
pub trait ConfigLayer
{
  /// Get configuration value by key path (e.g., "database.url")
  fn get( &self, key: &str ) -> Result< Option< ConfigValue > >;

  /// Get all configuration as a map
  fn get_all( &self ) -> Result< HashMap< String, ConfigValue > >;

  /// Layer name for debugging
  fn name( &self ) -> &str;

  /// Priority (higher = more important, 1-5)
  fn priority( &self ) -> u8;
}

/// Environment variable layer (priority 5 - highest)
#[ derive( Debug ) ]
pub struct EnvLayer
{
  /// Environment variable prefix (e.g., "IRON_TOKEN_MANAGER_")
  prefix: String,
  /// Cached environment variables
  vars: HashMap< String, String >,
}

impl EnvLayer
{
  /// Create new environment layer with prefix
  ///
  /// # Examples
  ///
  /// ```ignore
  /// let layer = EnvLayer::new("IRON_TOKEN_MANAGER_");
  /// ```
  pub fn new( prefix: impl Into< String > ) -> Self
  {
    let prefix = prefix.into();
    let vars = std::env::vars()
      .filter( |( k, _ )| k.starts_with( &prefix ) )
      .collect();

    Self { prefix, vars }
  }

  /// Convert environment variable name to config key
  ///
  /// Example: `IRON_TOKEN_MANAGER_DATABASE_URL` → `database.url`
  fn env_to_key( &self, env_var: &str ) -> String
  {
    env_var
      .strip_prefix( &self.prefix )
      .unwrap_or( env_var )
      .to_lowercase()
      .replace( '_', "." )
  }

  /// Convert config key to environment variable name
  ///
  /// Example: `database.url` → `IRON_TOKEN_MANAGER_DATABASE_URL`
  fn key_to_env( &self, key: &str ) -> String
  {
    format!( "{}{}", self.prefix, key.replace( '.', "_" ).to_uppercase() )
  }

  /// Parse environment variable value as TOML value
  ///
  /// Attempts to parse the value as boolean, integer, or string in that order.
  fn parse_value( value: &str ) -> toml::Value
  {
    // Try boolean
    if value.eq_ignore_ascii_case( "true" )
    {
      return toml::Value::Boolean( true );
    }
    if value.eq_ignore_ascii_case( "false" )
    {
      return toml::Value::Boolean( false );
    }

    // Try integer
    if let Ok( i ) = value.parse::< i64 >()
    {
      return toml::Value::Integer( i );
    }

    // Try float
    if let Ok( f ) = value.parse::< f64 >()
    {
      return toml::Value::Float( f );
    }

    // Default to string
    toml::Value::String( value.to_string() )
  }
}

impl ConfigLayer for EnvLayer
{
  fn get( &self, key: &str ) -> Result< Option< ConfigValue > >
  {
    let env_var = self.key_to_env( key );

    if let Some( value ) = self.vars.get( &env_var )
    {
      let toml_value = Self::parse_value( value );

      Ok( Some( ConfigValue
      {
        value: toml_value,
        source: format!( "env:{}", env_var ),
      } ) )
    }
    else
    {
      Ok( None )
    }
  }

  fn get_all( &self ) -> Result< HashMap< String, ConfigValue > >
  {
    let mut result = HashMap::new();

    for ( env_var, value ) in &self.vars
    {
      let key = self.env_to_key( env_var );
      let toml_value = Self::parse_value( value );

      result.insert( key, ConfigValue
      {
        value: toml_value,
        source: format!( "env:{}", env_var ),
      } );
    }

    Ok( result )
  }

  fn name( &self ) -> &str
  {
    "Environment Variables"
  }

  fn priority( &self ) -> u8
  {
    5
  }
}

/// File-based configuration layer
#[ derive( Debug ) ]
pub struct FileLayer
{
  /// Layer name
  name: String,
  /// Priority level (1-4)
  priority: u8,
  /// Parsed TOML content
  content: Option< toml::Table >,
  /// Source file path for debugging
  source_path: Option< PathBuf >,
}

impl FileLayer
{
  /// Create layer from TOML file
  pub fn from_file( name: impl Into< String >, priority: u8, path: PathBuf ) -> Result< Self >
  {
    let name = name.into();

    if !path.exists()
    {
      return Ok( Self
      {
        name,
        priority,
        content: None,
        source_path: Some( path ),
      } );
    }

    let contents = std::fs::read_to_string( &path )?;
    let content = toml::from_str::< toml::Table >( &contents )
      .map_err( | e | ConfigError::InvalidToml
      {
        path: path.clone(),
        error: e.to_string(),
      } )?;

    Ok( Self
    {
      name,
      priority,
      content: Some( content ),
      source_path: Some( path ),
    } )
  }

  /// Create layer from TOML string (for defaults)
  pub fn from_str( name: impl Into< String >, priority: u8, toml_str: &str ) -> Result< Self >
  {
    let content = toml::from_str::< toml::Table >( toml_str )
      .map_err( | e | ConfigError::InvalidToml
      {
        path: PathBuf::from( "<inline>" ),
        error: e.to_string(),
      } )?;

    Ok( Self
    {
      name: name.into(),
      priority,
      content: Some( content ),
      source_path: None,
    } )
  }

  /// Get nested value from TOML table by key path
  fn get_nested( table: &toml::Table, key_path: &str ) -> Option< toml::Value >
  {
    let parts: Vec< &str > = key_path.split( '.' ).collect();
    let mut current = toml::Value::Table( table.clone() );

    for part in parts
    {
      current = current.as_table()?.get( part )?.clone();
    }

    Some( current )
  }
}

impl ConfigLayer for FileLayer
{
  fn get( &self, key: &str ) -> Result< Option< ConfigValue > >
  {
    let Some( ref content ) = self.content else
    {
      return Ok( None );
    };

    if let Some( value ) = Self::get_nested( content, key )
    {
      let source = if let Some( ref path ) = self.source_path
      {
        format!( "{}:{}", self.name, path.display() )
      }
      else
      {
        self.name.clone()
      };

      Ok( Some( ConfigValue { value, source } ) )
    }
    else
    {
      Ok( None )
    }
  }

  fn get_all( &self ) -> Result< HashMap< String, ConfigValue > >
  {
    let mut result = HashMap::new();

    let Some( ref content ) = self.content else
    {
      return Ok( result );
    };

    fn flatten_table(
      table: &toml::Table,
      prefix: &str,
      result: &mut HashMap< String, toml::Value >,
    )
    {
      for ( key, value ) in table
      {
        let full_key = if prefix.is_empty()
        {
          key.clone()
        }
        else
        {
          format!( "{}.{}", prefix, key )
        };

        if let Some( nested_table ) = value.as_table()
        {
          flatten_table( nested_table, &full_key, result );
        }
        else
        {
          result.insert( full_key, value.clone() );
        }
      }
    }

    let mut flattened = HashMap::new();
    flatten_table( content, "", &mut flattened );

    let source = if let Some( ref path ) = self.source_path
    {
      format!( "{}:{}", self.name, path.display() )
    }
    else
    {
      self.name.clone()
    };

    for ( key, value ) in flattened
    {
      result.insert( key, ConfigValue
      {
        value,
        source: source.clone(),
      } );
    }

    Ok( result )
  }

  fn name( &self ) -> &str
  {
    &self.name
  }

  fn priority( &self ) -> u8
  {
    self.priority
  }
}

/// Configuration layers builder
pub struct LayersBuilder
{
  /// Module name (e.g., "iron_token_manager")
  module: String,
  /// Environment (e.g., "development", "test", "production")
  env: String,
  /// Custom layers (for testing)
  custom_layers: Vec< Box< dyn ConfigLayer > >,
}

impl LayersBuilder
{
  /// Create new builder for module
  pub fn new( module: impl Into< String > ) -> Self
  {
    let env = std::env::var( "IRON_ENV" ).unwrap_or_else( |_| "development".to_string() );

    Self
    {
      module: module.into(),
      env,
      custom_layers: Vec::new(),
    }
  }

  /// Set environment explicitly
  pub fn env( mut self, env: impl Into< String > ) -> Self
  {
    self.env = env.into();
    self
  }

  /// Add custom layer (for testing)
  pub fn add_layer( mut self, layer: Box< dyn ConfigLayer > ) -> Self
  {
    self.custom_layers.push( layer );
    self
  }

  /// Build all configuration layers
  pub fn build( self ) -> Result< Vec< Box< dyn ConfigLayer > > >
  {
    let mut layers: Vec< Box< dyn ConfigLayer > > = Vec::new();

    // Layer 1: Environment variables (priority 5)
    let env_prefix = format!( "{}_", self.module.to_uppercase().replace( '-', "_" ) );
    layers.push( Box::new( EnvLayer::new( env_prefix ) ) );

    // Layer 2: Project-specific config (priority 4)
    if let Ok( ws ) = workspace_tools::workspace()
    {
      let project_config = ws.root()
        .join( "config" )
        .join( format!( "{}.{}.toml", self.module, self.env ) );

      if let Ok( layer ) = FileLayer::from_file( "Project Config", 4, project_config )
      {
        layers.push( Box::new( layer ) );
      }

      // Layer 3: User config (priority 3)
      if let Some( home ) = dirs::home_dir()
      {
        let user_config = home
          .join( ".config" )
          .join( "iron" )
          .join( format!( "{}.toml", self.module ) );

        if let Ok( layer ) = FileLayer::from_file( "User Config", 3, user_config )
        {
          layers.push( Box::new( layer ) );
        }
      }

      // Layer 4: Workspace defaults (priority 2)
      let workspace_default = ws.root()
        .join( "config" )
        .join( format!( "{}.default.toml", self.module ) );

      if let Ok( layer ) = FileLayer::from_file( "Workspace Default", 2, workspace_default )
      {
        layers.push( Box::new( layer ) );
      }
    }

    // Add custom layers
    layers.extend( self.custom_layers );

    // Sort by priority (highest first)
    layers.sort_by_key( | b | std::cmp::Reverse( b.priority() ) );

    Ok( layers )
  }
}
