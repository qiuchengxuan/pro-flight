use alloc::rc::Rc;

use crate::datastructures::data_source::singular::SingularDataSource;
use crate::datastructures::measurement::Magnetism;

mod global;
pub mod qmc5883l;

pub fn get_data_source() -> Option<SingularDataSource<Magnetism>> {
    if let Some(magnetometer) = unsafe { global::MAGNETOMETER.as_ref() } {
        let buffer = unsafe { &Rc::from_raw(magnetometer) };
        core::mem::forget(buffer);
        return Some(SingularDataSource::new(buffer));
    }
    None
}
