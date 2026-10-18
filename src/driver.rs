use embedded_hal::spi::SpiDevice;

use crate::registers::{
    self, CHANNEL_REGISTER_BYTES, CONFIG_REGISTER_BYTES, Channel, ConfGain, Config, ConfigReg,
    Gain, Oversampling, PHASE_REGISTER_BYTES, Register, STATUS_REGISTER_BYTES, StatusReg,
    VREFCAL_REGISTER_BYTES,
};

// --- Constants ---
const READ_BIT: u8 = 0x01;

/// Full-scale ADC code.
///
/// From Equation 5-3:
/// DATA = Vin / Vref × PGA × 1.5 × 8,388,608
///
/// Therefore:
/// FS = 1.5 × 8,388,608 = 12,582,912 counts.
const MCP3911_FULL_SCALE_COUNTS: i64 = 12_582_912;

const MIN_I24: i32 = -8_388_608;
const MAX_I24: i32 = 8_388_607;

/// Comprehensive driver error type
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Error<SpiError> {
    Spi(SpiError),
    ValueOutOfRange,
}

/// Standalone helper for 24-bit two's complement sign extension
#[inline]
pub const fn sign_extend24(raw: u32) -> i32 {
    ((raw << 8) as i32) >> 8
}

#[inline]
pub const fn to_u24(value: i32) -> [u8; 3] {
    [
        ((value >> 16) & 0xFF) as u8,
        ((value >> 8) & 0xFF) as u8,
        (value & 0xFF) as u8,
    ]
}

#[inline]
fn decode_i24(buf: &[u8; CHANNEL_REGISTER_BYTES]) -> i32 {
    let raw = ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32);

    sign_extend24(raw)
}

#[inline]
fn validate_i24(value: i32) -> bool {
    (MIN_I24..=MAX_I24).contains(&value)
}

/// Converts a raw ADC code to microvolts.
///
/// Based on MCP3911 Equation 5-3:
///
/// DATA = Vin / Vref × PGA × 1.5 × 8,388,608
pub fn raw_to_uv(raw: i32, gain: Gain, vref_mv: u16) -> i64 {
    let gain_multiplier = gain.multiplier() as i64;
    let numerator = (raw as i64) * (vref_mv as i64) * 1_000;
    let denominator = MCP3911_FULL_SCALE_COUNTS * gain_multiplier;

    numerator / denominator
}

pub struct MCP3911<SPI> {
    spi: SPI,
    vref_mv: u16,
}

