use alloc::boxed::Box;
use alloc::rc::Rc;

use sbus_parser::{is_sbus_packet_end, SbusData, SbusPacket, SBUS_PACKET_BEGIN, SBUS_PACKET_SIZE};

use crate::components::event::Notify;
use crate::config;
use crate::datastructures::data_source::singular::{SingularData, SingularDataSource};
use crate::datastructures::data_source::{DataSource, DataWriter};
use crate::datastructures::input::ControlInput;
use crate::datastructures::input::InputType;
use crate::datastructures::input::Receiver;

pub struct SbusReceiver {
    sequence: u8,
    counter: u8,
    loss: u8,
    loss_rate: u8,
    receiver: Rc<SingularData<Receiver>>,
    control_input: Rc<SingularData<ControlInput>>,
    notify: Option<Box<dyn Notify>>,
}

#[inline]
fn to_axis(value: u16) -> i32 {
    // [0, 2047] -> [-1024, 1023] -> [-32768, 32736]
    (value as i32).wrapping_sub(0x400) << 5
}

impl SbusReceiver {
    pub fn new() -> Self {
        Self {
            sequence: 0,
            counter: 0,
            loss: 0,
            loss_rate: 0,
            receiver: Rc::new(SingularData::default()),
            control_input: Rc::new(SingularData::default()),
            notify: None,
        }
    }

    pub fn as_receiver(&self) -> impl DataSource<Receiver> {
        SingularDataSource::new(&self.receiver)
    }

    pub fn as_control_input(&self) -> impl DataSource<ControlInput> {
        SingularDataSource::new(&self.control_input)
    }

    pub fn set_notify(&mut self, notify: Box<dyn Notify>) {
        self.notify = Some(notify);
    }

    fn handle_sbus_data(&mut self, data: &SbusData) {
        self.sequence = self.sequence.wrapping_add(1);
        self.loss += data.frame_lost as u8;
        self.counter += 1;
        if self.counter == 100 {
            self.loss_rate = self.loss;
            self.counter = 0;
        }
        self.receiver.write(Receiver { rssi: 100 - self.loss_rate, sequence: self.sequence });

        let mut control_input = ControlInput::default();
        let channels = &config::get().receiver.channels;
        for (index, option) in channels.0.iter().enumerate() {
            if index >= data.channels.len() {
                continue; // TODO: two bit channel
            }
            if let Some(channel) = option {
                let scaled = to_axis(data.channels[index]) * channel.scale as i32 / 100;
                let scaled = if scaled > i16::MAX as i32 {
                    i16::MAX
                } else if scaled < i16::MIN as i32 {
                    i16::MIN
                } else {
                    scaled as i16
                };
                match channel.input_type {
                    InputType::Throttle => control_input.throttle = scaled,
                    InputType::Roll => control_input.roll = scaled,
                    InputType::Pitch => control_input.pitch = scaled,
                    InputType::Yaw => control_input.yaw = scaled,
                }
            }
        }
        self.control_input.write(control_input);
        if let Some(ref mut notify) = self.notify {
            notify.notify()
        }
    }

    pub fn handle(&mut self, ring: &[u8], half: bool) {
        let begin = if half { 0 } else { ring.len() / 2 };
        let end = if half { ring.len() / 2 } else { ring.len() };
        let mut offset = usize::MAX;
        let mut packet = [0u8; 1 + SBUS_PACKET_SIZE];
        for i in begin..end {
            if !is_sbus_packet_end(ring[i]) {
                continue;
            }
            let index = (i + ring.len() - SBUS_PACKET_SIZE) % ring.len();
            if ring[index] == SBUS_PACKET_BEGIN {
                offset = index;
                break;
            }
        }
        if offset == usize::MAX {
            return;
        }
        for i in 0..SBUS_PACKET_SIZE {
            packet[1 + i] = ring[(offset + i) % ring.len()];
        }
        let packet = SbusPacket::from_bytes(&packet).unwrap();
        let sbus_data = packet.parse();
        self.handle_sbus_data(&sbus_data);
    }
}
