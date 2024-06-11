use std::ops::Sub;

/// maps numeric ranges
pub fn map_range<T: Sub + Copy>(from_range: (T, T), to_range: (T, T), s: T) -> T {
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
}
