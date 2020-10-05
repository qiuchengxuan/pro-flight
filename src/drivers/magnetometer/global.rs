use alloc::rc::Rc;

use crate::datastructures::data_source::singular::SingularData;
use crate::datastructures::measurement::Magnetism;

pub static mut MAGNETOMETER: Option<Rc<SingularData<Magnetism>>> = None;
