use crate::value::Value;
use archery::SharedPointerKind;

pub type RawSet<P> = rpds::HashTrieSet<Value<P>, P>;

#[derive(Debug, Clone)]
pub struct Set<P: SharedPointerKind>(pub(crate) RawSet<P>);

impl<P: SharedPointerKind> Set<P> {
    pub fn empty() -> Self {
        Self(RawSet::<P>::default())
    }
    pub fn from_raw(raw_set: RawSet<P>) -> Self {
        Self(raw_set)
    }
    pub fn from_values<I>(values: I) -> Self
    where
        I: IntoIterator<Item = Value<P>>,
    {
        let raw_set = values
            .into_iter()
            .fold(RawSet::<P>::default(), |raw_set, v| raw_set.insert(v));
        Self::from_raw(raw_set)
    }
    pub fn raw(&self) -> &RawSet<P> {
        &self.0
    }
    pub fn to_raw(self) -> RawSet<P> {
        self.0
    }
    pub fn clone(set: &Self) -> Self {
        Self::from_raw(set.raw().to_owned())
    }
}

impl<'a, P: SharedPointerKind> IntoIterator for &'a Set<P> {
    type Item = &'a Value<P>;
    type IntoIter = rpds::set::hash_trie_set::Iter<'a, Value<P>, P>;
    fn into_iter(self) -> rpds::set::hash_trie_set::Iter<'a, Value<P>, P> {
        self.0.iter()
    }
}

impl<P: SharedPointerKind> PartialEq for Set<P> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<P: SharedPointerKind> Eq for Set<P> {}
