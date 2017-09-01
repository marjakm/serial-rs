extern crate serial_unix;
extern crate serial_core;

use std::path::Path;
use serial_unix::TTYPort;
use serial_core::{BaudRate, SerialDevice, SerialPortSettings};


fn main() {
    let port_path = ::std::env::args().skip(1).next().expect("Usage: ./custom_baud <port>");
    println!("opening port: {:?}", port_path);
    let mut port = TTYPort::open(Path::new(&port_path)).expect("Could not open port");
    println!("--------------------");
    set_baud(&mut port, BaudRate::Baud115200).expect("Could not set baud to 115200");
    println!("--------------------");
    set_baud(&mut port, BaudRate::BaudOther(250000)).expect("Could not set baud to 250000");
    println!("--------------------");
}

fn set_baud(port: &mut TTYPort, b: BaudRate) -> serial_core::Result<()> {
    let conf = port.read_settings()?;
    println!("original: {:?} {:?}", conf.baud_rate(), conf);

    let mut new_conf = conf.clone();
    new_conf.set_baud_rate(b)?;
    println!("modified: {:?} {:?}", new_conf.baud_rate(), new_conf);
    port.write_settings(&new_conf)?;

    let new_conf_read = port.read_settings()?;
    println!("reread:   {:?} {:?}", new_conf_read.baud_rate(), new_conf_read);
    port.write_settings(&conf)?;

    let restored_conf = port.read_settings()?;
    println!("restored: {:?} {:?}", restored_conf.baud_rate(), restored_conf);
    Ok(())
}
