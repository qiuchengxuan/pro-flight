use crate::alloc;
use crate::datastructures::data_source::singular::{SingularData, SingularDataSource};
use crate::datastructures::data_source::{DataSource, DataWriter};
use crate::datastructures::measurement::{Altitude, DistanceUnit, Pressure, Velocity};
use crate::datastructures::schedule::{Hertz, Schedulable};

const SECONDS_PER_MINUTE: i16 = 60;
const MAX_RECORDS: usize = 25;

pub struct Altimeter<D> {
    data_source: D,
    data: &'static SingularData<(Altitude, Velocity)>,

    records: &'static mut [i16],
    rate: u16, // hz
    counter: u8,
}

impl<D: DataSource<Pressure>> Altimeter<D> {
    pub fn new(data_source: D, rate: u16) -> Self {
        let data = alloc::into_static(SingularData::default(), false).unwrap();
        let mut size = MAX_RECORDS;
        for i in 0..16 {
            size = (rate >> i) as usize;
            if (size & 1 > 0) || size <= MAX_RECORDS {
                break;
            }
        }
        let records = alloc::typed_allocate(0, size, false).unwrap();
        Self { data_source, data, records, rate, counter: 0 }
    }

    pub fn as_data_source(&self) -> impl DataSource<(Altitude, Velocity)> {
        SingularDataSource::new(&self.data)
    }
}

impl<D: DataSource<Pressure>> Schedulable for Altimeter<D> {
    fn schedule(&mut self) {
        if let Some(value) = self.data_source.read_last() {
            let altitude: Altitude = value.into();
            let meters = altitude.convert(DistanceUnit::CentiMeter, DistanceUnit::Meter, 1) as i16;
            self.records[self.counter as usize] = meters;
            self.counter = (self.counter + 1) % self.rate as u8;
            let delta = meters - self.records[self.counter as usize];
            self.data.write((altitude, delta * SECONDS_PER_MINUTE))
        }
    }

    fn rate(&self) -> Hertz {
        50
    }
}
