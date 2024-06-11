use std::ops::{Add, Div, Mul, Sub};

/// maps numeric ranges
pub fn map_range<
    T: Sub<Output = T> + Copy + Mul<Output = T> + Div<Output = T> + Add<Output = T>,
>(
    from_range: (T, T),
    to_range: (T, T),
    s: T,
) -> T {
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
}
