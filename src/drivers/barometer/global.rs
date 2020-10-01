use crate::datastructures::data_source::overwriting::OverwritingData;
use crate::datastructures::measurement::Pressure;

pub static mut BAROMETER: Option<OverwritingData<Pressure>> = None;
