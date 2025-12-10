use iron_cost::converter::{
    micros_to_usd, usd_to_micros,
    usd_per_token_to_micros_per_mtok, micros_per_mtok_to_usd_per_token,
};

// =============================================================================
// Basic USD <-> Micros conversions
// =============================================================================

#[test]
fn usd_to_micros_converts_correctly() {
    assert_eq!(usd_to_micros(1.0), 1_000_000);
    assert_eq!(usd_to_micros(0.5), 500_000);
    assert_eq!(usd_to_micros(0.000001), 1);
    assert_eq!(usd_to_micros(0.0), 0);
}

#[test]
fn micros_to_usd_converts_correctly() {
    assert_eq!(micros_to_usd(1_000_000), 1.0);
    assert_eq!(micros_to_usd(500_000), 0.5);
    assert_eq!(micros_to_usd(1), 0.000001);
    assert_eq!(micros_to_usd(0), 0.0);
}

#[test]
fn roundtrip_preserves_precision() {
    let original = 123.456789;
    let micros = usd_to_micros(original);
    let back = micros_to_usd(micros);
    // Should be within 1 microdollar precision
    assert!((original - back).abs() < 0.000001);
}

#[test]
fn negative_clamps_to_zero() {
    assert_eq!(usd_to_micros(-1.0), 0);
    assert_eq!(usd_to_micros(-0.5), 0);
}

// =============================================================================
// Per-token pricing conversions
// =============================================================================

#[test]
fn usd_per_token_to_micros_per_mtok_converts_correctly() {
    // $0.00000125/token = $1.25/M = 1,250,000 micros/M
    assert_eq!(usd_per_token_to_micros_per_mtok(0.00000125), 1_250_000);

    // $0.00001/token = $10/M = 10,000,000 micros/M
    assert_eq!(usd_per_token_to_micros_per_mtok(0.00001), 10_000_000);

    // $0.001/token = $1000/M = 1,000,000,000 micros/M
    assert_eq!(usd_per_token_to_micros_per_mtok(0.001), 1_000_000_000);
}

#[test]
fn micros_per_mtok_to_usd_per_token_converts_correctly() {
    // 1,250,000 micros/M = $1.25/M = $0.00000125/token
    assert_eq!(micros_per_mtok_to_usd_per_token(1_250_000), 0.00000125);

    // 10,000,000 micros/M = $10/M = $0.00001/token
    assert_eq!(micros_per_mtok_to_usd_per_token(10_000_000), 0.00001);
}

#[test]
fn per_token_roundtrip_conversion() {
    let original = 0.00000125;
    let micros = usd_per_token_to_micros_per_mtok(original);
    let back = micros_per_mtok_to_usd_per_token(micros);
    assert_eq!(original, back);
}
