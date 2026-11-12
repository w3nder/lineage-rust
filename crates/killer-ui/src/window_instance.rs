use std::hash::{Hash, Hasher};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct WindowInstance(Arc<()>);

impl Default for WindowInstance {
    fn default() -> Self { Self(Arc::new(())) }
}

impl PartialEq for WindowInstance {
    fn eq(&self, other: &Self) -> bool { Arc::ptr_eq(&self.0, &other.0) }
}

impl Eq for WindowInstance {}

impl Hash for WindowInstance {
    fn hash<H: Hasher>(&self, state: &mut H) { std::ptr::hash(Arc::as_ptr(&self.0), state); }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn clones_share_identity_and_new_allocations_never_alias_live_handles() {
        let first = WindowInstance::default();
        let copy = first.clone();
        assert_eq!(first, copy);
        let mut instances = HashSet::from([first]);
        assert!(!instances.insert(copy.clone()));
        for _ in 0..4096 { assert!(instances.insert(WindowInstance::default())); }
        assert!(instances.contains(&copy));
        assert_eq!(instances.len(), 4097);
    }
}
