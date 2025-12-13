//! Performance benchmarks for formatting
//!
//! Measures TreeFmtFormatter performance across different data sizes
//! and formats. Used to establish baseline and detect regressions.
//!
//! # Running Benchmarks
//!
//! ```bash
//! cargo bench --bench formatting_benchmarks
//! ```
//!
//! # Expected Results
//!
//! - Small data (2 keys): ~1-2 µs
//! - Large data (100 keys): ~40-60 µs
//! - List (10 items): ~15-20 µs
//! - List (1000 items): ~1.5-2.5 ms
//!
//! # Regression Detection
//!
//! Alert if any benchmark shows >50% regression from baseline.

use criterion::{ black_box, criterion_group, criterion_main, Criterion };
use iron_cli::formatting::{ TreeFmtFormatter, OutputFormat };
use std::collections::HashMap;

/// Benchmark format_single with small dataset (2 keys)
fn benchmark_format_single_small( c: &mut Criterion )
{
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let mut data = HashMap::new();
  data.insert( "key1".to_string(), "value1".to_string() );
  data.insert( "key2".to_string(), "value2".to_string() );

  c.bench_function( "format_single_small_2keys", |b|
  {
    b.iter( ||
    {
      formatter.format_single( black_box( &data ) )
    });
  });
}

/// Benchmark format_single with medium dataset (10 keys)
fn benchmark_format_single_medium( c: &mut Criterion )
{
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let mut data = HashMap::new();
  for i in 0..10
  {
    data.insert( format!( "key{}", i ), format!( "value{}", i ) );
  }

  c.bench_function( "format_single_medium_10keys", |b|
  {
    b.iter( ||
    {
      formatter.format_single( black_box( &data ) )
    });
  });
}

/// Benchmark format_single with large dataset (100 keys)
fn benchmark_format_single_large( c: &mut Criterion )
{
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let mut data = HashMap::new();
  for i in 0..100
  {
    data.insert( format!( "key{}", i ), format!( "value{}", i ) );
  }

  c.bench_function( "format_single_large_100keys", |b|
  {
    b.iter( ||
    {
      formatter.format_single( black_box( &data ) )
    });
  });
}

/// Benchmark format_list with small dataset (10 items)
fn benchmark_format_list_small( c: &mut Criterion )
{
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let items: Vec<_> = ( 0..10 )
    .map( |i|
    {
      let mut data = HashMap::new();
      data.insert( "id".to_string(), i.to_string() );
      data.insert( "name".to_string(), format!( "Item{}", i ) );
      data.insert( "status".to_string(), "active".to_string() );
      data
    })
    .collect();

  c.bench_function( "format_list_small_10items", |b|
  {
    b.iter( ||
    {
      formatter.format_list( black_box( &items ) )
    });
  });
}

/// Benchmark format_list with medium dataset (100 items)
fn benchmark_format_list_medium( c: &mut Criterion )
{
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let items: Vec<_> = ( 0..100 )
    .map( |i|
    {
      let mut data = HashMap::new();
      data.insert( "id".to_string(), i.to_string() );
      data.insert( "name".to_string(), format!( "Item{}", i ) );
      data.insert( "status".to_string(), "active".to_string() );
      data
    })
    .collect();

  c.bench_function( "format_list_medium_100items", |b|
  {
    b.iter( ||
    {
      formatter.format_list( black_box( &items ) )
    });
  });
}

/// Benchmark format_list with large dataset (1000 items)
fn benchmark_format_list_large( c: &mut Criterion )
{
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let items: Vec<_> = ( 0..1000 )
    .map( |i|
    {
      let mut data = HashMap::new();
      data.insert( "id".to_string(), i.to_string() );
      data.insert( "name".to_string(), format!( "Item{}", i ) );
      data.insert( "status".to_string(), "active".to_string() );
      data
    })
    .collect();

  c.bench_function( "format_list_large_1000items", |b|
  {
    b.iter( ||
    {
      formatter.format_list( black_box( &items ) )
    });
  });
}

/// Benchmark different output formats (Table vs Expanded vs JSON vs YAML)
fn benchmark_output_formats( c: &mut Criterion )
{
  let mut data = HashMap::new();
  for i in 0..10
  {
    data.insert( format!( "key{}", i ), format!( "value{}", i ) );
  }

  // Table format
  let table_formatter = TreeFmtFormatter::new( OutputFormat::Table );
  c.bench_function( "format_single_table", |b|
  {
    b.iter( ||
    {
      table_formatter.format_single( black_box( &data ) )
    });
  });

  // Expanded format
  let expanded_formatter = TreeFmtFormatter::new( OutputFormat::Expanded );
  c.bench_function( "format_single_expanded", |b|
  {
    b.iter( ||
    {
      expanded_formatter.format_single( black_box( &data ) )
    });
  });

  // JSON format
  let json_formatter = TreeFmtFormatter::new( OutputFormat::Json );
  c.bench_function( "format_single_json", |b|
  {
    b.iter( ||
    {
      json_formatter.format_single( black_box( &data ) )
    });
  });

  // YAML format
  let yaml_formatter = TreeFmtFormatter::new( OutputFormat::Yaml );
  c.bench_function( "format_single_yaml", |b|
  {
    b.iter( ||
    {
      yaml_formatter.format_single( black_box( &data ) )
    });
  });
}

/// Benchmark with realistic CLI data (token data)
fn benchmark_realistic_token_data( c: &mut Criterion )
{
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let mut token_data = HashMap::new();
  token_data.insert( "token_id".to_string(), "tok_abc123def456ghi789".to_string() );
  token_data.insert( "name".to_string(), "ProductionAPIToken".to_string() );
  token_data.insert( "scope".to_string(), "read:write:admin".to_string() );
  token_data.insert( "created_at".to_string(), "2025-12-12T10:30:00Z".to_string() );
  token_data.insert( "expires_at".to_string(), "2026-12-12T10:30:00Z".to_string() );
  token_data.insert( "status".to_string(), "active".to_string() );

  c.bench_function( "format_single_realistic_token", |b|
  {
    b.iter( ||
    {
      formatter.format_single( black_box( &token_data ) )
    });
  });
}

/// Benchmark with realistic CLI data (token list)
fn benchmark_realistic_token_list( c: &mut Criterion )
{
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let tokens: Vec<_> = ( 0..20 )
    .map( |i|
    {
      let mut data = HashMap::new();
      data.insert( "token_id".to_string(), format!( "tok_abc{:03}", i ) );
      data.insert( "name".to_string(), format!( "Token-{}", i ) );
      data.insert( "scope".to_string(), "read:write".to_string() );
      data.insert( "status".to_string(), if i % 3 == 0 { "revoked".to_string() } else { "active".to_string() } );
      data
    })
    .collect();

  c.bench_function( "format_list_realistic_tokens_20", |b|
  {
    b.iter( ||
    {
      formatter.format_list( black_box( &tokens ) )
    });
  });
}

criterion_group!(
  single_benchmarks,
  benchmark_format_single_small,
  benchmark_format_single_medium,
  benchmark_format_single_large,
);

criterion_group!(
  list_benchmarks,
  benchmark_format_list_small,
  benchmark_format_list_medium,
  benchmark_format_list_large,
);

criterion_group!(
  format_benchmarks,
  benchmark_output_formats,
);

criterion_group!(
  realistic_benchmarks,
  benchmark_realistic_token_data,
  benchmark_realistic_token_list,
);

criterion_main!(
  single_benchmarks,
  list_benchmarks,
  format_benchmarks,
  realistic_benchmarks,
);
