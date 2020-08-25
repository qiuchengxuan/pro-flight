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
