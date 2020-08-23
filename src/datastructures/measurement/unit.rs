#[derive(Copy, Clone, Default, Debug)]
pub struct MilliMeter;

impl Into<i32> for MilliMeter {
    fn into(self) -> i32 {
        1
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct CentiMeter;

impl Into<i32> for CentiMeter {
    fn into(self) -> i32 {
        10
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Meter;

impl Into<i32> for Meter {
    fn into(self) -> i32 {
        1000
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Feet;

impl Into<i32> for Feet {
    fn into(self) -> i32 {
        3300
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct KiloMeter;

impl Into<i32> for KiloMeter {
    fn into(self) -> i32 {
        1000_000
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct NauticalMile;

impl Into<i32> for NauticalMile {
    fn into(self) -> i32 {
        1852_000
    }
}
