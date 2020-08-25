macro_rules! units {
    () => ();
    (
        $class:ident => $value:expr,
        $(
            $classes:ident => $values:expr,
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

        units!{ $($classes => $values,)* }
    };
}

units! {
    MilliMeter => 1.0,
    CentiMeter => 10.0,
    Meter => 1000.0,
    Feet => 3300.0,
    FTpM => 3300.0 / 60.0,
    KiloMeter => 1000_000.0,
    KMpH => 1000_000.0 / 3600.0,
    NauticalMile => 1852_000.0,
    Knot => 1852_000.0 / 3600.0,
}
