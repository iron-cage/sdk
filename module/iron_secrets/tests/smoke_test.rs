//! Smoke tests for `iron_secrets`
//!
//! These tests verify that the crate compiles and core functionality exists.

use iron_secrets::crypto::{ CryptoService, KEY_SIZE };

/// Smoke test: Verify crypto module exists and basic encryption works
///
/// This test ensures the crypto module compiles and provides basic functionality.
#[test]
fn test_crypto_module_works()
{
  let key = [ 0x42u8; KEY_SIZE ];
  let crypto = CryptoService::new( &key ).unwrap();
  let encrypted = crypto.encrypt( "test" ).unwrap();
  let decrypted = crypto.decrypt( &encrypted ).unwrap();
  assert_eq!( &*decrypted, "test" );
}

/// Smoke test: Verify crate compiles with all features
///
/// This test ensures all modules compile successfully.
#[test]
fn test_crate_compiles()
{
  // This test passes if the crate builds with all features enabled
}
