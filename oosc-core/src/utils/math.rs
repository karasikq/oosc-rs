#[inline]
pub fn clamp<T: Clone + PartialOrd>(input: T, range: &(T, T)) -> T {
    debug_assert!(range.0 <= range.1, "min must be less than or equal to max");
    if input < range.0 {
        range.0.clone()
    } else if input > range.1 {
        range.1.clone()
    } else {
        input
    }
}
