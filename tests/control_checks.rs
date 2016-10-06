/*
#![feature(clone_from_slice)]
extern crate cinch;
extern crate rand;
extern crate byteorder;
#[macro_use]
extern crate log;
extern crate env_logger;


use byteorder::{LittleEndian, ByteOrder};
use cinch::usb;
use cinch::parser;
use cinch::parser::usbr;
use cinch::modules::control_checks;


macro_rules! w_u16 {
    ($buf:expr, $val:expr) => {
        LittleEndian::write_u16(&mut $buf, $val);
    }
}

*/
