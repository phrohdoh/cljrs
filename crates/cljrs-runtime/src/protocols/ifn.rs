use std::sync::Arc;

use archery::SharedPointerKind;
use crate::value::{ValuePtr, Value, RcValue, ArcValue};
use crate::protocol::TProtocol;

pub struct PFn<P: SharedPointerKind>(ValuePtr<P>);

impl<P: SharedPointerKind> TProtocol<P> for PFn<P> {
    fn is_instance_of(value: &ValuePtr<P>) -> bool {
        value.is_ifn()
    }
    fn raw_wrap(value: &ValuePtr<P>) -> Self {
        Self(value.clone())
    }
    fn raw_unwrap(&self) -> ValuePtr<P> {
        self.0.clone()
    }
}

impl<P: SharedPointerKind> std::ops::Deref for PFn<P> {
    type Target = ValuePtr<P>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<P: SharedPointerKind> std::fmt::Debug for PFn<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("PFn")/*.field(&self.0)*/.finish()
    }
}

impl<P: SharedPointerKind> PFn<P> {
    pub fn clone(&self) -> Self {
        Self(ValuePtr::clone(&self.0))
    }
    pub fn from_arc_ifn(arc_ifn: Arc<dyn TFn<P>>) -> Self {
        Self(ValuePtr::from(Value::IFn(arc_ifn)))
    }
    pub fn from_value(value: Value<P>) -> Self {
        Self(value.to_ptr())
    }
    pub fn from_value_ptr(value_ptr: ValuePtr<P>) -> Self {
        Self::try_from_value_ptr(value_ptr).unwrap()
    }
    pub fn try_from_value_ptr(value_ptr: ValuePtr<P>) -> Option<Self> {
        match ValuePtr::try_unwrap(value_ptr) {
            Ok(Value::IFn(ifn)) => Some(Self(ValuePtr::from(Value::IFn(ifn)))),
            Ok(_) => None,
            Err(_) => None,
        }
    }
    pub fn as_inner(&self) -> &ValuePtr<P> {
        &self.0
    }
    pub fn to_raw(self) -> ValuePtr<P> {
        self.0
    }
}

impl PFn<archery::RcK> {
    pub fn from_rc_value(v: RcValue) -> Self {
        Self(v.to_rc())
    }
}

impl PFn<archery::ArcK> {
    pub fn from_arc_value(v: ArcValue) -> Self {
        Self(v.to_arc())
    }
}

pub trait TFn<P: SharedPointerKind> {
    fn invoke(&self, args: Vec<ValuePtr<P>>) -> ValuePtr<P>;
}

impl<P: SharedPointerKind> TFn<P> for PFn<P> {
    fn invoke(&self, args: Vec<ValuePtr<P>>) -> ValuePtr<P> {
        match self.0.as_ref() {
            Value::IFn(ifn) => ifn.invoke(args),
            _ => unimplemented!("unsupported invocation: TFn::invoke on non-TFn Value")
        }
    }
}

impl<P: SharedPointerKind> TFn<P> for dyn std::ops::Fn(Vec<ValuePtr<P>>) -> ValuePtr<P> {
    fn invoke(&self, args: Vec<ValuePtr<P>>) -> ValuePtr<P> {
        self(args)
    }
}

impl<P: SharedPointerKind> TFn<P> for &dyn std::ops::Fn(Vec<ValuePtr<P>>) -> ValuePtr<P> {
    fn invoke(&self, args: Vec<ValuePtr<P>>) -> ValuePtr<P> {
        self(args)
    }
}
