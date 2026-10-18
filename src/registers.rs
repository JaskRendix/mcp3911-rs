use modular_bitfield::prelude::*;

pub const CHANNEL_REGISTER_BYTES: usize = 3;
pub const CONFIG_REGISTER_BYTES: usize = 2;
pub const STATUS_REGISTER_BYTES: usize = 2;
pub const PHASE_REGISTER_BYTES: usize = 2;
pub const VREFCAL_REGISTER_BYTES: usize = 1;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Register {
    Channel0 = 0x00,
    Channel1 = 0x03,
    Modulator = 0x06,
    Phase = 0x07,
    Gain = 0x09,
    Status = 0x0a,
    Config = 0x0c,
    OffCalCh0 = 0x0e,
    GainCalCh0 = 0x11,
    OffCalCh1 = 0x14,
    GainCalCh1 = 0x17,
    VrefCal = 0x1a,
}

/// Logical channel identifier decoupled from register implementation details
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Channel {
    Ch0,
    Ch1,
}

impl Channel {
    pub const fn data_register(self) -> Register {
        match self {
            Channel::Ch0 => Register::Channel0,
            Channel::Ch1 => Register::Channel1,
        }
    }

    pub const fn gaincal_register(self) -> Register {
        match self {
            Channel::Ch0 => Register::GainCalCh0,
            Channel::Ch1 => Register::GainCalCh1,
        }
    }

    pub const fn offcal_register(self) -> Register {
        match self {
            Channel::Ch0 => Register::OffCalCh0,
            Channel::Ch1 => Register::OffCalCh1,
        }
    }
}

#[derive(Specifier, Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
#[bits = 3]
pub enum Gain {
    X1 = 0,
    X2 = 1,
    X4 = 2,
    X8 = 3,
    X16 = 4,
    X32 = 5,
}

impl Gain {
    pub const fn multiplier(self) -> u32 {
        match self {
            Gain::X1 => 1,
            Gain::X2 => 2,
            Gain::X4 => 4,
            Gain::X8 => 8,
            Gain::X16 => 16,
            Gain::X32 => 32,
        }
    }
}

#[derive(Specifier, Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
#[bits = 2]
pub enum Boost {
    X0_5 = 0,
    X0_66 = 1,
    X1 = 2,
    X2 = 3,
}

#[derive(Specifier, Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
#[bits = 3]
pub enum Oversampling {
    O32 = 0,
    O64 = 1,
    O128 = 2,
    O256 = 3,
    O512 = 4,
    O1024 = 5,
    O2048 = 6,
    O4096 = 7,
}

#[derive(Specifier, Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
#[bits = 2]
pub enum Prescale {
    Mclk1 = 0,
    Mclk2 = 1,
    Mclk4 = 2,
    Mclk8 = 3,
}

#[bitfield]
#[derive(Copy, Clone, Debug)]
pub struct ConfGain {
    pub ch0: Gain,
    pub ch1: Gain,
    pub boost: Boost,
}

#[bitfield]
#[derive(Copy, Clone, Debug)]
pub struct ConfigReg {
    pub az_on_hi_speed: AutoZero,
    pub dither: Dither,
    pub osr: Oversampling,
    pub prescale: Prescale,
    #[skip]
    _reserved0: B1,
    pub clkext: bool,
    pub vrefext: bool,
    #[skip]
    _reserved1: B1,
    pub shutdown_ch0: bool,
    pub shutdown_ch1: bool,
    pub ch0_reset: bool,
    pub ch1_reset: bool,
}

#[derive(Specifier, Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
#[bits = 2]
pub enum ReadMode {
    Static = 0,
    Group = 1,
    Type = 2,
    EntireMap = 3,
}

#[derive(Specifier, Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
#[bits = 2]
pub enum DrMode {
    Lagging = 0,
    Ch0 = 1,
    Ch1 = 2,
    Both = 3,
}

#[derive(Specifier, Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
#[bits = 2]
pub enum WordWidth {
    Both16 = 0,
    Ch0_24Ch1_16 = 1,
    Ch1_24Ch0_16 = 2,
    Both24 = 3,
}

#[derive(Specifier, Copy, Clone, Debug, PartialEq, Eq)]
#[bits = 2]
pub enum Dither {
    Off = 0,
    Low = 1,
    Medium = 2,
    High = 3,
}

