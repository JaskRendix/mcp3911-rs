extern crate std;

use std::vec;

use crate::registers::{
    self, Channel, ConfGain, Config, ConfigReg, DrMode, Gain, Oversampling, StatusReg,
};
use crate::{raw_to_uv, sign_extend24, to_u24, MCP3911};

#[test]
fn sign_extension_zero() {
    assert_eq!(sign_extend24(0x000000), 0);
}

#[test]
fn sign_extension_positive() {
    assert_eq!(sign_extend24(0x007FFFFF), 8388607);
}

#[test]
fn sign_extension_negative_limit() {
    assert_eq!(sign_extend24(0x00800000), -8388608);
}

#[test]
fn sign_extension_minus_one() {
    assert_eq!(sign_extend24(0x00FFFFFF), -1);
}

#[test]
fn sign_extension_small_negative() {
    assert_eq!(sign_extend24(0x00FFFFFE), -2);
}

#[test]
fn sign_extension_small_positive() {
    assert_eq!(sign_extend24(0x000001), 1);
}

#[test]
fn gain_multiplier_values() {
    assert_eq!(Gain::X1.multiplier(), 1);
    assert_eq!(Gain::X2.multiplier(), 2);
    assert_eq!(Gain::X4.multiplier(), 4);
    assert_eq!(Gain::X8.multiplier(), 8);
    assert_eq!(Gain::X16.multiplier(), 16);
    assert_eq!(Gain::X32.multiplier(), 32);
}

#[test]
fn gain_register_x8_x8() {
    let conf = ConfGain::new().with_ch0(Gain::X8).with_ch1(Gain::X8);

    assert_eq!(conf.into_bytes(), [0x1B]);
}

#[test]
fn gain_register_x1_x1() {
    let conf = ConfGain::new().with_ch0(Gain::X1).with_ch1(Gain::X1);

    assert_eq!(conf.into_bytes(), [0x00]);
}

#[test]
fn gain_register_x32_x32() {
    let conf = ConfGain::new().with_ch0(Gain::X32).with_ch1(Gain::X32);

    assert_eq!(conf.into_bytes(), [0x2D]);
}

#[test]
fn config_osr_roundtrip() {
    let reg = ConfigReg::new().with_osr(Oversampling::O256);

    let bytes = reg.into_bytes();

    let decoded = ConfigReg::from_bytes(bytes);

    assert_eq!(decoded.osr(), Oversampling::O256);
}

#[test]
fn config_prescale_roundtrip() {
    let reg = ConfigReg::new().with_prescale(registers::Prescale::Mclk8);

    let bytes = reg.into_bytes();

    let decoded = ConfigReg::from_bytes(bytes);

    assert_eq!(decoded.prescale(), registers::Prescale::Mclk8);
}

#[test]
fn raw_to_uv_zero() {
    assert_eq!(raw_to_uv(0, Gain::X1, 1200), 0);
}

#[test]
fn raw_to_uv_full_scale_positive() {
    let uv = raw_to_uv(8_388_607, Gain::X1, 1200);

    assert_eq!(uv, 799_999);
}

#[test]
fn read_channel0_sends_correct_command() {
    use embedded_hal_mock::eh1::spi::{Mock, Transaction};

    let expectations = [
        Transaction::transaction_start(),
        Transaction::write_vec(vec![0x01]),
        Transaction::read_vec(vec![0x12, 0x34, 0x56]),
        Transaction::transaction_end(),
    ];

    let spi = Mock::new(&expectations);

    let mut adc = MCP3911::new(spi, 1200);

    let _ = adc.read_raw(Channel::Ch0);

    let mut spi = adc.release();
    spi.done();
}

#[test]
fn gain_write_sends_correct_frame() {
    use embedded_hal_mock::eh1::spi::{Mock, Transaction};

    let expectations = [
        Transaction::transaction_start(),
        Transaction::write_vec(vec![0x12]),
        Transaction::write_vec(vec![0x1B]),
        Transaction::transaction_end(),
    ];

    let spi = Mock::new(&expectations);

    let mut adc = MCP3911::new(spi, 1200);

    adc.set_gain_and_boost(Gain::X8, Gain::X8, registers::Boost::X0_5)
        .unwrap();

    let mut spi = adc.release();
    spi.done();
}

#[test]
fn configure_builds_successfully() {
    let cfg = Config::default()
        .gain_ch0(Gain::X8)
        .gain_ch1(Gain::X8)
        .boost(registers::Boost::X1)
        .osr(Oversampling::O256)
        .prescale(registers::Prescale::Mclk1);

    assert_eq!(cfg.osr, Oversampling::O256);
}

#[test]
fn config_osr_and_prescale_roundtrip() {
    let reg = ConfigReg::new()
        .with_osr(Oversampling::O256)
        .with_prescale(registers::Prescale::Mclk4);

    let bytes = reg.into_bytes();
    let decoded = ConfigReg::from_bytes(bytes);

    assert_eq!(decoded.osr(), Oversampling::O256);
    assert_eq!(decoded.prescale(), registers::Prescale::Mclk4);
}

#[test]
fn to_u24_zero() {
    assert_eq!(to_u24(0), [0x00, 0x00, 0x00]);
}

#[test]
fn to_u24_positive() {
    assert_eq!(to_u24(0x123456), [0x12, 0x34, 0x56]);
}

