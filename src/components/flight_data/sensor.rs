use crate::datastructures::measurement::{Acceleration, Course, Gyro, Magnetism};

#[derive(Copy, Clone, Debug)]
pub struct GNSS {
    pub fixed: bool,
    pub course: Course,
}

impl sval::value::Value for GNSS {
    fn stream(&self, stream: &mut sval::value::Stream) -> sval::value::Result {
        stream.map_begin(Some(if self.fixed { 2 } else { 1 }))?;
        stream.map_key("fixed")?;
        stream.map_value(self.fixed)?;
        if self.fixed {
            stream.map_key("fixed")?;
            let course: f32 = self.course.into();
            stream.map_value(course)?;
        }
        stream.map_end()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Sensor {
    pub acceleration: Acceleration,
    pub gyro: Gyro,
    pub magnetism: Option<Magnetism>,
    pub gnss: Option<GNSS>,
}

impl Default for Sensor {
    fn default() -> Self {
        Self {
            acceleration: Acceleration::default(),
            gyro: Gyro::default(),
            magnetism: None,
            gnss: None,
        }
    }
}

impl sval::value::Value for Sensor {
    fn stream(&self, stream: &mut sval::value::Stream) -> sval::value::Result {
        stream.map_begin(Some(
            4 + self.gnss.is_some() as usize + self.magnetism.is_some() as usize,
        ))?;
        stream.map_key("acceleration")?;
        stream.map_value(&self.acceleration)?;
        stream.map_key("gyro")?;
        stream.map_value(&self.gyro)?;
        if let Some(magnetism) = self.magnetism {
            stream.map_key("magnetism")?;
            stream.map_value(magnetism)?;
        }
        if let Some(gnss) = self.gnss {
            stream.map_key("gnss")?;
            stream.map_value(gnss)?;
        }
        stream.map_end()
    }
}

impl core::fmt::Display for Sensor {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        sval_json::to_fmt(f, self).ok();
        Ok(())
    }
}
