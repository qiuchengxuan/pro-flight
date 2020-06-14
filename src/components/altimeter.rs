use crate::datastructures::ring_buffer::RingBufferReader;
use crate::hal::sensors::Pressure;

pub struct Altimeter<'a> {
    reader: RingBufferReader<'a, Pressure>,

    altitude: i16,
    prev_altitude: i16,
    vertical_speed: i16,
}

impl<'a> Altimeter<'a> {
    pub fn new(reader: RingBufferReader<'a, Pressure>) -> Self {
        Self { reader, altitude: 0, prev_altitude: 0, vertical_speed: 0 }
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
            self.altitude = (feet as i16 + 5) / 10 * 10;
            let vertical_speed = (self.altitude - self.prev_altitude) * 20 * 60;
            let delta = (vertical_speed - self.vertical_speed) / 10;
            self.vertical_speed = (self.vertical_speed + delta) / 100 * 100;
        }
    }
}
