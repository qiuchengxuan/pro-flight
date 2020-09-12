pub mod mahony;

use core::ops::{Add, AddAssign, Div, Mul, Sub};

#[derive(Copy)]
pub struct ComplementaryFilter<T: Default> {
    alpha: T,
    d_t: T,
    output: T,
}

impl<T: Copy + Default> Clone for ComplementaryFilter<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self { alpha: self.alpha, d_t: self.d_t, output: T::default() }
    }
}

impl<T> ComplementaryFilter<T>
where
    T: Default
        + Mul<Output = T>
        + Add<Output = T>
        + Sub<Output = T>
        + Div<Output = T>
        + AddAssign
        + From<u8>
        + Copy,
{
    pub fn new(alpha: T, d_t: T) -> Self {
        Self { alpha, d_t, output: T::default() }
    }

    //                       w
    //                      +|
    // input       ┌───┐   + v     ┌───┐
    // ──────>◯───>│ α │────>◯────>│ ∫ │──┬─>Output
    //      + ↑    └───┘           └───┘  │
    //       -│                           │
    //        └───────────────────────────┘
    pub fn filter(&mut self, input: T, w: T) -> T {
        let derivative = (input - self.output) * self.alpha + w;
        self.output += derivative * self.d_t;
        self.output
    }
}
