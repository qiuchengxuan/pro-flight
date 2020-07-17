use core::sync::atomic::{AtomicUsize, Ordering};

use sbus_parser::{is_sbus_packet_end, SbusData, SbusPacket, SBUS_PACKET_BEGIN, SBUS_PACKET_SIZE};

use crate::config::receiver::{Channels, MAX_CHANNEL};
use crate::datastructures::input::InputType;
use crate::datastructures::input::{Pitch, Roll, Throttle, Yaw};
use crate::hal::input::BasicInput;
use crate::hal::receiver::Receiver;

#[derive(Default, Debug)]
pub struct SbusReceiver {
    sequence: AtomicUsize,
    data: SbusData,
    counter: u8,
    loss: u8,
    loss_rate: u8,
    channel_mapping: [u8; MAX_CHANNEL],
}

impl SbusReceiver {
    pub fn set_mapping(&mut self, config: &Channels) {
        for (index, &option) in config.0.iter().enumerate() {
            if let Some(input_type) = option {
                self.channel_mapping[input_type as usize] = index as u8;
            }
        }
    }

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

    #[inline]
    fn channel_index(&self, input_type: InputType) -> usize {
        self.channel_mapping[input_type as usize] as usize
    }
}

impl BasicInput for SbusReceiver {
    fn get_throttle(&self) -> Throttle {
        self.data.channels[self.channel_index(InputType::Throttle)] << 5
    }

    fn get_roll(&self) -> Roll {
        self.data.channels[self.channel_index(InputType::Roll)] as i16 - (1 << 10)
    }

    fn get_pitch(&self) -> Pitch {
        self.data.channels[self.channel_index(InputType::Pitch)] as i16 - (1 << 10)
    }

    fn get_yaw(&self) -> Yaw {
        self.data.channels[self.channel_index(InputType::Yaw)] as i16 - (1 << 10)
    }
}

impl Receiver for SbusReceiver {
    fn rssi(&self) -> u8 {
        self.loss_rate
    }

    fn get_sequence(&self) -> usize {
        self.sequence.load(Ordering::Relaxed)
    }
}
