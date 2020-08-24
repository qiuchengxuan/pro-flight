use alloc::rc::Rc;
use alloc::vec::Vec;

use crate::components::schedule::{Rate, Schedulable};
use crate::datastructures::data_source::singular::{SingularData, SingularDataSource};
use crate::datastructures::data_source::{DataWriter, OptionData};
use crate::datastructures::measurement::distance::Distance;
use crate::datastructures::measurement::unit::Meter;
use crate::datastructures::measurement::{Altitude, Pressure, Velocity};

const SECONDS_PER_MINUTE: i16 = 60;
const MAX_RECORDS: usize = 25;

pub struct Altimeter<B> {
    barometer: B,
    data: Rc<SingularData<(Altitude, Velocity<i16, Meter>)>>,

    records: Vec<Distance<i16, Meter>>,
    rate: Rate,
    counter: u8,
}

impl<B: OptionData<Pressure>> Altimeter<B> {
    pub fn new(barometer: B, rate: Rate) -> Self {
        let data = Rc::new(SingularData::default());
        let mut size = MAX_RECORDS;
        for i in 0..16 {
            size = (rate >> i) as usize;
            if (size & 1 > 0) || size <= MAX_RECORDS {
                break;
            }
        }
        let records = vec![Distance::new(0, Meter); size];
        Self { barometer, data, records, rate, counter: 0 }
    }

    pub fn reader(&self) -> SingularDataSource<(Altitude, Velocity<i16, Meter>)> {
        SingularDataSource::new(&self.data)
    }
}

impl<D: OptionData<Pressure>> Schedulable for Altimeter<D> {
    fn schedule(&mut self) -> bool {
        if let Some(value) = self.barometer.read() {
            let altitude: Altitude = value.into();
            let meters = altitude.to_unit(Meter).convert(|x| x as i16);
            self.records[self.counter as usize] = meters;
            self.counter = (self.counter + 1) % self.rate as u8;
            let delta = meters - self.records[self.counter as usize];
            self.data.write((altitude, Velocity::new(delta.value() * SECONDS_PER_MINUTE, Meter)))
        }
        true
    }

    fn rate(&self) -> Rate {
        self.rate
    }
}