#[derive(Specifier, Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
#[bits = 2]
pub enum DrStatus {
    None = 0,
    Ch0 = 1,
    Ch1 = 2,
    Both = 3,
}

#[derive(Specifier, Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
#[bits = 1]
pub enum AutoZero {
    Disabled = 0,
    Enabled = 1,
}

#[derive(Specifier, Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
#[bits = 2]
pub enum ModulatorOutput {
    Channel0 = 0,
    Channel1 = 1,
    Shared = 2,
    Disabled = 3,
}

#[bitfield]
#[derive(Copy, Clone, Debug)]
pub struct StatusReg {
    pub en_gaincal: bool,
    pub en_offcal: bool,
    pub width: WordWidth,
    pub write: bool,
    pub read: ReadMode,

    #[skip]
    _reserved0: B1,

    pub drstatus: DrStatus,
    pub drmode: DrMode,
    pub dr_hiz: bool,

    #[skip]
    _reserved1: B1,

    pub modout: ModulatorOutput,
}

/// Ergonomic configuration builder
#[derive(Copy, Clone, Debug)]
pub struct Config {
    pub gain_ch0: Gain,
    pub gain_ch1: Gain,
    pub boost: Boost,
    pub osr: Oversampling,
    pub prescale: Prescale,
    pub dither: Dither,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            gain_ch0: Gain::X1,
            gain_ch1: Gain::X1,
            boost: Boost::X1,
            osr: Oversampling::O256,
            prescale: Prescale::Mclk1,
            dither: Dither::High,
        }
    }
}

impl Config {
    pub const fn new() -> Self {
        Self {
            gain_ch0: Gain::X1,
            gain_ch1: Gain::X1,
            boost: Boost::X1,
            osr: Oversampling::O256,
            prescale: Prescale::Mclk1,
            dither: Dither::High,
        }
    }

    pub const fn gain_ch0(mut self, gain: Gain) -> Self {
        self.gain_ch0 = gain;
        self
    }

    pub const fn gain_ch1(mut self, gain: Gain) -> Self {
        self.gain_ch1 = gain;
        self
    }

    pub const fn boost(mut self, boost: Boost) -> Self {
        self.boost = boost;
        self
    }

    pub const fn osr(mut self, osr: Oversampling) -> Self {
        self.osr = osr;
        self
    }

    pub const fn prescale(mut self, prescale: Prescale) -> Self {
        self.prescale = prescale;
        self
    }

