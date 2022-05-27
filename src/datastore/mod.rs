use core::mem::MaybeUninit;

use concat_idents::concat_idents;
use fugit::NanosDurationU64 as Duration;
use nalgebra::Vector3;

use crate::{
    fcs::out::FCS,
    imu::out::IMU,
    ins::out::INS,
    protocol::serial::gnss::out::GNSS,
    sync::ReadSpinLock,
    sys::jiffies,
    types::{
        control::Control,
        measurement::{voltage::Voltage, Altitude},
    },
};

#[derive(Clone)]
struct Entry<T> {
    timestamp: Duration,
    data: Option<T>,
}

impl<T: Default> Default for Entry<T> {
    fn default() -> Self {
        Self { timestamp: Duration::nanos(0), data: None }
    }
}

impl<T: Clone + Default> Entry<T> {
    fn read(&self) -> T {
        self.data.clone().unwrap_or_default()
    }

    fn read_within(&self, max_timestamp: Duration) -> Option<T> {
        if self.timestamp > max_timestamp {
            return None;
        }
        self.data.clone()
    }
}

macro_rules! datastore {
    ($($names:ident: $types:ty),+) => {
        #[derive(Default)]
        pub struct DataStore {
            $($names: ReadSpinLock<Entry<$types>>),+
        }

        impl DataStore {
            $(
                concat_idents!(getter = read_, $names, _within {
                    pub fn getter(&self, timeout: Duration) -> Option<$types> {
                        self.$names.read().read_within(jiffies::get() + timeout.convert())
                    }
                });

                concat_idents!(getter = read_, $names {
                    pub fn getter(&self) -> $types {
                        self.$names.read().read()
                    }
                });

                concat_idents!(setter = write_, $names {
                    pub fn setter(&self, data: $types) {
                        let entry = Entry{timestamp: jiffies::get(), data: Some(data)};
                        if self.$names.write(entry.clone()).is_err() {
                            error!("Write {} conflict", core::any::type_name::<$types>())
                        }
                    }
                });
            )+
        }
    }
}

datastore! {
    altitude: Altitude,
    control: Control,
    fcs: FCS,
    gnss: GNSS,
    imu: IMU,
    ins: INS,
    magnetism: Vector3<f32>,
    voltage: Voltage
}

#[inline]
pub fn acquire() -> &'static DataStore {
    static DATASTORE: MaybeUninit<DataStore> = MaybeUninit::uninit();
    unsafe { &*DATASTORE.as_ptr() }
}
