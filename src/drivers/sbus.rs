use core::sync::atomic::{AtomicUsize, Ordering};

use sbus_parser::{is_sbus_packet_end, SbusData, SbusPacket, SBUS_PACKET_BEGIN, SBUS_PACKET_SIZE};

use crate::hal::receiver::{Receiver, ReceiverInput};

#[derive(Default, Debug)]
pub struct SbusReceiver {
    sequence: AtomicUsize,
    data: SbusData,
    counter: u8,
    loss: u8,
    loss_rate: u8,
}

impl SbusReceiver {
    pub fn handle(&mut self, ring: &[u8], offset: usize, num_bytes: usize) {
        let mut index = SBUS_PACKET_SIZE;
        let mut packet = [0u8; 1 + SBUS_PACKET_SIZE];
        for i in 0..num_bytes {
            if !is_sbus_packet_end(ring[(offset + i) % ring.len()]) {
                continue;
            }
            let start_index = (offset + i + ring.len() - SBUS_PACKET_SIZE) % ring.len();
            if ring[start_index] == SBUS_PACKET_BEGIN {
                index = start_index;
                break;
            }
        }
        if index == SBUS_PACKET_SIZE {
            return;
        }
        for i in 0..SBUS_PACKET_SIZE {
            packet[1 + i] = ring[(index + i) % ring.len()];
        }
        let packet = SbusPacket::from_bytes(&packet).unwrap();
        self.data = packet.parse();
        self.sequence.fetch_add(1, Ordering::Relaxed);
        if self.data.frame_lost {
            self.loss += 1;
        }
        self.counter += 1;
        if self.counter == 100 {
            self.loss_rate = self.loss;
            self.counter = 0;
            self.loss = 0;
        }
    }
}

impl Receiver for SbusReceiver {
    fn rssi(&self) -> u8 {
        self.loss_rate
    }

    fn get_sequence(&self) -> usize {
        self.sequence.load(Ordering::Relaxed)
    }

    fn get_input(&self) -> ReceiverInput {
        ReceiverInput {
            throttle: self.data.channels[2] << 5,
            roll: (self.data.channels[0] << 5) as i16,
            pitch: (self.data.channels[1] << 5) as i16,
            yaw: (self.data.channels[3] << 5) as i16,
        }
    }
}
