//! Reliability module: circuit breakers and fallbacks

use std::collections::HashMap;
use std::sync::{ Arc, Mutex };
use std::time::{ Duration, Instant };

#[derive( Debug, Clone, Copy, PartialEq )]
pub enum CircuitState
{
  Closed,
  Open,
  HalfOpen,
}

type CircuitStateEntry = ( CircuitState, Instant, u32 );

pub struct CircuitBreaker
{
  state : Arc< Mutex< HashMap< String, CircuitStateEntry > > >,
  failure_threshold : u32,
  timeout : Duration,
}

impl CircuitBreaker
{
  pub fn new( failure_threshold : u32, timeout_secs : u64 ) -> Self
  {
    Self
    {
      state : Arc::new( Mutex::new( HashMap::new() ) ),
      failure_threshold,
      timeout : Duration::from_secs( timeout_secs ),
    }
  }

  pub fn is_open( &self, service : &str ) -> bool
  {
    let state = self.state.lock().unwrap();
    if let Some( ( circuit_state, opened_at, _ ) ) = state.get( service )
    {
      if *circuit_state == CircuitState::Open && opened_at.elapsed() < self.timeout
      {
        return true;
      }
    }
    false
  }

  pub fn record_success( &self, service : &str )
  {
    let mut state = self.state.lock().unwrap();
    state.insert( service.to_string(), ( CircuitState::Closed, Instant::now(), 0 ) );
  }

  pub fn record_failure( &self, service : &str )
  {
    let mut state = self.state.lock().unwrap();
    let entry = state.entry( service.to_string() )
      .or_insert( ( CircuitState::Closed, Instant::now(), 0 ) );

    entry.2 += 1;
    if entry.2 >= self.failure_threshold
    {
      entry.0 = CircuitState::Open;
      entry.1 = Instant::now();
    }
  }
}

#[cfg( test )]
mod tests
{
  use super::*;

  #[test]
  fn test_circuit_breaker()
  {
    let cb = CircuitBreaker::new( 3, 60 );

    assert!( !cb.is_open( "service1" ) );

    cb.record_failure( "service1" );
    cb.record_failure( "service1" );
    assert!( !cb.is_open( "service1" ) );

    cb.record_failure( "service1" );
    assert!( cb.is_open( "service1" ) );
  }
}
