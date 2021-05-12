#[macro_use]
pub mod macros;
mod system;
mod gpio;

use crate::system::DeviceInfo;
use crate::gpio::mem::{GpioMem, PATH_DEV_GPIOMEM };
use crate::gpio::pin::{OutputPin, Pin};

use std::rc::Rc;
use std::thread;
use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let dev_info = DeviceInfo::new();
    println!("{}", dev_info.unwrap());
    if let Ok(gpio_mem) =GpioMem::open(){
        println!("gpio_mem: {}", gpio_mem);
        let gpio_rc = Rc::new(gpio_mem);
        let mut out_pin3 = Pin::new(3,gpio_rc).into_output();
        while  running.load(Ordering::SeqCst){
            println!("led on");
            out_pin3.set_high();
            thread::sleep(Duration::from_millis(2000));
            out_pin3.set_low();
            println!("led off");
            thread::sleep(Duration::from_millis(2000));
        }

    } else {
        println!("ERROR: unable to map {:?}",PATH_DEV_GPIOMEM);
    }

}
