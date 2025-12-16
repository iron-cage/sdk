//! Configuration loader with precedence-based resolution
//!
//! Provides unified configuration loading across all Iron Runtime modules
//! using a 5-layer precedence system.

use crate::error::{ ConfigError, Result };
use crate::layer::{ ConfigLayer, ConfigValue, LayersBuilder };
use serde::de::DeserializeOwned;
use std::collections::HashMap;

/// Configuration loader with precedence-based resolution
///
/// Implements the 5-layer configuration precedence system:
/// 1. Environment variables (highest priority)
/// 2. Project-specific config (`{workspace}/config/{module}.{env}.toml`)
/// 3. User config (`~/.config/iron/{module}.toml`)
/// 4. Workspace defaults (`{workspace}/config/{module}.default.toml`)
/// 5. Crate defaults (lowest priority)
///
/// # Examples
///
/// ```ignore
/// use iron_config::ConfigLoader;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct DatabaseConfig {
///   url: String,
///   max_connections: u32,
/// }
///
/// let loader = ConfigLoader::new("iron_token_manager")?;
/// let db_config: DatabaseConfig = loader.get("database")?;
/// ```
pub struct ConfigLoader
{
  /// Configuration layers (sorted by priority)
  layers: Vec< Box< dyn ConfigLayer > >,
  /// Module name
  module: String,
  /// Resolved configuration cache
  cache: HashMap< String, ConfigValue >,
}

impl ConfigLoader
{
  /// Create new configuration loader for module
  ///
  /// # Arguments
  ///
  /// * `module` - Module name (e.g., "iron_token_manager")
  ///
  /// # Errors
  ///
  /// Returns error if workspace cannot be detected.
  ///
  /// # Examples
  ///
  /// ```ignore
  /// let loader = ConfigLoader::new("iron_token_manager")?;
  /// ```
  pub fn new( module: impl Into< String > ) -> Result< Self >
  {
    let module = module.into();
    let layers = LayersBuilder::new( module.clone() ).build()?;

    let mut loader = Self
    {
      layers,
      module,
      cache: HashMap::new(),
    };

    loader.resolve_all()?;

    Ok( loader )
  }

  /// Create new configuration loader with custom environment
  ///
  /// # Arguments
  ///
  /// * `module` - Module name
  /// * `env` - Environment (e.g., "development", "test", "production")
  ///
  /// # Examples
  ///
  /// ```ignore
  /// let loader = ConfigLoader::with_env("iron_token_manager", "production")?;
  /// ```
  pub fn with_env( module: impl Into< String >, env: impl Into< String > ) -> Result< Self >
  {
    let module = module.into();
    let layers = LayersBuilder::new( module.clone() )
      .env( env )
      .build()?;

    let mut loader = Self
    {
      layers,
      module,
      cache: HashMap::new(),
    };

    loader.resolve_all()?;

    Ok( loader )
  }

  /// Create configuration loader with custom default values
  ///
  /// # Arguments
  ///
  /// * `module` - Module name
  /// * `defaults` - Default configuration as TOML string
  ///
  /// # Examples
  ///
  /// ```ignore
  /// let defaults = r#"
  /// [database]
  /// url = "sqlite:///:memory:"
  /// max_connections = 5
  /// "#;
  ///
  /// let loader = ConfigLoader::with_defaults("iron_token_manager", defaults)?;
  /// ```
  pub fn with_defaults( module: impl Into< String >, defaults: &str ) -> Result< Self >
  {
    let module = module.into();
    let mut builder = LayersBuilder::new( module.clone() );

    // Add default layer (priority 1 - lowest)
    let default_layer = crate::layer::FileLayer::from_str( "Crate Defaults", 1, defaults )?;
    builder = builder.add_layer( Box::new( default_layer ) );

    let layers = builder.build()?;

    let mut loader = Self
    {
      layers,
      module,
      cache: HashMap::new(),
    };

    loader.resolve_all()?;

    Ok( loader )
  }

  /// Resolve all configuration values from layers
  fn resolve_all( &mut self ) -> Result< () >
  {
    // Collect all keys from all layers
    let mut all_keys = std::collections::HashSet::new();

    for layer in &self.layers
    {
      for key in layer.get_all()?.keys()
      {
        all_keys.insert( key.clone() );
      }
    }

    // Resolve each key using precedence
    for key in all_keys
    {
      if let Some( value ) = self.resolve_key( &key )?
      {
        self.cache.insert( key, value );
      }
    }

    Ok( () )
  }

  /// Resolve single key using precedence (highest priority wins)
  fn resolve_key( &self, key: &str ) -> Result< Option< ConfigValue > >
  {
    for layer in &self.layers
    {
      if let Some( value ) = layer.get( key )?
      {
        return Ok( Some( value ) );
      }
    }

    Ok( None )
  }

