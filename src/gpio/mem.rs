
use std::fmt;
use std::fs::OpenOptions;
use std::io;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use libc::{self, c_void, off_t, size_t, MAP_FAILED, MAP_SHARED, O_SYNC, PROT_READ, PROT_WRITE};

use crate::gpio::{Error, Result, Mode};
use crate::system::{DeviceInfo, SoC};

macro_rules! my_match {
   ($obj:expr, $($matcher:pat $(if $pred:expr)* => $result:expr),*) => {
       match $obj {
           $($matcher $(if $pred)* => $result),*
       }
   }
}

macro_rules! to_mode {
    ($val:expr) => {{

    }};
}

pub const PATH_DEV_GPIOMEM: &str = "/dev/gpiomem";
const PATH_DEV_MEM: &str = "/dev/mem";
// The BCM2835 has 41 32-bit registers related to the GPIO (datasheet @ 6.1).
// The BCM2711 (RPi4) has 58 32-bit registers related to the GPIO.
const GPIO_MEM_REGISTERS: usize = 58;
const GPIO_MEM_SIZE: usize = GPIO_MEM_REGISTERS * std::mem::size_of::<u32>();
const GPFSEL0: usize = 0x00;
const GPSET0: usize = 0x1c / std::mem::size_of::<u32>();
const GPCLR0: usize = 0x28 / std::mem::size_of::<u32>();
const GPLEV0: usize = 0x34 / std::mem::size_of::<u32>();
const GPPUD: usize = 0x94 / std::mem::size_of::<u32>();
const GPPUDCLK0: usize = 0x98 / std::mem::size_of::<u32>();
// Only available in BCM2711 (RPi4).
const GPPUD_CNTRL_REG0: usize = 0xe4 / std::mem::size_of::<u32>();


pub struct GpioMem {
    mem_ptr: *mut u32,
    soc: SoC,
}
impl fmt::Debug for GpioMem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GpioMem")
            .field("mem_ptr", &self.mem_ptr)
            .field("soc", &self.soc)
            .finish()
    }
}

impl fmt::Display for GpioMem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GpioMem")
            .field("mem_ptr", &self.mem_ptr)
            .field("soc", &self.soc)
            .finish()
    }
}

impl GpioMem {
    pub(crate) fn open() -> Result<GpioMem> {
        if let Ok(mem_ptr) = GpioMem::map_devgpiomem() {
            Ok(
                GpioMem{
                    mem_ptr: mem_ptr,
                    soc: SoC::Bcm2711,
                }
            )
        } else {
            Err(Error::Io(io::Error::last_os_error()))
        }

    }
    pub(crate) fn mode(&self, pin: u8) -> Mode {

        let offset = pin / 10;
        let reg_value = self.read(0);
        let shift = (pin % 10) * 3;
        //println!(" reg_value: {:#034b} shift: {}", reg_value, shift);
        let mode = unsafe { std::mem::transmute((reg_value >> shift) as u8 & 0b111) };
        mode
    }

    pub(crate) fn set_mode(&self, pin: u8, mode: Mode) {
        let offset = (pin / 10) as usize;
        let reg_value = self.read(0);
        let shift = (pin % 10) * 3;
        self.write(
            offset,
            (reg_value & !(0b111 << shift)) | ((mode as u32) << shift),
        );

    }

    /// this is not a register, so just setting bit has no affect on other pins
    ///    1098_7654_3210_9876_5432_1098_7654_3210
    /// e.g 0000_0000_0000_0000_0000_0001_0000_0000 would clear 8th pin only.
    pub(crate) fn set_low(&self, pin: u8) {
        let offset = GPCLR0 + ((pin / 32) as usize);
        let shift = pin % 32;
        let value = 1 << shift;
        self.write(offset,  value);
    }

    /// this is not a register, so just setting bit has no affect on other pins
    ///    1098_7654_3210_9876_5432_1098_7654_3210
    /// e.g 0000_0000_0000_0000_0000_0001_0000_0000 would clear 8th pin only.
    pub(crate) fn set_high(&self, pin: u8) {
        let offset = GPSET0 + pin as usize / 32;
        let shift = pin % 32;
        self.write(offset, 1 << shift);
    }

    fn map_devgpiomem() -> Result<*mut u32> {
        // Open /dev/gpiomem with read/write/sync flags. This might fail if
        // /dev/gpiomem doesn't exist (< Raspbian Jessie), or /dev/gpiomem
        // doesn't have the appropriate permissions, or the current user is
        // not a member of the gpio group.
        let gpiomem_file = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(O_SYNC)
            .open(PATH_DEV_GPIOMEM)?;

        // Memory-map /dev/gpiomem at offset 0
        let gpiomem_ptr = unsafe {
            libc::mmap(
                ptr::null_mut(),
                GPIO_MEM_SIZE,
                PROT_READ | PROT_WRITE,
                MAP_SHARED,
                gpiomem_file.as_raw_fd(),
                0,
            )
        };

        Ok(gpiomem_ptr as *mut u32)
    }
    #[inline(always)]
    fn read(&self, offset: usize) -> u32 {
        unsafe { ptr::read_volatile(self.mem_ptr.add(offset)) }
    }

    #[inline(always)]
    fn write(&self, offset: usize, value: u32) {
        unsafe {
            ptr::write_volatile(self.mem_ptr.add(offset), value);
        }
    }
}