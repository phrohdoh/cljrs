use super::ValuePtr;
use crate::{value::Value, protocols::persistent_map::IPersistentMap};
use archery::SharedPointerKind;

pub type RawMap<P> = rpds::HashTrieMap<Value<P>, Value<P>, P>;

#[derive(Debug, Clone)]
pub struct Map<P: SharedPointerKind>(pub(crate) RawMap<P>);

impl<P: SharedPointerKind> Map<P> {
    pub fn empty() -> Self {
        Self(RawMap::<P>::default())
    }
    pub fn from_raw(raw_map: RawMap<P>) -> Self {
        Self(raw_map)
    }
    pub fn from_value_pairs<I>(values: I) -> Self
    where
        I: IntoIterator<Item = (Value<P>, Value<P>)>,
    {
        let raw_map = values
            .into_iter()
            .fold(RawMap::<P>::default(), raw_map_utils::insert_pair);
        Self::from_raw(raw_map)
    }
    pub fn raw(&self) -> &RawMap<P> {
        &self.0
    }
    pub fn to_raw(self) -> RawMap<P> {
        self.0
    }
    pub fn clone(map: &Self) -> Self {
        Self::from_raw(map.raw().to_owned())
    }
}

impl<'a, P: SharedPointerKind> IntoIterator for &'a Map<P> {
    type Item = (&'a Value<P>, &'a Value<P>);
    type IntoIter = rpds::map::hash_trie_map::Iter<'a, Value<P>, Value<P>, P>;
    fn into_iter(self) -> rpds::map::hash_trie_map::Iter<'a, Value<P>, Value<P>, P> {
        self.0.iter()
    }
}

impl<P: SharedPointerKind> PartialEq for Map<P> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<P: SharedPointerKind> Eq for Map<P> {}

impl<P: SharedPointerKind> IPersistentMap<P> for Map<P> {
    fn get(&self, key: &ValuePtr<P>) -> ValuePtr<P> {
        self.get_with_default(key, &Value::Nil.to_ptr())
    }

    fn get_with_default(&self, key: &ValuePtr<P>, default: &ValuePtr<P>) -> ValuePtr<P> {
        let opt_val = self.0.get(key.as_ref());
        let val_or_default = opt_val.unwrap_or(default.as_ref());
        let owned_val = Value::clone(val_or_default);
        ValuePtr::from(owned_val)
    }

    fn assoc(&self, key: &ValuePtr<P>, value: ValuePtr<P>) -> Self {
        let key = Value::clone(key);
        let value = Value::clone(&value);
        let new_raw_map = self.0.insert(key, value);
        Self::from_raw(new_raw_map)
    }

    fn contains_key(&self, key: &ValuePtr<P>) -> bool {
        self.0.contains_key(key.as_ref())
    }
}

pub mod raw_map_utils {
    use super::RawMap;
    use crate::value::Value;
    use archery::SharedPointerKind;
    pub fn insert_pair<P: SharedPointerKind>(
        raw_map: RawMap<P>,
        (k, v): (Value<P>, Value<P>),
    ) -> RawMap<P> {
        raw_map.insert(k, v)
    }
}
