use archery::SharedPointerKind;
use crate::{value::{ValuePtr, Value}, protocol::TProtocol};

pub trait IPersistentMap<P: SharedPointerKind> {
    fn get(&self, key: &ValuePtr<P>) -> ValuePtr<P>;
    fn get_with_default(&self, key: &ValuePtr<P>, default: &ValuePtr<P>) -> ValuePtr<P>;
    fn assoc(&self, key: &ValuePtr<P>, value: ValuePtr<P>) -> Self;
    fn contains_key(&self, key: &ValuePtr<P>) -> bool;
}

pub struct PersistentMap<P: SharedPointerKind>(ValuePtr<P>);

impl<P: SharedPointerKind> AsRef<Value<P>> for PersistentMap<P> {
    fn as_ref(&self) -> &Value<P> {
        self.0.as_ref()
    }
}

impl<P: SharedPointerKind> PersistentMap<P> {
    pub fn from_value_ptr(value_ptr: ValuePtr<P>) -> Self {
        Self(value_ptr)
    }
    pub fn try_from_value_ptr(value_ptr: ValuePtr<P>) -> Option<Self> {
        match value_ptr.as_ref() {
            Value::Map(map) => {
                let value = Value::from_map_ref(map);
                let value_ptr = ValuePtr::from(value);
                let this = Self(value_ptr);
                Some(this)
            },
            _ => None,
        }
    }
    pub fn to_raw(self) -> ValuePtr<P> {
        self.0
    }
}

impl<P: SharedPointerKind> TProtocol<P> for PersistentMap<P> {
    fn is_instance_of(value: &ValuePtr<P>) -> bool {
        value.is_map()
    }
    fn raw_wrap(value: &ValuePtr<P>) -> Self {
        Self(value.clone())
    }
    fn raw_unwrap(&self) -> ValuePtr<P> {
        self.0.clone()
    }
}

impl<P: SharedPointerKind> IPersistentMap<P> for PersistentMap<P> {
    fn get(&self, key: &ValuePtr<P>) -> ValuePtr<P> {
        match self.as_ref() {
            Value::Map(map) => map.get(key),
            _ => unimplemented!("unsupported invocation: <IPersistentMapProtocol as IPersistentMapTrait>::get on non-Map Value"),
        }
    }

    fn get_with_default(&self, key: &ValuePtr<P>, default: &ValuePtr<P>) -> ValuePtr<P> {
        match self.as_ref() {
            Value::Map(map) => map.get_with_default(key, default),
            _ => unimplemented!("unsupported invocation: <IPersistentMapProtocol as IPersistentMapTrait>::get_with_default on non-Map Value"),
        }
    }

    fn assoc(&self, key: &ValuePtr<P>, value: ValuePtr<P>) -> Self {
        match self.as_ref() {
            Value::Map(map) => Self(ValuePtr::from(Value::Map(map.assoc(key, value)))),
            _ => unimplemented!("unsupported invocation: <IPersistentMapProtocol as IPersistentMapTrait>::assoc on non-Map Value"),
        }
    }

    fn contains_key(&self, key: &ValuePtr<P>) -> bool {
        match self.as_ref() {
            Value::Map(map) => map.contains_key(key),
            _ => unimplemented!("unsupported invocation: <IPersistentMapProtocol as IPersistentMapTrait>::assoc on non-Map Value"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PersistentMap, IPersistentMap};
    use crate::value::{Value, ValuePtr, RcValuePtr};

    #[test]
    fn map() {
        let map = Value::empty_map();
        let map_proto = PersistentMap::from_value_ptr(RcValuePtr::from(map));
        let key = ValuePtr::from(Value::unqualified_keyword(String::from("k")));
        assert!(!map_proto.contains_key(&key));
        let val = ValuePtr::from(Value::unqualified_keyword(String::from("v")));
        let map_proto_2 = map_proto.assoc(&key, val.clone());
        assert!(map_proto_2.contains_key(&key));
        let val_out = map_proto_2.get(&key);
        assert_eq!(val, val_out, "v={:?} v2={:?}", val, val_out);
    }
}