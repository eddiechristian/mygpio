use std::error;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::result;

const PERIPHERAL_BASE_RPI4: u32 = 0xfe00_0000;
const GPIO_OFFSET: u32 = 0x20_0000;

/// Errors that can occur when trying to identify the Raspberry Pi hardware.
#[derive(Debug)]
pub enum Error {
    /// Unknown model.
    ///
    /// `DeviceInfo` was unable to identify the Raspberry Pi model based on the
    /// contents of `/proc/cpuinfo`, `/sys/firmware/devicetree/base/compatible`
    /// and `/sys/firmware/devicetree/base/model`.
    ///
    /// Support for new models is usually added shortly after they are officially
    /// announced and available to the public. Make sure you're using the latest
    /// release of RPPAL.
    ///
    /// You may also encounter this error if your Linux distribution
    /// doesn't provide any of the common user-accessible system files
    /// that are used to identify the model and SoC.
    UnknownModel,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::UnknownModel => write!(f, "Unknown Raspberry Pi model"),
        }
    }
}

impl error::Error for Error {}

/// Result type returned from methods that can have `system::Error`s.
pub type Result<T> = result::Result<T, Error>;


#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Model {
    RaspberryPi4ModelB,
}

impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Model::RaspberryPi4ModelB => write!(f, "RaspberryPi4ModelB"),
        }
    }
}


/// Identifiable Raspberry Pi SoCs.
///
/// `SoC` might be extended with additional variants in a minor or
/// patch revision, and must not be exhaustively matched against.
/// Instead, add a `_` catch-all arm to match future variants.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum SoC {
    Bcm2711,
}

impl fmt::Display for SoC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            SoC::Bcm2711 => write!(f, "BCM2711"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct DeviceInfo {
    model: Model,
    soc: SoC,
    peripheral_base: u32,
    gpio_offset: u32,
}

impl fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(model:{:?} soc: {:?} peripheral_base: {:#02X} gpio_offset: {:#02X})",
               self.model, self.soc, self.peripheral_base, self.gpio_offset)
    }
}

impl DeviceInfo {
    fn parse_proc_cpuinfo() -> Result<DeviceInfo> {
        let proc_file = File::open("/proc/cpuinfo");
        let proc_cpuinfo = BufReader::new(proc_file.unwrap());
        let mut hardware: String = String::new();
        let mut revision: String = String::new();
        for line_result in proc_cpuinfo.lines() {
            if let Ok(line) = line_result {
                if let Some(line_value) = line.strip_prefix("Hardware\t: ") {
                    hardware = String::from(line_value);
                } else if let Some(line_value) = line.strip_prefix("Revision\t: ") {
                    revision = String::from(line_value).to_lowercase();
                }
            }
        }
        match &hardware[..] {
            "BCM2711" => {}
            _ => return Err(Error::UnknownModel),
        }
        let model = {
            match &revision[..] {
                "d03114" => Model::RaspberryPi4ModelB,
                _ => return Err(Error::UnknownModel),
            }
        };
        Ok(
            {
                DeviceInfo {
                    model: model,
                    soc: SoC::Bcm2711,
                    peripheral_base: PERIPHERAL_BASE_RPI4,
                    gpio_offset: GPIO_OFFSET,
                }
            }
        )

    }

    pub fn new() -> Result<DeviceInfo> {
        let ret =DeviceInfo::parse_proc_cpuinfo()?;
        Ok(ret)
    }
}