use crate::datastructures::ring_buffer::RingBufferReader;
use crate::hal::sensors::Pressure;

pub struct Altimeter<'a> {
    reader: RingBufferReader<'a, Pressure>,

    altitude: i16, // dimensionless
    prev_altitude: i16,
    vertical_speed: i16, // per minute
    rate: u16,           // hz
}

fn round_up_10(value: i16) -> i16 {
    (value + 5) / 10 * 10
}

const SECONDS_PER_MINUTE: i16 = 60;

impl<'a> Altimeter<'a> {
    pub fn new(reader: RingBufferReader<'a, Pressure>, rate: u16) -> Self {
        Self { reader, altitude: 0, prev_altitude: 0, vertical_speed: 0, rate }
    }

    pub fn altitude(&self) -> i16 {
        self.altitude
    }

    pub fn vertical_speed(&self) -> i16 {
        self.vertical_speed
    }

    pub fn update(&mut self) {
        if let Some(value) = self.reader.read_latest() {
            self.prev_altitude = self.altitude;
            let feet = value.to_sea_level_altitude().as_feet();
            self.altitude = round_up_10(feet as i16);
            let delta = self.altitude - self.prev_altitude;
            let rate = self.rate as i16;
            let speed = self.vertical_speed * (rate - 1) / rate + delta * SECONDS_PER_MINUTE;
            self.vertical_speed = speed * 100 / 100;
        }
    }
}
