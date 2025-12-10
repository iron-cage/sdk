//! Currency conversion utilities for microdollar arithmetic.
//!
//! 1 USD = 1,000,000 microdollars
//!
//! Using integer arithmetic (microdollars) avoids floating-point precision
//! errors that can accumulate in financial calculations.

/// Microdollars per USD
pub const MICROS_PER_USD: u64 = 1_000_000;

/// Convert USD (f64) to microdollars (u64)
///
/// # Example
/// ```
/// use iron_cost::converter::usd_to_micros;
/// assert_eq!(usd_to_micros(1.50), 1_500_000);
/// assert_eq!(usd_to_micros(0.000001), 1);
/// ```
pub fn usd_to_micros(usd: f64) -> u64 {
    (usd * MICROS_PER_USD as f64).round().max(0.0) as u64
}

/// Convert microdollars (u64) to USD (f64)
///
/// # Example
/// ```
/// use iron_cost::converter::micros_to_usd;
/// assert_eq!(micros_to_usd(1_500_000), 1.5);
/// assert_eq!(micros_to_usd(1), 0.000001);
/// ```
pub fn micros_to_usd(micros: u64) -> f64 {
    micros as f64 / MICROS_PER_USD as f64
}
