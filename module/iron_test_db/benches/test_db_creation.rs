//! Benchmarks for test database creation

use criterion::{ criterion_group, criterion_main, Criterion, black_box };
use iron_test_db::TestDatabaseBuilder;

fn bench_in_memory_creation( c: &mut Criterion )
{
  let runtime = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .expect( "LOUD FAILURE: Failed to create tokio runtime" );

  c.bench_function( "create_in_memory_db", |b| {
    b.to_async( &runtime ).iter( || async {
      let db = TestDatabaseBuilder::new()
        .in_memory()
        .build()
        .await
        .expect( "LOUD FAILURE: Failed to create database" );
      black_box( db );
    } );
  } );
}

fn bench_temp_file_creation( c: &mut Criterion )
{
  let runtime = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .expect( "LOUD FAILURE: Failed to create tokio runtime" );

  c.bench_function( "create_temp_file_db", |b| {
    b.to_async( &runtime ).iter( || async {
      let db = TestDatabaseBuilder::new()
        .temp_file()
        .build()
        .await
        .expect( "LOUD FAILURE: Failed to create database" );
      black_box( db );
    } );
  } );
}

criterion_group!( benches, bench_in_memory_creation, bench_temp_file_creation );
criterion_main!( benches );
