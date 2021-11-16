use core::time::Duration;

use embedded_hal::timer::CountDown;
use sbus_parser::receiver::Receiver;

use crate::{
    config,
    datastructures::{
        control::{Control, ControlType},
        RSSI,
    },
    protocol::serial,
    sync::DataWriter,
    sys::time::TickTimer,
};

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
    inter_frame_gap: Duration,
    timer: TickTimer,
    loss_bitmap: u128,
    loss_bitmap_index: usize,
    rssi: &'a R,
    control_input: &'a C,
}

impl<'a, R: DataWriter<RSSI>, C: DataWriter<Control>> SBUS<'a, R, C> {
    pub fn new(rssi: &'a R, fast: bool, control_input: &'a C) -> Self {
        let gap = Duration::from_millis(if fast { 10 } else { 20 } - 1);
        Self {
            receiver: Receiver::new(),
            inter_frame_gap: gap,
            timer: TickTimer::default(),
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
    C: DataWriter<Control> + Sync,
{
    fn receive_size(&self) -> usize {
        1
    }

    fn receive(&mut self, bytes: &[u8]) {
        if !self.timer.wait().is_ok() {
            return;
        }
        let packet = match self.receiver.receive(bytes) {
            Some(packet) => packet,
            None => return,
        };
        self.receiver.reset();
        self.timer.start(self.inter_frame_gap);

        if packet.frame_lost {
            self.loss_bitmap |= 1u128 << self.loss_bitmap_index;
        } else {
            self.loss_bitmap &= !(1u128 << self.loss_bitmap_index);
        }
        self.loss_bitmap_index = (self.loss_bitmap_index + 1) % 100;

        let mut counter = 0;
        let mut input = Control::default();
        for (id, cfg) in config::get().receiver.inputs.0.iter() {
            let channel = cfg.channel as usize - 1;
            if channel > packet.channels.len() {
                continue;
            }
            let value = scale(packet.channels[channel], cfg.scale);
            match id {
                ControlType::Throttle => input.throttle = (value as i32 - i16::MIN as i32) as u16,
                ControlType::Roll => input.roll = value,
                ControlType::Pitch => input.pitch = value,
                ControlType::Yaw => input.yaw = value,
            }
            counter += 1;
            if counter >= 4 {
                break;
            }
        }
        self.rssi.write(100 - self.loss_bitmap.count_ones() as u16);
        self.control_input.write(input);
    }

    fn reset(&mut self) {
        self.receiver.reset();
    }
}
