use core::time::Duration;

use ascii::{AsciiChar, ToAsciiChar};

use crate::{
    io::{self, Read, Write},
    sys::time::TickTimer,
};

#[repr(C)]
struct Packet {
    pub packet_number: u8,
    pub packet_number_checksum: u8,
    pub packet_data: [u8; 128],
    pub crc: u16,
}

impl Packet {
    pub fn is_valid(&self) -> bool {
        if self.packet_number + self.packet_number_checksum != 0xFF {
            return false;
        }
        crc16::State::<crc16::XMODEM>::calculate(&self.packet_data).to_be() == self.crc
    }
}

pub struct XMODEM<'a> {
    stdin: &'a mut io::Stdin,
    stdout: &'a mut io::Stdout,
    first: bool,
}

impl<'a> XMODEM<'a> {
    pub fn new(stdin: &'a mut io::Stdin, stdout: &'a mut io::Stdout) -> Self {
        Self { stdin, stdout, first: true }
    }

    fn receive_packet(&mut self) -> Option<[u8; 128]> {
        let mut buffer = [0u8; core::mem::size_of::<Packet>()];
        let mut read = 0;
        while read < buffer.len() {
            if let Some(size) = self.stdin.read(&mut buffer[read..]).ok() {
                read += size;
            }
        }
        let packet: Packet = unsafe { core::mem::transmute(buffer) };
        if !packet.is_valid() {
            self.stdout.write(&[AsciiChar::NAK as u8]).ok();
            self.stdout.flush().ok();
            return None;
        }
        self.stdout.write(&[AsciiChar::ACK as u8]).ok();
        self.stdout.flush().ok();
        Some(packet.packet_data)
    }

    pub async fn receive(&mut self) -> Option<[u8; 128]> {
        let mut byte = 0u8;
        'outer: for _ in 0..10 {
            while self.stdin.read(core::slice::from_mut(&mut byte)).ok().unwrap_or(0) > 0 {
                match byte.to_ascii_char().ok().unwrap_or_default() {
                    AsciiChar::EOT | AsciiChar::CAN | AsciiChar::ETB => return None,
                    AsciiChar::SOH => break 'outer,
                    _ => continue,
                }
            }
            let read_timeout = TickTimer::after(Duration::from_secs(1));
            if self.first {
                self.stdout.write(b"C").ok();
                self.stdout.flush().ok();
            }
            read_timeout.await;
        }
        if byte != AsciiChar::SOH {
            return None;
        }
        self.first = false;
        self.receive_packet()
    }
}