#[test]
fn to_u24_minus_one() {
    assert_eq!(to_u24(-1), [0xFF, 0xFF, 0xFF]);
}

#[test]
fn to_u24_negative_limit() {
    assert_eq!(to_u24(-8_388_608), [0x80, 0x00, 0x00]);
}

#[test]
fn read_gain_calibration_ch0() {
    use embedded_hal_mock::eh1::spi::{Mock, Transaction};

    let expectations = [
        Transaction::transaction_start(),
        Transaction::write_vec(vec![0x23]),
        Transaction::read_vec(vec![0x12, 0x34, 0x56]),
        Transaction::transaction_end(),
    ];

    let spi = Mock::new(&expectations);

    let mut adc = MCP3911::new(spi, 1200);

    let value = adc.read_gain_calibration(Channel::Ch0).unwrap();

    assert_eq!(value, 0x123456);

    adc.release().done();
}

#[test]
fn write_gain_calibration_ch0() {
    use embedded_hal_mock::eh1::spi::{Mock, Transaction};

    let expectations = [
        Transaction::transaction_start(),
        Transaction::write_vec(vec![0x22]),
        Transaction::write_vec(vec![0x12, 0x34, 0x56]),
        Transaction::transaction_end(),
    ];

    let spi = Mock::new(&expectations);

    let mut adc = MCP3911::new(spi, 1200);

    adc.write_gain_calibration(Channel::Ch0, 0x123456).unwrap();

    adc.release().done();
}

#[test]
fn read_offset_calibration_ch0() {
    use embedded_hal_mock::eh1::spi::{Mock, Transaction};

    let expectations = [
        Transaction::transaction_start(),
        Transaction::write_vec(vec![0x1D]),
        Transaction::read_vec(vec![0xFF, 0xFF, 0xFF]),
        Transaction::transaction_end(),
    ];

    let spi = Mock::new(&expectations);

    let mut adc = MCP3911::new(spi, 1200);

    let value = adc.read_offset_calibration(Channel::Ch0).unwrap();

    assert_eq!(value, -1);

    adc.release().done();
}

#[test]
fn write_offset_calibration_ch0() {
    use embedded_hal_mock::eh1::spi::{Mock, Transaction};

    let expectations = [
        Transaction::transaction_start(),
        Transaction::write_vec(vec![0x1C]),
        Transaction::write_vec(vec![0xFF, 0xFF, 0xFF]),
        Transaction::transaction_end(),
    ];

    let spi = Mock::new(&expectations);

    let mut adc = MCP3911::new(spi, 1200);

    adc.write_offset_calibration(Channel::Ch0, -1).unwrap();

    adc.release().done();
}

#[test]
fn write_vrefcal() {
    use embedded_hal_mock::eh1::spi::{Mock, Transaction};

    let expectations = [
        Transaction::transaction_start(),
        Transaction::write_vec(vec![0x34]),
        Transaction::write_vec(vec![0x42]),
        Transaction::transaction_end(),
    ];

    let spi = Mock::new(&expectations);

    let mut adc = MCP3911::new(spi, 1200);

    adc.write_vrefcal(0x42).unwrap();

    adc.release().done();
}

#[test]
fn read_vrefcal() {
    use embedded_hal_mock::eh1::spi::{Mock, Transaction};

    let expectations = [
        Transaction::transaction_start(),
        Transaction::write_vec(vec![0x35]),
        Transaction::read_vec(vec![0x42]),
        Transaction::transaction_end(),
    ];

    let spi = Mock::new(&expectations);

    let mut adc = MCP3911::new(spi, 1200);

    let value = adc.read_vrefcal().unwrap();

    assert_eq!(value, 0x42);

    adc.release().done();
}

#[test]
fn status_drmode_roundtrip() {
    let reg = StatusReg::new().with_drmode(DrMode::Both);

    let decoded = StatusReg::from_bytes(reg.into_bytes());

    assert_eq!(decoded.drmode(), DrMode::Both);
}

#[test]
fn status_enable_calibration_bits() {
    let reg = StatusReg::new().with_en_gaincal(true).with_en_offcal(true);

    let decoded = StatusReg::from_bytes(reg.into_bytes());

    assert!(decoded.en_gaincal());
    assert!(decoded.en_offcal());
}

#[test]
fn phase_zero_roundtrip() {
    let phase: i16 = 0;

    let raw = (phase as u16) & 0x0FFF;

    assert_eq!(raw, 0x0000);
}

#[test]
fn phase_positive_limit() {
    let phase: i16 = 2047;

    let raw = (phase as u16) & 0x0FFF;

    assert_eq!(raw, 0x07FF);
}

#[test]
fn phase_negative_limit() {
    let phase: i16 = -2048;

    let raw = (phase as u16) & 0x0FFF;

    assert_eq!(raw, 0x0800);
}

#[test]
fn phase_sign_extension_positive() {
    let raw: u16 = 0x07FF;

    let phase = (((raw & 0x0FFF) << 4) as i16) >> 4;

    assert_eq!(phase, 2047);
}

#[test]
fn phase_sign_extension_negative() {
    let raw: u16 = 0x0800;

    let phase = (((raw & 0x0FFF) << 4) as i16) >> 4;

    assert_eq!(phase, -2048);
}

#[test]
fn phase_sign_extension_minus_one() {
    let raw: u16 = 0x0FFF;

    let phase = (((raw & 0x0FFF) << 4) as i16) >> 4;

    assert_eq!(phase, -1);
}
