use archery::SharedPointerKind;

use crate::{
    protocols::{ifn::TFn, persistent_map::IPersistentMap},
    value::{Value, ValuePtr, ValuePtrs},
};

pub struct GetFn;

impl<P: SharedPointerKind> TFn<P> for GetFn {
    fn invoke(&self, args: ValuePtrs<P>) -> ValuePtr<P> {
        debug_assert_eq!(args.len(), 2); // (get {} :k)

        tracing::trace!(
            "({} {})",
            stringify!(GetFn),
            args.clone()
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        );

        let map_val_ptr = args.get(0).map(ValuePtr::clone).unwrap();
        let key_val_ptr = args.get(1).unwrap();

        match map_val_ptr.as_ref() {
            Value::Map(map) => map.get(key_val_ptr),
            value => {
                tracing::warn!("first arg to get-fn must be a map, but was {:?}", value);
                unimplemented!("unsupported invocation: get on non-Map Value: {:?}", value);
            }
        }
    }
}
