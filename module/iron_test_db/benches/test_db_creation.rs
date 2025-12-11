//! Benchmarks for test database creation

use criterion::{ criterion_group, criterion_main, Criterion, black_box };
use iron_test_db::TestDatabaseBuilder;

fn bench_in_memory_creation( c: &mut Criterion )
{
  c.bench_function( "create_in_memory_db", |b| {
    b.iter( || {
      tokio::runtime::Runtime::new().unwrap().block_on( async {
        let db = TestDatabaseBuilder::new()
          .in_memory()
          .build()
          .await
          .expect( "Failed to create database" );
        black_box( db );
      } );
    } );
  } );
}

fn bench_temp_file_creation( c: &mut Criterion )
{
  c.bench_function( "create_temp_file_db", |b| {
    b.iter( || {
      tokio::runtime::Runtime::new().unwrap().block_on( async {
        let db = TestDatabaseBuilder::new()
          .temp_file()
          .build()
          .await
          .expect( "Failed to create database" );
        black_box( db );
      } );
    } );
  } );
}

criterion_group!( benches, bench_in_memory_creation, bench_temp_file_creation );
criterion_main!( benches );
