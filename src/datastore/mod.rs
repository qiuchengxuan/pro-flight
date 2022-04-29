use core::{mem::MaybeUninit, time};

use concat_idents::concat_idents;

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
        sensor::Magnetism,
    },
};

#[derive(Copy, Clone, Default)]
struct Entry<T: Default> {
    timestamp: time::Duration,
    data: Option<T>,
}

impl<T: Copy + Default> Entry<T> {
    fn read(&self) -> T {
        self.data.unwrap_or_default()
    }

    fn read_within(&self, max_timestamp: time::Duration) -> Option<T> {
        if !max_timestamp.is_zero() && self.timestamp > max_timestamp {
            return None;
        }
        self.data
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
                    pub fn getter(&self, timeout: time::Duration) -> Option<$types> {
                        self.$names.read().read_within(jiffies::get() + timeout)
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
                        if self.$names.write(entry).is_err() {
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
    magnetism: Magnetism,
    voltage: Voltage
}

#[inline]
pub fn acquire() -> &'static DataStore {
    static DATASTORE: MaybeUninit<DataStore> = MaybeUninit::uninit();
    unsafe { &*DATASTORE.as_ptr() }
}
