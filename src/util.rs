/// Tell if two slices contain the same set of elements, regardless of order.
pub fn are_unordered_eq<T>(lhs: &[T], rhs: &[T]) -> bool
where
    T: Eq,
{
    lhs.len() == rhs.len() && lhs.iter().all(|lh| rhs.contains(lh))
}
