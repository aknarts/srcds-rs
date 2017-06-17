use byteorder::{LittleEndian, ReadBytesExt};

use ::ReadCString;
use query::Query;
use error::{Error, Result};

use std::io::{Cursor, ErrorKind};
use std::net::ToSocketAddrs;

const INFO_REQUEST: [u8; 25] = [0xFF, 0xFF, 0xFF, 0xFF, // single packet header
                                0x54, // info request header
                                0x53, 0x6F, 0x75, 0x72, 0x63, 0x65, 0x20, 0x45, 0x6E, 0x67, 0x69,
                                0x6E, 0x65, 0x20, 0x51, 0x75, 0x65, 0x72, 0x79, // payload
                                0x00]; // null terminator

#[derive(Debug)]
pub struct Info {
    pub header: u8,
    pub protocol: u8,
    pub name: String,
    pub map: String,
    pub folder: String,
    pub game: String,
    pub id: i16,
    pub players: u8,
    pub max_players: u8,
    pub bots: u8,
    pub server_type: Type,
    pub environment: Environment,
    pub visibility: Visibility,
    pub vac: Vac,
    pub version: String,
    pub extra_data_flag: u8,

    // optional fields, may not be present
    pub game_port: Option<i16>,
    pub steam_id: Option<u64>,
    pub sourcetv_port: Option<i16>,
    pub sourcetv_name: Option<String>,
    pub keywords: Option<String>,
    pub game_id_u64: Option<u64>

}

#[derive(Debug)]
pub enum Type {
    Dedicated,
    NonDedicated,
    Proxy
}

#[derive(Debug)]
pub enum Environment {
    Linux,
    Windows,
    Mac
}

#[derive(Debug)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug)]
pub enum Vac {
    Unsecured,
    Secured,
}

impl Query {
    pub fn info<A: ToSocketAddrs>(&mut self, addr: A) -> Result<Info> {
        let data = self.send(&INFO_REQUEST[..], addr)?;
        let mut data = Cursor::new(data);

        let header = data.read_u8()?;
        if header != 'I' as u8 {
            return Err(Error::InvalidResponse);
        }

        let mut flag = 0;

        Ok(Info {
            header: header,
            protocol: data.read_u8()?,
            name: data.read_cstring()?,
            map: data.read_cstring()?,
            folder: data.read_cstring()?,
            game: data.read_cstring()?,
            id: data.read_i16::<LittleEndian>()?,
            players: data.read_u8()?,
            max_players: data.read_u8()?,
            bots: data.read_u8()?,
            server_type: {
                match data.read_u8()? as char {
                    'd' => Type::Dedicated,
                    'i' => Type::NonDedicated,
                    'p' => Type::Proxy,
                    _ => return Err(Error::Other("Invalid server type"))
                }
            },
            environment: {
                match data.read_u8()? as char {
                    'l' => Environment::Linux,
                    'w' => Environment::Windows,
                    'm' | 'o' => Environment::Mac,
                    _ => return Err(Error::Other("Invalid environment"))
                }
            },
            visibility: {
                match data.read_u8()? {
                    0 => Visibility::Public,
                    1 => Visibility::Private,
                    _ => return Err(Error::Other("Invalid visibility"))
                }
            },
            vac: {
                match data.read_u8()? {
                    0 => Vac::Unsecured,
                    1 => Vac::Secured,
                    _ => return Err(Error::Other("Invalid VAC"))
                }
            },
            version: data.read_cstring()?,
            extra_data_flag: {
                match data.read_u8() {
                    Ok(val) => {flag = val;},
                    Err(err) => {if err.kind() != ErrorKind::UnexpectedEof { return Err(Error::Io(err)); }}
                }
                flag
            },
            game_port: {
                if (flag & 0x80) > 0 { Some(data.read_i16::<LittleEndian>()?) } else { None }
            },
            steam_id: {
                if (flag & 0x10) > 0 { Some(data.read_u64::<LittleEndian>()?) } else { None }
            },
            sourcetv_port: {
                if (flag & 0x40) > 0 { Some(data.read_i16::<LittleEndian>()?) } else { None }
            },
            sourcetv_name: {
                if (flag & 0x40) > 0 { Some(data.read_cstring()?) } else { None }
            },
            keywords: {
                if (flag & 0x20) > 0 { Some(data.read_cstring()?) } else { None }
            },
            game_id_u64: {
                if (flag & 0x01) > 0 { Some(data.read_u64::<LittleEndian>()?) } else { None }
            }
        })
    }
}
