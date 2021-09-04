pub trait Velocity {}
pub trait Distance {}

macro_rules! units {
    ($($class:ident => ($value:expr, $name:expr, $trait:ty)),+) => {
        $(
            #[derive(Copy, Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
            pub struct $class;

            impl Into<u32> for $class {
                fn into(self) -> u32 {
                    ($value) as u32
                }
            }

            impl Into<i32> for $class {
                fn into(self) -> i32 {
                    ($value) as i32
                }
            }

            impl Into<f32> for $class {
                fn into(self) -> f32 {
                    $value
                }
            }

            impl core::fmt::Display for $class {
                fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    write!(f, $name)
                }
            }

            impl $trait for $class {}
        )+
    };
}

macro_rules! velocity_units {
    ($($class:ident => ($value:expr, $name:literal)),+) => {
        units!{$($class => ($value, $name, Velocity)),+}
    }
}

velocity_units! {
    MMpS => (1.0, "mm/s"),
    CMpS => (10.0, "cm/s"),
    MpS => (1000.0, "m/s"),
    FTpM => (303.0 / 60.0, "ft/min"),
    KMpH => (1000_000.0 / 3600.0 , "km/h"),
    Knot => (1852_000.0 / 3600.0, "knot")
}

macro_rules! distance_units {
    ($($class:ident => ($value:expr, $name:literal)),+) => {
        units!{$($class => ($value, $name, Distance)),+}
    }
}

distance_units! {
    MilliMeter => (1.0, "mm"),
    CentiMeter => (10.0, "cm"),
    Feet => (303.0, "ft"),
    Meter => (1000.0, "m"),
    KiloMeter => (1000_000.0, "km"),
    NauticalMile => (1852_000.0, "nm")
}
