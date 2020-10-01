use crate::datastructures::data_source::singular::SingularData;
use crate::datastructures::measurement::Magnetism;

pub static mut MAGNETOMETER: Option<SingularData<Magnetism>> = None;
