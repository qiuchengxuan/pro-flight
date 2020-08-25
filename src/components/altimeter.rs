use alloc::rc::Rc;

use crate::components::schedule::{Rate, Schedulable};
use crate::datastructures::data_source::singular::{SingularData, SingularDataSource};
use crate::datastructures::data_source::{DataWriter, OptionData};
use crate::datastructures::measurement::{Altitude, Pressure};

pub struct Altimeter<B> {
    barometer: B,
    data: Rc<SingularData<Altitude>>,

    rate: Rate,
}

impl<B: OptionData<Pressure>> Altimeter<B> {
    pub fn new(barometer: B, rate: Rate) -> Self {
        let data = Rc::new(SingularData::default());
        Self { barometer, data, rate }
    }

    pub fn reader(&self) -> SingularDataSource<Altitude> {
        SingularDataSource::new(&self.data)
    }
}

impl<D: OptionData<Pressure>> Schedulable for Altimeter<D> {
    fn schedule(&mut self) -> bool {
        if let Some(value) = self.barometer.read() {
            self.data.write(value.into())
        }
        true
    }

    fn rate(&self) -> Rate {
        self.rate
    }
}
