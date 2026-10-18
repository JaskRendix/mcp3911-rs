# MCP3911-rs

A modern, `no_std`, `embedded-hal` 1.0 compatible Rust driver for the Microchip MCP3911 dual-channel 24-bit Analog Front End (AFE).

This crate is a Rust reimplementation and modernization of the original C++ driver by Bernhard Schneider: [https://github.com/bersch/MCP3911](https://github.com/bersch/MCP3911).

## Features

* **embedded-hal 1.0 Integration**: Uses generic `SpiDevice` traits for hardware-agnostic communication across STM32, RP2040, ESP32, and other microcontrollers.
* **Integer-Only Voltage Conversion**: Uses integer arithmetic and returns microvolts (`i64`), avoiding floating-point overhead on MCUs without hardware FPUs.
* **Timing Utilities**: Compute AMCLK, DMCLK and ADC sample rates from MCP3911 clock configuration settings.
* **Type-Safe Bitfields**: Utilizes `modular-bitfield` to map configuration registers, gains, and oversampling ratios (OSR) to strongly typed enums instead of raw bitmasks.
* **Ergonomic Configuration Builder**: Provides a clean builder pattern for setting device parameters.
* **Optional Async Support**: Includes a feature flag for `embedded-hal-async` compatibility.
* **Tested & Reliable**: Includes unit-tested bitfield mappings, 24-bit sign extension edge cases, and SPI mock test support.
* **Calibration Support**: Read and write channel offset and gain calibration registers (`OFFCAL_CHn`, `GAINCAL_CHn`) as well as the advanced `VREFCAL` register.
* **Typed Register Access**: Strongly-typed `ConfigReg`, `StatusReg`, gain, oversampling, prescaler, DR mode, and width abstractions instead of raw bit manipulation.
* **Status Register Support**: Read and modify `STATUSCOM` settings through the typed `StatusReg` abstraction.
* **Phase Control**: Read and write the signed 12-bit `PHASE` register and perform ADC soft resets through `soft_reset_adcs()`.

## Installation

This crate is currently not published on crates.io.

To use the latest version directly from GitHub:

```toml
[dependencies]
mcp3911-rs = { git = "https://github.com/JaskRendix/mcp3911" }
```

## Quick Start & Usage Example

```rust
use mcp3911::{
    MCP3911,
    registers::{
        Boost,
        Channel,
        Config,
        Gain,
        Oversampling,
        Prescale,
        sample_rate_hz,
    },
};

// Initialize with an embedded-hal SpiDevice and a 1.2V reference.
let mut adc = MCP3911::new(spi, 1200);

// Reset and configure the device.
adc.reset()?;

adc.configure(
    Config::default()
        .gain_ch0(Gain::X8)
        .gain_ch1(Gain::X8)
        .boost(Boost::X1)
        .osr(Oversampling::O256)
        .prescale(Prescale::Mclk1),
)?;

// Read Channel 0.
let raw = adc.read_raw(Channel::Ch0)?;

// Convert to microvolts.
let uv = adc.counts_to_uv(raw, Gain::X8);

// Calculate the resulting sample rate.
let sample_rate =
    sample_rate_hz(4_000_000, Oversampling::O256, Prescale::Mclk1);
```

## Credits

Inspired by the original MCP3911 C++ driver by Bernhard Schneider and reimplemented in idiomatic Rust using embedded-hal ([bersch/MCP3911](https://github.com/bersch/MCP3911)).

## Documentation

* MCP3911 Datasheet (Microchip): [https://ww1.microchip.com/downloads/aemDocuments/documents/MSLD/ProductDocuments/DataSheets/MCP3911-3.3V-Two-Channel-Analog-Front-End-20002286E.pdf](https://ww1.microchip.com/downloads/aemDocuments/documents/MSLD/ProductDocuments/DataSheets/MCP3911-3.3V-Two-Channel-Analog-Front-End-20002286E.pdf)
* MCP3911 Product Page (Microchip): [https://www.microchip.com/en-us/product/MCP3911](https://www.microchip.com/en-us/product/MCP3911)
