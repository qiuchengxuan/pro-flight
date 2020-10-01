use alloc::rc::Rc;

use crate::alloc;
use crate::datastructures::data_source::overwriting::{OverwritingData, OverwritingDataSource};
use crate::datastructures::measurement::Gyro;

pub static mut GYROSCOPE: Option<Rc<OverwritingData<Gyro>>> = None;

pub fn init_data_source() -> OverwritingDataSource<Gyro> {
    unsafe { GYROSCOPE = Some(Rc::new(OverwritingData::sized(40))) };
    unsafe { GYROSCOPE.as_ref().map(|a| OverwritingDataSource::new(&a)).unwrap() }
}
