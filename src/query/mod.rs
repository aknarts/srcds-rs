use byteorder::{ByteOrder, LittleEndian, ReadBytesExt, WriteBytesExt};

use std::net::{UdpSocket, ToSocketAddrs};
use std::time::Duration;
use std::io::{Cursor, Write};

use error::{Error, Result};

mod info;
mod players;
mod rules;

const SINGLE_PACKET: i32 = -1;
const MULTI_PACKET: i32 = -2;

struct UnorderedPacket {
    number: u8,
    payload: Vec<u8>
}

pub struct Query {
    socket: UdpSocket
}

impl Query {
    pub fn new() -> Result<Query> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_read_timeout(Some(Duration::new(5, 0)))?;
        socket.set_write_timeout(Some(Duration::new(5, 0)))?;

        Ok(Query {
            socket: socket
        })
    }
    fn send<A: ToSocketAddrs>(&self, send: &[u8], addr: A) -> Result<Vec<u8>> {
        self.socket.send_to(send, addr)?;

        let mut header = [0; 4];
        self.socket.peek_from(&mut header)?;
        let header = LittleEndian::read_i32(&header);

        if header == SINGLE_PACKET {
            let mut data = vec![0; 1400];
            let read = self.socket.recv(&mut data)?;
            // discard first 4 header bytes
            data.remove(0); data.remove(0); data.remove(0); data.remove(0);
            data.truncate(read);
            Ok(data)
        } else if header == MULTI_PACKET {
            // peek the first 12 bytes to learn at which size the packets are split
            let mut data = [0; 12];
            self.socket.peek(&mut data)?;

            let id = LittleEndian::read_i32(&data[4..8]);
            let total_packets: usize = data[8] as usize;
            let switching_size: usize = LittleEndian::read_i16(&data[10..12]) as usize;

            let mut all_packets: Vec<UnorderedPacket> = Vec::with_capacity(total_packets);

            loop {
                let mut data = vec![0; switching_size];
                let read = self.socket.recv(&mut data)?;
                if read < data.len() {data.truncate(read)};

                // check the id
                let local_id = LittleEndian::read_i32(&data[4..8]);
                if local_id != id { return Err(Error::Other("Subsequent packet IDs don't match")); }

                all_packets.push(UnorderedPacket {
                    number: data[10],
                    payload: Vec::from(&data[12..])
                });

                if (all_packets.len()) == total_packets { break; }
            }

            // now we reconstruct the packet

            // first let's sort them by order number
            all_packets.sort_by_key(|p| p.number);

            // now just concatenate each packet together
            let mut joined = Vec::with_capacity(total_packets * 1400);
            for p in all_packets {
                joined.extend(p.payload);
            }

            // discard first 4 header bytes
            joined.remove(0); joined.remove(0); joined.remove(0); joined.remove(0);

            Ok(joined)
        } else {
            Err(Error::Other("Unknown packet header"))
        }
    }
}

fn do_challenge_request<A: ToSocketAddrs>(query: &mut Query,
                                          addr: A, header: &[u8]) -> Result<Vec<u8>> {
    let packet = Vec::with_capacity(9);
    let mut packet = Cursor::new(packet);

    packet.write_all(header)?;
    packet.write_i32::<LittleEndian>(-1)?;
    let data = query.send(packet.get_ref(), &addr)?;
    let mut data = Cursor::new(data);

    let header = data.read_u8()?;
    if header != 'A' as u8 {
        return Err(Error::InvalidResponse);
    }

    let challenge = data.read_i32::<LittleEndian>()?;

    packet.set_position(5);
    packet.write_i32::<LittleEndian>(challenge)?;
    let data = query.send(packet.get_ref(), &addr)?;

    Ok(data)
}