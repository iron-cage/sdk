use iron_cost::converter::{micros_to_usd, usd_to_micros};

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
