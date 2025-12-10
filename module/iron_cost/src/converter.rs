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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usd_to_micros() {
        assert_eq!(usd_to_micros(1.0), 1_000_000);
        assert_eq!(usd_to_micros(0.5), 500_000);
        assert_eq!(usd_to_micros(0.000001), 1);
        assert_eq!(usd_to_micros(0.0), 0);
    }

    #[test]
    fn test_micros_to_usd() {
        assert_eq!(micros_to_usd(1_000_000), 1.0);
        assert_eq!(micros_to_usd(500_000), 0.5);
        assert_eq!(micros_to_usd(1), 0.000001);
        assert_eq!(micros_to_usd(0), 0.0);
    }

    #[test]
    fn test_roundtrip() {
        let original = 123.456789;
        let micros = usd_to_micros(original);
        let back = micros_to_usd(micros);
        // Should be within 1 microdollar precision
        assert!((original - back).abs() < 0.000001);
    }

    #[test]
    fn test_negative_clamps_to_zero() {
        assert_eq!(usd_to_micros(-1.0), 0);
        assert_eq!(usd_to_micros(-0.5), 0);
    }
}