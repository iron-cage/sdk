//! Circuit breaker pattern for preventing cascading failures.
//!
//! Implements the circuit breaker reliability pattern to protect Iron Runtime
//! from cascading failures when upstream LLM providers become unavailable.
//! Automatically opens circuits after threshold failures, preventing wasted
//! requests to failing services.
//!
//! # Purpose
//!
//! This crate provides fault tolerance for LLM provider integration:
//! - Detect failing services through failure rate monitoring
//! - Prevent cascading failures by short-circuiting bad requests
//! - Auto-recovery with configurable timeout periods
//! - Per-service state isolation
//!
//! # Circuit Breaker States
//!
//! The circuit breaker follows a three-state model:
//!
//! - **Closed**: Normal operation, requests pass through. Failure counter increments
//!   on each failure. Transitions to Open when failures reach threshold.
//!
//! - **Open**: Circuit is open, requests fail fast without hitting upstream service.
//!   Prevents wasted resources on known-bad endpoints. Transitions to HalfOpen
//!   after timeout expires.
//!
//! - **HalfOpen**: Trial period, allows limited requests to test recovery.
//!   First success closes circuit. First failure reopens circuit.
//!
//! # Key Types
//!
//! - [`CircuitBreaker`] - Main circuit breaker with per-service state tracking
//! - [`CircuitState`] - Circuit state enum (Closed, Open, HalfOpen)
//!
//! # Public API
//!
//! ## Basic Usage
//!
//! ```rust
//! use iron_reliability::CircuitBreaker;
//!
//! // Create breaker: 5 failures triggers open, 60s timeout
//! let breaker = CircuitBreaker::new(5, 60);
//!
//! // Check before making request
//! if breaker.is_open("openai") {
//!   // Circuit open, fail fast
//!   return Err("Service unavailable");
//! }
//!
//! // Make request...
//! match make_llm_request() {
//!   Ok(response) => {
//!     breaker.record_success("openai");
//!     Ok(response)
//!   }
//!   Err(e) => {
//!     breaker.record_failure("openai");
//!     Err(e)
//!   }
//! }
//! # fn make_llm_request() -> Result<(), &'static str> { Ok(()) }
//! # Ok::<(), &str>(())
//! ```
//!
//! ## Integration Pattern
//!
//! ```rust
//! use iron_reliability::CircuitBreaker;
//! use std::sync::Arc;
//!
//! struct LlmRouter {
//!   breaker: Arc<CircuitBreaker>,
//! }
//!
//! impl LlmRouter {
//!   fn route_request(&self, provider: &str) -> Result<(), String> {
//!     // Fast-fail if circuit open
//!     if self.breaker.is_open(provider) {
//!       return Err(format!("Circuit open for {}", provider));
//!     }
//!
//!     // Attempt request
//!     match self.call_provider(provider) {
//!       Ok(resp) => {
//!         self.breaker.record_success(provider);
//!         Ok(resp)
//!       }
//!       Err(e) => {
//!         self.breaker.record_failure(provider);
//!         Err(e)
//!       }
//!     }
//!   }
//!
//!   fn call_provider(&self, provider: &str) -> Result<(), String> {
//!     // Implementation...
//!     # Ok(())
//!   }
//! }
//! ```
//!
//! # Configuration
//!
//! Circuit breaker behavior is controlled by two parameters:
//!
//! - **failure_threshold**: Number of consecutive failures before opening circuit.
//!   Higher values tolerate transient failures. Lower values provide faster detection.
//!   Typical: 3-10 failures.
//!
//! - **timeout_secs**: How long circuit stays open before attempting recovery.
//!   Longer timeouts reduce load on failing services. Shorter timeouts enable
//!   faster recovery. Typical: 30-120 seconds.
//!
//! # Thread Safety
//!
//! [`CircuitBreaker`] is thread-safe and designed for concurrent access.
//! Internal state uses `Arc<Mutex<>>` for safe sharing across request handlers.

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
