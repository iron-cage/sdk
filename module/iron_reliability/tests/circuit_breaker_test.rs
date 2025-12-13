use iron_reliability::*;

#[ test ]
fn test_circuit_breaker()
{
  let cb = CircuitBreaker::new( 3, 60 );

  assert!( !cb.is_open( "service1" ), "Circuit should be closed initially with no failures" );

  cb.record_failure( "service1" );
  cb.record_failure( "service1" );
  assert!( !cb.is_open( "service1" ), "Circuit should remain closed with 2 failures (threshold is 3)" );

  cb.record_failure( "service1" );
  assert!( cb.is_open( "service1" ), "Circuit should open after reaching failure threshold (3 failures)" );
}
