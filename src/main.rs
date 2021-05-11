use crate::system::DeviceInfo;

#[macro_use]
pub mod macros;
mod system;
mod gpio;

fn main() {
    let dev_info = DeviceInfo::new();
    println!("{}", dev_info.unwrap());

}
