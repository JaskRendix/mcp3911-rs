//! Basic MCP3911 usage example.
//!
//! This example shows the typical configuration flow:
//! 1. Create the driver
//! 2. Reset the device
//! 3. Configure gain and sampling settings
//! 4. Read a conversion result
//! 5. Convert counts to microvolts

use mcp3911_rs::registers::{Boost, Channel, Config, Gain, Oversampling, Prescale, sample_rate_hz};

fn main() {
    // Replace this with your platform-specific SPI device.
    //
    // let spi = ...;
    // let mut adc = MCP3911::new(spi, 1200);
    //
    // The code below shows the intended usage pattern.

    println!("MCP3911 example");

    // Example configuration:
    let config = Config::default()
        .gain_ch0(Gain::X8)
        .gain_ch1(Gain::X8)
        .boost(Boost::X1)
        .osr(Oversampling::O256)
        .prescale(Prescale::Mclk1);

    let sample_rate = sample_rate_hz(4_000_000, Oversampling::O256, Prescale::Mclk1);

    println!("Configured sample rate: {} SPS", sample_rate);

    // Typical embedded usage:
    //
    // adc.reset()?;
    // adc.configure(config)?;
    //
    // let raw = adc.read_raw(Channel::Ch0)?;
    // let uv = adc.counts_to_uv(raw, Gain::X8);
    //
    // println!("CH0: {} counts ({} µV)", raw, uv);

    let _ = config;
    let _ = Channel::Ch0;
}
