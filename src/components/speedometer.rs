use alloc::boxed::Box;
use alloc::rc::Rc;

use nalgebra::Vector3;

use crate::algorithm::ComplementaryFilter;
use crate::components::schedule::{Rate, Schedulable};
use crate::datastructures::data_source::singular::{SingularData, SingularDataSource};
use crate::datastructures::data_source::{AgingStaticData, DataWriter, OptionData, StaticData};
use crate::datastructures::measurement::unit::{Meter, MilliMeter};
use crate::datastructures::measurement::{Altitude, VelocityVector, GRAVITY};

pub struct Speedometer<A, ACCEL> {
    altimeter: A,
    accelerometer: ACCEL,
    sample_interval: f32,

    gnss: Option<Box<dyn AgingStaticData<VelocityVector<i32, MilliMeter>>>>,
    filters: [ComplementaryFilter<f32>; 3],

    altitude: Altitude,
    vector: (f32, f32, f32),
    output: Rc<SingularData<VelocityVector<f32, Meter>>>,
}

impl<A, ACCEL> Speedometer<A, ACCEL> {
    pub fn new(altimeter: A, accelerometer: ACCEL, sample_rate: usize) -> Self {
        Self {
            altimeter,
            accelerometer,
            sample_interval: 1.0 / sample_rate as f32,
            gnss: None,
            filters: [ComplementaryFilter::new(0.05, 1.0 / sample_rate as f32); 3],
            altitude: Altitude::default(),
            vector: (0.0, 0.0, 0.0),
            output: Rc::new(SingularData::default()),
        }
    }

    pub fn set_gnss(&mut self, gnss: Box<dyn AgingStaticData<VelocityVector<i32, MilliMeter>>>) {
        self.gnss = Some(gnss)
    }

    pub fn reader(&mut self) -> SingularDataSource<VelocityVector<f32, Meter>> {
        SingularDataSource::new(&self.output)
    }
}

impl<A, ACCEL> Schedulable for Speedometer<A, ACCEL>
where
    A: StaticData<Altitude>,
    ACCEL: OptionData<Vector3<f32>>,
{
    fn schedule(&mut self) -> bool {
        let rate = self.rate();

        let altitude = self.altimeter.read();
        let z = (altitude - self.altitude).convert(|v| v as f32).to_unit(Meter);
        self.altitude = altitude;
        let gnss = self.gnss.as_mut().map(|gnss| gnss.read(rate)).flatten().map(|v| {
            let x = v.x.convert(|v| v as f32).to_unit(Meter);
            let y = v.y.convert(|v| v as f32).to_unit(Meter);
            VelocityVector::new(x.value(), y.value(), 0.0, Meter)
        });
        while let Some(mut acceleration) = self.accelerometer.read() {
            acceleration *= GRAVITY;
            if let Some(vector) = gnss {
                self.vector.0 = self.filters[0].filter(vector.x.value(), acceleration[0]);
                self.vector.1 = self.filters[1].filter(vector.y.value(), acceleration[1]);
            } else {
                self.vector.0 += acceleration[0] * self.sample_interval;
                self.vector.1 += acceleration[1] * self.sample_interval;
            }
            self.vector.2 = self.filters[2].filter(z.value(), acceleration[2] + GRAVITY);
        }
        self.output.write(VelocityVector::new(self.vector.0, self.vector.1, self.vector.2, Meter));
        true
    }

    fn rate(&self) -> Rate {
        50
    }
}
