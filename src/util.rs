use core::ops::{Add, Div, Mul, Sub};

/// Map a number from one range to another range
pub fn map<T>(x: T, in_min: T, in_max: T, out_min: T, out_max: T) -> T
where
    T: Copy + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T>,
{
    (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
}

/// Clamp x to the inclusive range from min to max
pub fn clamp<T>(x: T, min: T, max: T) -> T
where
    T: PartialOrd,
{
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}
