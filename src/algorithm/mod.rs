pub fn runge_kutta4<T, F>(f: F, y: T, x: T, dx: T) -> T
where
    F: Fn(T, T) -> T,
    T: From<u8>
        + core::ops::Mul<Output = T>
        + core::ops::Add<Output = T>
        + core::ops::Div<Output = T>
        + Copy,
{
    let _2: T = 2u8.into();
    let _6: T = 6u8.into();
    let k1 = dx * f(y, x);
    let k2 = dx * f(y + k1 / _2, x + dx / _2);
    let k3 = dx * f(y + k2 / _2, x + dx / _2);
    let k4 = dx * f(y + k3, x + dx);

    y + (k1 + _2 * k2 + _2 * k3 + k4) / _6
}

//                       w
//                      +|
// input       ┌───┐   + v     ┌───┐
// ──────>◯───>│ α │────>◯────>│ ∫ │──┬─>Output
//      + ↑    └───┘           └───┘  │
//       -│                           │
//        └───────────────────────────┘
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
        + core::ops::Mul<Output = T>
        + core::ops::Add<Output = T>
        + core::ops::Sub<Output = T>
        + core::ops::Div<Output = T>
        + Copy,
{
    pub fn new(alpha: T, d_t: T) -> Self {
        Self { alpha, d_t, output: T::default() }
    }

    pub fn filter(&mut self, input: T, w: T) -> T {
        let integrator = (input - self.output) * self.alpha + w;
        self.output = self.output + integrator * self.d_t;
        self.output
    }
}
