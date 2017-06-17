//! srcds is a Rust library for Source server queries and RCON.

extern crate byteorder;

pub mod query;
pub mod error;

use error::Result;
use std::io::{Cursor, Read};

trait ReadCString {
    fn read_cstring(&mut self) -> Result<String>;
}

impl ReadCString for Cursor<Vec<u8>> {
    fn read_cstring(&mut self) -> Result<String> {
        let mut buf = [0; 1];
        let mut str_vec = Vec::with_capacity(256);
        loop {
            self.read(&mut buf)?;
            if buf[0] == 0 { break; } else { str_vec.push(buf[0]); }
        }
        Ok(unsafe {String::from_utf8_unchecked(str_vec)})
    }
}