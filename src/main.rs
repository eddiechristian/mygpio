#[macro_use]
pub mod macros;
mod system;
mod gpio;

use crate::system::DeviceInfo;
use crate::gpio::mem::{GpioMem, PATH_DEV_GPIOMEM };
use crate::gpio::pin::{OutputPin, Pin};
use std::rc::Rc;

fn main() {
    let dev_info = DeviceInfo::new();
    println!("{}", dev_info.unwrap());
    if let Ok(gpio_mem) =GpioMem::open(){
        println!("gpio_mem: {}", gpio_mem);
        let gpio_rc = Rc::new(gpio_mem);
        let out_pin5 = Pin::new(27,gpio_rc).into_output();
    } else {
        println!("ERROR: unable to map {:?}",PATH_DEV_GPIOMEM);
    }

}