    pub const fn dither(mut self, dither: Dither) -> Self {
        self.dither = dither;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conf_gain_layout() {
        let conf = ConfGain::new().with_ch0(Gain::X8).with_ch1(Gain::X8);

        assert_eq!(conf.into_bytes(), [0b0001_1011]);
    }

    #[test]
    fn gain_boost_x2_encoding() {
        let reg = ConfGain::new().with_boost(Boost::X2);

        assert_eq!(reg.into_bytes(), [0b1100_0000]);
    }

    #[test]
    fn gain_x32_x32_encoding() {
        let reg = ConfGain::new().with_ch0(Gain::X32).with_ch1(Gain::X32);

        assert_eq!(reg.into_bytes(), [0b0010_1101]);
    }

    #[test]
    fn gain_x1_x1_encoding() {
        let reg = ConfGain::new().with_ch0(Gain::X1).with_ch1(Gain::X1);

        assert_eq!(reg.into_bytes(), [0x00]);
    }

    #[test]
    fn config_reset_both_channels_encoding() {
        let reg = ConfigReg::new().with_ch0_reset(true).with_ch1_reset(true);

        assert_eq!(reg.into_bytes(), [0x00, 0xC0]);
    }

    #[test]
    fn config_shutdown_both_channels_encoding() {
        let reg = ConfigReg::new()
            .with_shutdown_ch0(true)
            .with_shutdown_ch1(true);

        assert_eq!(reg.into_bytes(), [0x00, 0x30]);
    }

    #[test]
    fn config_prescale_mclk1_encoding() {
        let reg = ConfigReg::new().with_prescale(Prescale::Mclk1);

        assert_eq!(reg.into_bytes(), [0x00, 0x00]);
    }

    #[test]
    fn config_prescale_mclk2_encoding() {
        let reg = ConfigReg::new().with_prescale(Prescale::Mclk2);

        assert_eq!(reg.into_bytes(), [0x40, 0x00]);
    }

    #[test]
    fn config_prescale_mclk4_encoding() {
        let reg = ConfigReg::new().with_prescale(Prescale::Mclk4);

        assert_eq!(reg.into_bytes(), [0x80, 0x00]);
    }

    #[test]
    fn config_prescale_mclk8_encoding() {
        let reg = ConfigReg::new().with_prescale(Prescale::Mclk8);

        assert_eq!(reg.into_bytes(), [0xC0, 0x00]);
    }

    #[test]
    fn config_osr_encoding_roundtrip() {
        let values = [
            Oversampling::O32,
            Oversampling::O64,
            Oversampling::O128,
            Oversampling::O256,
            Oversampling::O512,
            Oversampling::O1024,
            Oversampling::O2048,
            Oversampling::O4096,
        ];

        for expected in values {
            let reg = ConfigReg::new().with_osr(expected);
            let decoded = ConfigReg::from_bytes(reg.into_bytes());

            assert_eq!(decoded.osr(), expected);
        }
    }

    #[test]
    fn config_prescale_roundtrip() {
        let values = [
            Prescale::Mclk1,
            Prescale::Mclk2,
            Prescale::Mclk4,
            Prescale::Mclk8,
        ];

        for expected in values {
            let reg = ConfigReg::new().with_prescale(expected);
            let decoded = ConfigReg::from_bytes(reg.into_bytes());

            assert_eq!(decoded.prescale(), expected);
        }
    }

    #[test]
    fn config_clkext_bit_encoding() {
        let reg = ConfigReg::new().with_clkext(true);

        assert_eq!(reg.into_bytes(), [0x00, 0x02]);
    }

    #[test]
    fn config_vrefext_bit_encoding() {
        let reg = ConfigReg::new().with_vrefext(true);

        assert_eq!(reg.into_bytes(), [0x00, 0x04]);
    }

    #[test]
    fn config_osr_256_encoding() {
        let reg = ConfigReg::new().with_osr(Oversampling::O256);

        assert_eq!(reg.into_bytes(), [0x18, 0x00]);
    }

    #[test]
    fn config_osr_4096_encoding() {
        let reg = ConfigReg::new().with_osr(Oversampling::O4096);

        assert_eq!(reg.into_bytes(), [0x38, 0x00]);
    }

    #[test]
    fn status_default_width_encoding() {
        let reg = StatusReg::new().with_width(WordWidth::Both24);

        assert_eq!(reg.width(), WordWidth::Both24);
    }

    #[test]
    fn status_read_mode_roundtrip() {
        let reg = StatusReg::new().with_read(ReadMode::EntireMap);

        let decoded = StatusReg::from_bytes(reg.into_bytes());

        assert_eq!(decoded.read(), ReadMode::EntireMap);
    }

    #[test]
    fn status_drmode_roundtrip() {
        let reg = StatusReg::new().with_drmode(DrMode::Both);

        let decoded = StatusReg::from_bytes(reg.into_bytes());

        assert_eq!(decoded.drmode(), DrMode::Both);
    }

    #[test]
    fn config_dither_roundtrip() {
        let reg = ConfigReg::new().with_dither(Dither::High);

        let decoded = ConfigReg::from_bytes(reg.into_bytes());

        assert_eq!(decoded.dither(), Dither::High);
    }

    #[test]
    fn config_dither_encoding() {
        let reg = ConfigReg::new().with_dither(Dither::Off);

        assert_eq!(reg.dither(), Dither::Off);
    }

    #[test]
    fn drstatus_roundtrip() {
        let reg = StatusReg::new().with_drstatus(DrStatus::Both);

        let decoded = StatusReg::from_bytes(reg.into_bytes());

        assert_eq!(decoded.drstatus(), DrStatus::Both);
    }

    #[test]
    fn autozero_roundtrip() {
        let reg = ConfigReg::new().with_az_on_hi_speed(AutoZero::Enabled);

        let decoded = ConfigReg::from_bytes(reg.into_bytes());

        assert_eq!(decoded.az_on_hi_speed(), AutoZero::Enabled);
    }
}