impl<SPI, E> MCP3911<SPI>
where
    SPI: SpiDevice<Error = E>,
{
    /// Create a new instance of the MCP3911 driver with a given Vref in millivolts (e.g., 1200 for 1.2V)
    pub fn new(spi: SPI, vref_mv: u16) -> Self {
        Self { spi, vref_mv }
    }

    /// Reset the device using strongly-typed configuration register bits (ch0_reset & ch1_reset)
    pub fn reset(&mut self) -> Result<(), Error<E>> {
        let config = ConfigReg::new().with_ch0_reset(true).with_ch1_reset(true);

        self.write_register(Register::Config, &config.into_bytes())
    }

    #[inline]
    fn read_i24_register(&mut self, reg: Register) -> Result<i32, Error<E>> {
        let mut buf = [0u8; CHANNEL_REGISTER_BYTES];

        self.read_register(reg, &mut buf)?;

        Ok(decode_i24(&buf))
    }

    #[inline]
    fn write_i24_register(&mut self, reg: Register, value: i32) -> Result<(), Error<E>> {
        self.write_register(reg, &to_u24(value))
    }

    /// Read a signed 24-bit two's complement ADC sample.
    ///
    /// The returned value is sign-extended to `i32`.
    pub fn read_raw(&mut self, channel: Channel) -> Result<i32, Error<E>> {
        self.read_i24_register(channel.data_register())
    }

    /// Convenience wrapper for Channel 0 raw read
    pub fn read_raw_ch0(&mut self) -> Result<i32, Error<E>> {
        self.read_raw(Channel::Ch0)
    }

    /// Convenience wrapper for Channel 1 raw read
    pub fn read_raw_ch1(&mut self) -> Result<i32, Error<E>> {
        self.read_raw(Channel::Ch1)
    }

    /// Convert a raw ADC code into microvolts.
    ///
    /// Uses the configured reference voltage supplied to
    /// [`MCP3911::new`] and the specified PGA gain.
    pub fn counts_to_uv(&self, raw: i32, gain: Gain) -> i64 {
        raw_to_uv(raw, gain, self.vref_mv)
    }

    /// Apply a full ergonomic configuration bundle
    pub fn configure(&mut self, config: Config) -> Result<(), Error<E>> {
        self.set_gain_and_boost(config.gain_ch0, config.gain_ch1, config.boost)?;

        self.set_config_fields(config.osr, config.prescale, config.dither)
    }

    pub fn set_gain_and_boost(
        &mut self,
        ch0: Gain,
        ch1: Gain,
        boost: registers::Boost,
    ) -> Result<(), Error<E>> {
        let conf = ConfGain::new()
            .with_ch0(ch0)
            .with_ch1(ch1)
            .with_boost(boost);
        self.write_register(Register::Gain, &conf.into_bytes())
    }

    /// Note:
    /// According to the MCP3911 datasheet, modifying
    /// OSR or PRESCALE automatically resets and
    /// restarts both ADCs.
    pub fn set_config_fields(
        &mut self,
        osr: Oversampling,
        prescale: registers::Prescale,
        dither: registers::Dither,
    ) -> Result<(), Error<E>> {
        let mut buf = [0u8; CONFIG_REGISTER_BYTES];

        self.read_register(Register::Config, &mut buf)?;

        let mut config = ConfigReg::from_bytes(buf);

        config.set_osr(osr);
        config.set_prescale(prescale);
        config.set_dither(dither);

        self.write_register(Register::Config, &config.into_bytes())
    }

    /// Low-level register read
    pub fn read_register(&mut self, addr: Register, buffer: &mut [u8]) -> Result<(), Error<E>> {
        let cmd = ((addr as u8) << 1) | READ_BIT;
        self.spi
            .transaction(&mut [
                embedded_hal::spi::Operation::Write(&[cmd]),
                embedded_hal::spi::Operation::Read(buffer),
            ])
            .map_err(Error::Spi)?;
        Ok(())
    }

    /// Low-level register write
    pub fn write_register(&mut self, addr: Register, data: &[u8]) -> Result<(), Error<E>> {
        let cmd = (addr as u8) << 1;
        self.spi
            .transaction(&mut [
                embedded_hal::spi::Operation::Write(&[cmd]),
                embedded_hal::spi::Operation::Write(data),
            ])
            .map_err(Error::Spi)?;
        Ok(())
    }

    /// Destroys the driver and returns the underlying SPI device (useful for tests to verify expectations)
    pub fn release(self) -> SPI {
        self.spi
    }

    /// Read the raw GAINCAL register value.
    ///
    /// See datasheet section 5.12.2.
    pub fn read_gain_calibration(&mut self, channel: Channel) -> Result<i32, Error<E>> {
        self.read_i24_register(channel.gaincal_register())
    }

    /// Write the raw GAINCAL register value.
    ///
    /// This is a signed 24-bit calibration value and
    /// does not directly
    pub fn write_gain_calibration(&mut self, channel: Channel, value: i32) -> Result<(), Error<E>> {
        if !validate_i24(value) {
            return Err(Error::ValueOutOfRange);
        }

        self.write_i24_register(channel.gaincal_register(), value)
    }

    /// Read the raw OFFCAL register value.
    ///
    /// See datasheet section 5.12.1.
    pub fn read_offset_calibration(&mut self, channel: Channel) -> Result<i32, Error<E>> {
        self.read_i24_register(channel.offcal_register())
    }

    /// Write the raw OFFCAL register value.
    ///
    /// The value is stored as a signed 24-bit
    /// two's complement offset correction.
    pub fn write_offset_calibration(
        &mut self,
        channel: Channel,
        value: i32,
    ) -> Result<(), Error<E>> {
        if !validate_i24(value) {
            return Err(Error::ValueOutOfRange);
        }

        self.write_i24_register(channel.offcal_register(), value)
    }

    /// Read the VREFCAL trim register.
    ///
    /// This register adjusts the internal voltage
    /// reference temperature compensation and is
    /// intended for advanced calibration use.
    pub fn read_vrefcal(&mut self) -> Result<u8, Error<E>> {
        let mut buf = [0u8; VREFCAL_REGISTER_BYTES];

        self.read_register(Register::VrefCal, &mut buf)?;

        Ok(buf[0])
    }

    /// Write the VREFCAL trim register.
    ///
    /// Warning: modifying this register may affect
    /// both the reference voltage value and its
    /// temperature coefficient.
    pub fn write_vrefcal(&mut self, value: u8) -> Result<(), Error<E>> {
        self.write_register(Register::VrefCal, &[value])
    }

    pub fn read_status(&mut self) -> Result<StatusReg, Error<E>> {
        let mut buf = [0u8; STATUS_REGISTER_BYTES];

        self.read_register(Register::Status, &mut buf)?;

        Ok(StatusReg::from_bytes(buf))
    }

    pub fn write_status(&mut self, status: StatusReg) -> Result<(), Error<E>> {
        self.write_register(Register::Status, &status.into_bytes())
    }

    /// Read the PHASE register value.
    ///
    /// Positive values cause Channel 0 to lag
    /// Channel 1. Negative values cause Channel 0
    /// to lead Channel 1.
    pub fn read_phase(&mut self) -> Result<i16, Error<E>> {
        let mut buf = [0u8; PHASE_REGISTER_BYTES];

        self.read_register(Register::Phase, &mut buf)?;

        let raw = u16::from_be_bytes(buf);

        let phase = (((raw & 0x0FFF) << 4) as i16) >> 4;

        Ok(phase)
    }

    /// Note:
    /// According to the MCP3911 datasheet, writing the
    /// PHASE register automatically resets and restarts
    /// both ADCs, even if the written value is unchanged.
    ///
    /// Rewriting the current PHASE value can therefore be
    /// used as a software reset for both ADCs.
    pub fn write_phase(&mut self, phase: i16) -> Result<(), Error<E>> {
        if !(-2048..=2047).contains(&phase) {
            return Err(Error::ValueOutOfRange);
        }

        let raw = (phase as u16) & 0x0FFF;

        self.write_register(Register::Phase, &raw.to_be_bytes())
    }

    pub fn read_config(&mut self) -> Result<ConfigReg, Error<E>> {
        let mut buf = [0u8; CONFIG_REGISTER_BYTES];

        self.read_register(Register::Config, &mut buf)?;

        Ok(ConfigReg::from_bytes(buf))
    }

    pub fn write_config(&mut self, config: ConfigReg) -> Result<(), Error<E>> {
        self.write_register(Register::Config, &config.into_bytes())
    }
}

// Optional Async Implementation Feature Block
#[cfg(feature = "async")]
impl<SPI, E> MCP3911<SPI>
where
    SPI: embedded_hal_async::spi::SpiDevice<Error = E>,
{
    pub async fn read_raw_async(&mut self, channel: Channel) -> Result<i32, Error<E>> {
        let reg = channel.data_register();
        let mut buf = [0u8; CHANNEL_REGISTER_BYTES];
        let cmd = ((reg as u8) << 1) | READ_BIT;
        self.spi
            .transaction(&mut [
                embedded_hal_async::spi::Operation::Write(&[cmd]),
                embedded_hal_async::spi::Operation::Read(&mut buf),
            ])
            .await
            .map_err(Error::Spi)?;
        Ok(decode_i24(&buf))
    }
}
