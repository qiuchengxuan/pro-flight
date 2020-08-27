macro_rules! units {
    () => ();
    (
        $class:ident => ($value:expr, $name:expr),
        $(
            $classes:ident => ($values:expr, $names:expr),
        )*
    ) => {
        #[derive(Copy, Clone, Default, Debug, PartialEq)]
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

        units!{ $($classes => ($values, $names),)* }
    };
}

units! {
    MilliMeter => (1.0, "mm"),
    CentiMeter => (10.0, "cm"),
    Meter => (1000.0, "m"),
    Feet => (3300.0, "ft"),
    FTpM => (3300.0 / 60.0, "ft/min"),
    KiloMeter => (1000_000.0, "km"),
    KMpH => (1000_000.0 / 3600.0 , "km/h"),
    NauticalMile => (1852_000.0, "nm"),
    Knot => (1852_000.0 / 3600.0, "knot"),
}
