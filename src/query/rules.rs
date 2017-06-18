use byteorder::{LittleEndian, ReadBytesExt};

use ::ReadCString;
use query::{Query, do_challenge_request};
use error::{Error, Result};

use std::io::Cursor;
use std::net::ToSocketAddrs;

// cid = challenge ID
const RULES_CID_REQUEST_HEADER: [u8; 5] = [0xFF, 0xFF, 0xFF, 0xFF, // single packet header
                                           0x56]; // rules request header

#[derive(Debug)]
pub struct Rules {
    pub nb_rules: i16,
    pub rules: Vec<RuleInfo>
}

#[derive(Debug)]
pub struct RuleInfo {
    pub name: String,
    pub value: String
}

impl Query {
    pub fn rules<A: ToSocketAddrs>(&mut self, addr: A) -> Result<Rules> {
        let data = do_challenge_request(self, addr, &RULES_CID_REQUEST_HEADER[..])?;
        let mut data = Cursor::new(data);

        let header = data.read_u8()?;
        if header != 'E' as u8 {
            return Err(Error::InvalidResponse);
        }

        let nb_rules = data.read_i16::<LittleEndian>()?;

        let mut rules: Vec<RuleInfo> = Vec::with_capacity(nb_rules as usize);

        for _ in 0..nb_rules {
            rules.push(RuleInfo {
                name: data.read_cstring()?,
                value: data.read_cstring()?,
            });
        }

        Ok(Rules {
            nb_rules: nb_rules,
            rules: rules
        })
    }
}
