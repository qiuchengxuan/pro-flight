use alloc::rc::Rc;

use crate::datastructures::data_source::overwriting::{OverwritingData, OverwritingDataSource};
use crate::datastructures::measurement::Pressure;

pub mod bmp280;
mod global;

pub fn init_data_source() -> OverwritingDataSource<Pressure> {
    unsafe { global::BAROMETER = Some(OverwritingData::sized(8)) };
    let buffer = unsafe { &Rc::from_raw(global::BAROMETER.as_ref().unwrap()) };
    core::mem::forget(buffer);
    OverwritingDataSource::new(buffer)
}
