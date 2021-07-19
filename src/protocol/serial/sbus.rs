use sbus_parser::receiver::Receiver;

use crate::config;
use crate::datastructures::input::{ControlInput, InputType, RSSI};
use crate::protocol::serial;
use crate::sync::DataWriter;

#[inline]
fn to_axis(value: u16) -> i32 {
    // [0, 2047] -> [-1024, 1023] -> [-32768, 32736]
    (value as i32).wrapping_sub(0x400) << 5
}

fn scale(data: u16, scale: u8) -> i16 {
    let scaled = to_axis(data) * scale as i32 / 100;
    if scaled > i16::MAX as i32 {
        i16::MAX
    } else if scaled < i16::MIN as i32 {
        i16::MIN
    } else {
        scaled as i16
    }
}

pub struct SBUS<'a, R, C> {
    receiver: Receiver,
    loss_bitmap: u128,
    loss_bitmap_index: usize,
    rssi: &'a R,
    control_input: &'a C,
}

impl<'a, R: DataWriter<RSSI>, C: DataWriter<ControlInput>> SBUS<'a, R, C> {
    pub fn new(rssi: &'a R, control_input: &'a C) -> Self {
        Self {
            receiver: Receiver::new(),
            loss_bitmap: 0u128,
            loss_bitmap_index: 0,
            rssi,
            control_input,
        }
    }
}

impl<'a, R, C> serial::Receiver for SBUS<'a, R, C>
where
    R: DataWriter<RSSI> + Sync,
    C: DataWriter<ControlInput> + Sync,
{
    fn receive_size(&self) -> usize {
        sbus_parser::packet::SBUS_PACKET_SIZE
    }

    fn receive(&mut self, bytes: &[u8]) {
        let packet = match self.receiver.receive(bytes) {
            Some(packet) => packet,
            None => return,
        };

        if packet.frame_lost {
            self.loss_bitmap |= 1u128 << self.loss_bitmap_index;
        } else {
            self.loss_bitmap &= !(1u128 << self.loss_bitmap_index);
        }
        self.loss_bitmap_index = (self.loss_bitmap_index + 1) % 100;

        let mut counter = 0;
        let mut input = ControlInput::default();
        for (id, cfg) in config::get().receiver.inputs.0.iter() {
            let channel = cfg.channel as usize;
            if channel > packet.channels.len() {
                continue;
            }
            match id {
                InputType::Throttle => input.throttle = scale(packet.channels[channel], cfg.scale),
                InputType::Roll => input.roll = scale(packet.channels[channel], cfg.scale),
                InputType::Pitch => input.pitch = scale(packet.channels[channel], cfg.scale),
                InputType::Yaw => input.yaw = scale(packet.channels[channel], cfg.scale),
            }
            counter += 1;
            if counter >= 4 {
                break;
            }
        }
        self.rssi.write(100 - self.loss_bitmap.count_ones() as u16);
        self.control_input.write(input);
    }
}
