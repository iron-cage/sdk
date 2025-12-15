use criterion::{ black_box, criterion_group, criterion_main, Criterion };
use iron_types::{ AgentId, ProviderId };

fn bench_generate( c: &mut Criterion )
{
  c.bench_function( "AgentId::generate", |b| {
    b.iter( || black_box( AgentId::generate() ) );
  } );
}

fn bench_parse_valid( c: &mut Criterion )
{
  let id_str = "agent_550e8400-e29b-41d4-a716-446655440000";

  c.bench_function( "AgentId::parse (valid)", |b| {
    b.iter( || black_box( AgentId::parse( id_str ).unwrap() ) );
  } );
}

fn bench_parse_invalid( c: &mut Criterion )
{
  let id_str = "invalid-format";

  c.bench_function( "AgentId::parse (invalid)", |b| {
    b.iter( || black_box( AgentId::parse( id_str ).err() ) );
  } );
}

fn bench_parse_flexible_current( c: &mut Criterion )
{
  let id_str = "agent_550e8400-e29b-41d4-a716-446655440000";

  c.bench_function( "AgentId::parse_flexible (current format)", |b| {
    b.iter( || black_box( AgentId::parse_flexible( id_str ).unwrap() ) );
  } );
}

fn bench_parse_flexible_legacy( c: &mut Criterion )
{
  let id_str = "agent-550e8400-e29b-41d4-a716-446655440000";

  c.bench_function( "AgentId::parse_flexible (legacy format)", |b| {
    b.iter( || black_box( AgentId::parse_flexible( id_str ).unwrap() ) );
  } );
}

fn bench_as_str( c: &mut Criterion )
{
  let id = AgentId::generate();

  c.bench_function( "AgentId::as_str", |b| {
    b.iter( || black_box( id.as_str() ) );
  } );
}

fn bench_clone( c: &mut Criterion )
{
  let id = AgentId::generate();

  c.bench_function( "AgentId::clone", |b| {
    b.iter( || black_box( id.clone() ) );
  } );
}

fn bench_to_string( c: &mut Criterion )
{
  let id = AgentId::generate();

  c.bench_function( "AgentId::to_string", |b| {
    b.iter( || black_box( id.to_string() ) );
  } );
}

fn bench_from_str( c: &mut Criterion )
{
  let id_str = "agent_550e8400-e29b-41d4-a716-446655440000";

  c.bench_function( "AgentId from_str", |b| {
    b.iter( || black_box( id_str.parse::< AgentId >().unwrap() ) );
  } );
}

fn bench_serialize( c: &mut Criterion )
{
  let id = AgentId::generate();

  c.bench_function( "AgentId serde serialize", |b| {
    b.iter( || black_box( serde_json::to_string( &id ).unwrap() ) );
  } );
}

fn bench_deserialize( c: &mut Criterion )
{
  let json = r#""agent_550e8400-e29b-41d4-a716-446655440000""#;

  c.bench_function( "AgentId serde deserialize", |b| {
    b.iter( || black_box( serde_json::from_str::< AgentId >( json ).unwrap() ) );
  } );
}

fn bench_multiple_id_types( c: &mut Criterion )
{
  c.bench_function( "Generate multiple ID types", |b| {
    b.iter( || {
      black_box( AgentId::generate() );
      black_box( ProviderId::generate() );
    } );
  } );
}

criterion_group!(
  benches,
  bench_generate,
  bench_parse_valid,
  bench_parse_invalid,
  bench_parse_flexible_current,
  bench_parse_flexible_legacy,
  bench_as_str,
  bench_clone,
  bench_to_string,
  bench_from_str,
  bench_serialize,
  bench_deserialize,
  bench_multiple_id_types,
);

criterion_main!( benches );
