use core::time::Duration;

use embedded_hal::timer::CountDown;
use sbus_parser::receiver::Receiver;

use crate::{protocol::rc::RawControl, sys::time::TickTimer};

pub struct SBUS {
    receiver: Receiver,
    inter_frame_gap: Duration,
    timer: TickTimer,
    loss_bitmap: u128,
    loss_bitmap_index: usize,
}

impl SBUS {
    pub fn new(fast: bool) -> Self {
        let gap = Duration::from_millis(if fast { 10 } else { 20 } - 1);
        Self {
            receiver: Receiver::new(),
            inter_frame_gap: gap,
            timer: TickTimer::default(),
            loss_bitmap: 0u128,
            loss_bitmap_index: 0,
        }
    }
}

pub const CHUNK_SIZE: usize = 1;

#[inline]
fn to_axis(value: u16) -> i16 {
    // [0, 2047] -> [-1024, 1023] -> [-32768, 32736]
    ((value as i32).wrapping_sub(0x400) << 5) as i16
}

impl SBUS {
    pub fn receive(&mut self, bytes: &[u8]) -> Option<RawControl> {
        if !self.timer.wait().is_ok() {
            return None;
        }
        let packet = match self.receiver.receive(bytes) {
            Some(packet) => packet,
            None => return None,
        };
        self.receiver.reset();
        self.timer.start(self.inter_frame_gap);

        if packet.frame_lost {
            self.loss_bitmap |= 1u128 << self.loss_bitmap_index;
        } else {
            self.loss_bitmap &= !(1u128 << self.loss_bitmap_index);
        }
        self.loss_bitmap_index = (self.loss_bitmap_index + 1) % 100;

        let mut channels = [0i16; 18];
        for (i, &ch) in packet.channels.iter().enumerate() {
            channels[i] = to_axis(ch);
        }
        channels[16] = (packet.channel17 as u16 * u16::MAX) as i16;
        channels[17] = (packet.channel18 as u16 * u16::MAX) as i16;
        Some(RawControl { rssi: 100 - self.loss_bitmap.count_ones() as u16, channels })
    }

    pub fn reset(&mut self) {
        self.receiver.reset();
    }
}
