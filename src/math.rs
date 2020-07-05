pub fn runge_kutta4<T, F>(f: F, y: T, x: T, dx: T) -> T
where
    F: Fn(T, T) -> T,
    T: From<u8>
        + core::ops::Mul<Output = T>
        + core::ops::Add<Output = T>
        + core::ops::Div<Output = T>
        + Copy,
{
    let two: T = 2u8.into();
    let six: T = 6u8.into();
    let k1 = dx * f(y, x);
    let k2 = dx * f(y + k1 / two, x + dx / two);
    let k3 = dx * f(y + k2 / two, x + dx / two);
    let k4 = dx * f(y + k3, x + dx);

    y + (k1 + two * k2 + two * k3 + k4) / six
}
