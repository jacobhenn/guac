pub fn unordered_eq<T>(lhs: &[T], rhs: &[T]) -> bool
where
    T: Eq,
{
    lhs.len() == rhs.len() && lhs.iter().fold(true, |acc, lh| acc && rhs.contains(lh))
}
