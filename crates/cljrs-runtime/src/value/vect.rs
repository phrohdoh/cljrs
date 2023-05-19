use crate::value::Value;
use archery::SharedPointerKind;

pub type RawVectItem<P> = Value<P>;
pub type RawVect<P> = rpds::Vector<RawVectItem<P>, P>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vect<P: SharedPointerKind>(pub(crate) RawVect<P>);

impl<P: SharedPointerKind> Vect<P> {
    pub fn empty() -> Self {
        Self(RawVect::new_with_ptr_kind())
    }
    pub fn from_raw(raw_vect: RawVect<P>) -> Self {
        Self(raw_vect)
    }
    pub fn from_values<I>(values: I) -> Self
    where
        I: IntoIterator<Item = Value<P>>,
    {
        let vals = values.into_iter().collect::<Vec<_>>();
        let mut raw_vect = RawVect::<P>::default();
        for val in vals {
            raw_vect.push_back_mut(val);
        }
        Self::from_raw(raw_vect)
    }
    pub fn raw(&self) -> &RawVect<P> {
        &self.0
    }
    pub fn to_raw(self) -> RawVect<P> {
        self.0
    }
    pub fn clone(vect: &Self) -> Self {
        Self::from_raw(vect.raw().to_owned())
    }
}

impl<'a, P: SharedPointerKind> IntoIterator for &'a Vect<P> {
    type Item = &'a Value<P>;
    type IntoIter = rpds::vector::Iter<'a, Value<P>, P>;
    fn into_iter(self) -> rpds::vector::Iter<'a, Value<P>, P> {
        self.0.iter()
    }
}
