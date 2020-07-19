use core::num::Wrapping;

use sbus_parser::{is_sbus_packet_end, SbusData, SbusPacket, SBUS_PACKET_BEGIN, SBUS_PACKET_SIZE};

use crate::alloc;
use crate::config;
use crate::datastructures::data_source::singular::{SingularData, SingularDataSource};
use crate::datastructures::data_source::{DataSource, DataWriter};
use crate::datastructures::input::ControlInput;
use crate::datastructures::input::InputType;
use crate::datastructures::input::Receiver;

pub struct SbusReceiver {
    sequence: Wrapping<u8>,
    counter: u8,
    loss: u8,
    loss_rate: u8,
    receiver: &'static SingularData<Receiver>,
    control_input: &'static SingularData<ControlInput>,
}

#[inline]
fn to_axis(value: u16) -> i16 {
    // [0, 2047] -> [-1024, 1023] -> [-32768, 32736]
    (Wrapping(value as i16) - Wrapping(0x400)).0 << 5
}

impl SbusReceiver {
    pub fn new() -> Self {
        Self {
            sequence: Wrapping(0),
            counter: 0,
            loss: 0,
            loss_rate: 0,
            receiver: alloc::into_static(SingularData::default(), false).unwrap(),
            control_input: alloc::into_static(SingularData::default(), false).unwrap(),
        }
    }

    pub fn as_receiver(&self) -> impl DataSource<Receiver> {
        SingularDataSource::new(self.receiver)
    }

    pub fn as_control_input(&self) -> impl DataSource<ControlInput> {
        SingularDataSource::new(self.control_input)
    }

    fn handle_sbus_data(&mut self, data: &SbusData) {
        self.sequence += Wrapping(1);
        if data.frame_lost {
            self.loss += 1;
        }
        self.counter += 1;
        if self.counter == 100 {
            self.loss_rate = self.loss;
            self.counter = 0;
            self.loss = 0;
        }
        self.receiver.write(Receiver { rssi: self.loss_rate, sequence: self.sequence.0 });

        let mut control_input = ControlInput::default();
        let channels = &config::get().receiver.channels;
        for (index, option) in channels.0.iter().enumerate() {
            if index >= data.channels.len() {
                continue; // TODO: two bit channel
            }
            let value = data.channels[index];
            if let Some(input_type) = option {
                match input_type {
                    InputType::Throttle => control_input.throttle = value << 5,
                    InputType::Roll => control_input.roll = to_axis(value),
                    InputType::Pitch => control_input.pitch = to_axis(value),
                    InputType::Yaw => control_input.yaw = to_axis(value),
                }
            }
        }
        self.control_input.write(control_input);
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
        let sbus_data = packet.parse();
        self.handle_sbus_data(&sbus_data);
    }
}
