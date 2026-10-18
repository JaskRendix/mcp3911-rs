use mcp3911_rs::{
    raw_to_uv,
    registers::{Gain, Oversampling, Prescale, sample_rate_hz},
    sign_extend24,
};

#[test]
fn public_api_smoke_test() {
    assert_eq!(sign_extend24(0x00FF_FFFF), -1);

    let uv = raw_to_uv(1_000_000, Gain::X1, 1200);

    assert!(uv > 0);

    assert_eq!(
        sample_rate_hz(4_000_000, Oversampling::O256, Prescale::Mclk1),
        3906
    );
}