  /// Get configuration value by key path
  ///
  /// # Arguments
  ///
  /// * `key` - Configuration key path (e.g., "database.url")
  ///
  /// # Errors
  ///
  /// Returns error if key not found or value cannot be deserialized.
  ///
  /// # Examples
  ///
  /// ```ignore
  /// let url: String = loader.get("database.url")?;
  /// let max_conn: u32 = loader.get("database.max_connections")?;
  /// ```
  pub fn get< T: DeserializeOwned >( &self, key: &str ) -> Result< T >
  {
    let value = self.cache
      .get( key )
      .ok_or_else( || ConfigError::MissingKey( key.to_string() ) )?;

    let deserialized = value.value.clone().try_into::< T >()
      .map_err( | _e | ConfigError::InvalidType
      {
        key: key.to_string(),
        expected: std::any::type_name::< T >().to_string(),
        actual: format!( "{:?}", value.value ),
      } )?;

    Ok( deserialized )
  }

  /// Get optional configuration value
  ///
  /// Returns `None` if key not found, otherwise same as `get()`.
  ///
  /// # Examples
  ///
  /// ```ignore
  /// if let Some(url) = loader.get_opt::<String>("database.url")? {
  ///   println!("Database URL: {}", url);
  /// }
  /// ```
  pub fn get_opt< T: DeserializeOwned >( &self, key: &str ) -> Result< Option< T > >
  {
    match self.get( key )
    {
      Ok( value ) => Ok( Some( value ) ),
      Err( ConfigError::MissingKey( _ ) ) => Ok( None ),
      Err( e ) => Err( e ),
    }
  }

  /// Get configuration subtree as struct
  ///
  /// # Arguments
  ///
  /// * `prefix` - Key prefix (e.g., "database")
  ///
  /// # Errors
  ///
  /// Returns error if subtree cannot be deserialized.
  ///
  /// # Examples
  ///
  /// ```ignore
  /// #[derive(Deserialize)]
  /// struct DatabaseConfig {
  ///   url: String,
  ///   max_connections: u32,
  /// }
  ///
  /// let db_config: DatabaseConfig = loader.get_section("database")?;
  /// ```
  pub fn get_section< T: DeserializeOwned >( &self, prefix: &str ) -> Result< T >
  {
    // Collect all keys with this prefix
    let mut section = toml::Table::new();

    for ( key, value ) in &self.cache
    {
      if let Some( suffix ) = key.strip_prefix( &format!( "{}.", prefix ) )
      {
        // Reconstruct nested structure
        Self::insert_nested( &mut section, suffix, value.value.clone() );
      }
    }

    // Deserialize the section
    let toml_value = toml::Value::Table( section );
    T::deserialize( toml_value )
      .map_err( | e | ConfigError::InvalidType
      {
        key: prefix.to_string(),
        expected: std::any::type_name::< T >().to_string(),
        actual: format!( "{:?}", e ),
      } )
  }

  /// Insert value into nested TOML table
  fn insert_nested( table: &mut toml::Table, key_path: &str, value: toml::Value )
  {
    let parts: Vec< &str > = key_path.split( '.' ).collect();

    if parts.len() == 1
    {
      table.insert( parts[ 0 ].to_string(), value );
    }
    else
    {
      let first = parts[ 0 ];
      let rest = parts[ 1.. ].join( "." );

      let nested = table
        .entry( first.to_string() )
        .or_insert_with( || toml::Value::Table( toml::Table::new() ) )
        .as_table_mut()
        .unwrap();

      Self::insert_nested( nested, &rest, value );
    }
  }

  /// Get all configuration keys
  pub fn keys( &self ) -> Vec< String >
  {
    self.cache.keys().cloned().collect()
  }

  /// Get configuration value with source information
  ///
  /// Returns the value along with information about which layer provided it.
  ///
  /// # Examples
  ///
  /// ```ignore
  /// let (url, source) = loader.get_with_source::<String>("database.url")?;
  /// println!("Database URL: {} (from {})", url, source);
  /// ```
  pub fn get_with_source< T: DeserializeOwned >( &self, key: &str ) -> Result< ( T, String ) >
  {
    let value = self.cache
      .get( key )
      .ok_or_else( || ConfigError::MissingKey( key.to_string() ) )?;

    let deserialized = value.value.clone().try_into::< T >()
      .map_err( | _e | ConfigError::InvalidType
      {
        key: key.to_string(),
        expected: std::any::type_name::< T >().to_string(),
        actual: format!( "{:?}", value.value ),
      } )?;

    Ok( ( deserialized, value.source.clone() ) )
  }

  /// Print configuration summary for debugging
  ///
  /// Shows all resolved configuration values with their sources.
  pub fn debug_summary( &self ) -> String
  {
    let mut lines = Vec::new();
    lines.push( format!( "Configuration for '{}' ({} keys)", self.module, self.cache.len() ) );
    lines.push( String::new() );

    let mut keys: Vec< _ > = self.cache.keys().collect();
    keys.sort();

    for key in keys
    {
      if let Some( value ) = self.cache.get( key )
      {
        lines.push( format!( "  {} = {:?}", key, value.value ) );
        lines.push( format!( "    source: {}", value.source ) );
      }
    }

    lines.join( "\n" )
  }
}
