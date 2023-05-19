use crate::value::ValuePtr;
use archery::SharedPointerKind;

pub trait TProtocol<P: SharedPointerKind>: Sized {
    fn is_instance_of(value: &ValuePtr<P>) -> bool;
    fn raw_wrap(value: &ValuePtr<P>) -> Self;
    fn raw_unwrap(&self) -> ValuePtr<P>;

    /// # Panics
    ///
    /// if provided [crate::value::Value] does not implement [self::TProtocol]
    fn try_as_protocol(value: &ValuePtr<P>) -> Option<Self> {
        if Self::is_instance_of(value) {
            Some(Self::raw_wrap(value))
        } else {
            None
        }
    }

    fn as_protocol(value: &ValuePtr<P>) -> Self {
        Self::try_as_protocol(value).unwrap()
    }

    /// # Panics
    ///
    /// if [unwrap] panics
    fn try_unwrap(&self) -> Option<ValuePtr<P>> {
        let value = self.raw_unwrap();
        if Self::is_instance_of(&value) {
            Some(value)
        } else {
            None
        }
    }

    /// # Panics
    ///
    /// if wrapped [crate::value::Value] does not implement [self::TProtocol]
    fn unwrap(&self) -> ValuePtr<P> {
        self.try_unwrap().unwrap()
    }
}

pub trait TProtocolCastable<P: SharedPointerKind> {
    fn is_instance_of<T: TProtocol<P>>(&self) -> bool;
    fn try_as_protocol<T: TProtocol<P>>(&self) -> Option<T>;
    fn as_protocol<T: TProtocol<P>>(&self) -> T;
}

impl<P: SharedPointerKind> TProtocolCastable<P> for ValuePtr<P> {
    fn is_instance_of<T: TProtocol<P>>(&self) -> bool {
        T::is_instance_of(self)
    }
    fn try_as_protocol<T: TProtocol<P>>(&self) -> Option<T> {
        T::try_as_protocol(self)
    }
    fn as_protocol<T: TProtocol<P>>(&self) -> T {
        T::as_protocol(self)
    }
}

/*
pub trait IFnTrait<P: SharedPointerKind> {
    fn invoke(&self, args: Vec<ValuePtr<P>>) -> Value<P>;
}
pub struct IFnProtocol<P: SharedPointerKind>(ValuePtr<P>);
impl<P: SharedPointerKind> IFnTrait<P> for IFnProtocol<P> {
    fn invoke(&self, args: Vec<ValuePtr<P>>) -> Value<P> {
        todo!()
    }
}
*/
