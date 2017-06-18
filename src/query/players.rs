use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use ::ReadCString;
use query::Query;
use error::{Error, Result};

use std::io::{Cursor, Write};
use std::net::ToSocketAddrs;

// cid = challenge ID
const PLAYER_CID_REQUEST_HEADER: [u8; 5] = [0xFF, 0xFF, 0xFF, 0xFF, // single packet header
                                            0x55]; // player request header

#[derive(Debug)]
pub struct Players {
    pub header: u8,
    pub nb_players: u8,
    pub players: Vec<PlayerInfo>,
}

#[derive(Debug)]
pub struct PlayerInfo {
    pub index: u8,
    pub name: String,
    pub score: i32,
    pub duration: f32
}

impl Query {
    pub fn players<A: ToSocketAddrs>(&mut self, addr: A) -> Result<Players> {
        let packet = Vec::with_capacity(9);
        let mut packet = Cursor::new(packet);

        packet.write_all(&PLAYER_CID_REQUEST_HEADER[..])?;
        packet.write_i32::<LittleEndian>(-1)?;
        let data = self.send(packet.get_ref(), &addr)?;
        let mut data = Cursor::new(data);

        let header = data.read_u8()?;
        if header != 'A' as u8 {
            return Err(Error::InvalidResponse);
        }

        let challenge = data.read_i32::<LittleEndian>()?;

        packet.set_position(5);
        packet.write_i32::<LittleEndian>(challenge)?;
        let data = self.send(packet.get_ref(), &addr)?;
        let mut data = Cursor::new(data);

        let header = data.read_u8()?;
        if header != 'D' as u8 {
            return Err(Error::InvalidResponse);
        }

        let nb_players = data.read_u8()?;

        let mut players: Vec<PlayerInfo> = Vec::with_capacity(nb_players as usize);

        for _ in 0..nb_players {
            players.push(PlayerInfo {
                index: data.read_u8()?,
                name: data.read_cstring()?,
                score: data.read_i32::<LittleEndian>()?,
                duration: data.read_f32::<LittleEndian>()?,
            });
        }

        Ok(Players {
            header: header,
            nb_players: nb_players,
            players: players
        })
    }
}
