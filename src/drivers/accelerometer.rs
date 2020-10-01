use alloc::rc::Rc;

use crate::alloc;
use crate::datastructures::data_source::overwriting::{OverwritingData, OverwritingDataSource};
use crate::datastructures::measurement::Acceleration;

pub static mut ACCELEROMETER: Option<Rc<OverwritingData<Acceleration>>> = None;

pub fn init_data_source() -> OverwritingDataSource<Acceleration> {
    unsafe { ACCELEROMETER = Some(Rc::new(OverwritingData::sized(40))) };
    unsafe { ACCELEROMETER.as_ref().map(|a| OverwritingDataSource::new(&a)).unwrap() }
}
