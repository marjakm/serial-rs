extern crate serial;

use std::env;
use std::fmt::Debug;

use serial::prelude::*;
use serial::{BaudRate, SerialDevice};

fn main() {
    for arg in env::args_os().skip(1) {
        println!("opening port: {:?}", arg);
        let mut port = serial::open(&arg).unwrap();
        println!("--------------------");
        set_baud(&mut port, BaudRate::Baud115200).unwrap();
        println!("--------------------");
        set_baud(&mut port, BaudRate::BaudOther(250000)).unwrap();
        println!("--------------------");
    }
}

fn set_baud<T>(port: &mut T, b: BaudRate) -> serial::Result<()> where
    T: SerialPort+SerialDevice,
    <T as serial::SerialDevice>::Settings: Clone+Debug
{
    let conf = try!(port.read_settings());
    println!("original: {:?} {:?}", conf.baud_rate(), conf);

    let mut new_conf = conf.clone();
    try!(new_conf.set_baud_rate(b));
    println!("modified: {:?} {:?}", new_conf.baud_rate(), new_conf);
    try!(port.write_settings(&new_conf));

    let new_conf_read = try!(port.read_settings());
    println!("reread:   {:?} {:?}", new_conf_read.baud_rate(), new_conf_read);
    try!(port.write_settings(&conf));

    let restored_conf = try!(port.read_settings());
    println!("restored: {:?} {:?}", restored_conf.baud_rate(), restored_conf);
    Ok(())
}
