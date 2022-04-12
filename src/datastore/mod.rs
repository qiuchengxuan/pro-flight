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
    },
};

#[derive(Copy, Clone, Default)]
struct Entry<T: Default> {
    timestamp: time::Duration,
    data: T,
}

impl<T: Copy + Default> Entry<T> {
    fn read(&self, max_timestamp: time::Duration) -> Option<T> {
        if !max_timestamp.is_zero() && self.timestamp > max_timestamp {
            return None;
        }
        Some(self.data)
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
                concat_idents!(getter = read_, $names {
                    pub fn getter(&self, timeout: Option<time::Duration>) -> Option<$types> {
                        let max_timestamp = timeout.map(|t| jiffies::get() + t).unwrap_or_default();
                        self.$names.read().read(max_timestamp)
                    }
                });

                concat_idents!(setter = write_, $names {
                    pub fn setter(&self, data: $types) {
                        self.$names.write(Entry{timestamp: jiffies::get(), data})
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
    voltage: Voltage
}

#[inline]
pub fn acquire() -> &'static DataStore {
    static DATASTORE: MaybeUninit<DataStore> = MaybeUninit::uninit();
    unsafe { &*DATASTORE.as_ptr() }
}
