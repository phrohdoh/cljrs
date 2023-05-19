use crate::value::Value;
use archery::SharedPointerKind;

pub type RawListItem<P> = Value<P>;
pub type RawList<P> = rpds::List<RawListItem<P>, P>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct List<P: SharedPointerKind>(pub(crate) RawList<P>);

impl<P: SharedPointerKind> List<P> {
    pub fn empty() -> Self {
        Self(RawList::new_with_ptr_kind())
    }
    pub fn from_raw(raw_list: RawList<P>) -> Self {
        Self(raw_list)
    }
    pub fn from_values<I>(values: I) -> Self
    where
        I: IntoIterator<Item = Value<P>>,
    {
        let vals = values.into_iter().collect::<Vec<_>>();
        let mut raw_list = RawList::<P>::default();
        for val in vals.into_iter().rev() {
            raw_list.push_front_mut(val);
        }
        Self::from_raw(raw_list)
    }
    pub fn raw(&self) -> &RawList<P> {
        &self.0
    }
    pub fn to_raw(self) -> RawList<P> {
        self.0
    }
    pub fn clone(list: &Self) -> Self {
        Self::from_raw(list.raw().to_owned())
    }
}

impl<'a, P: SharedPointerKind> IntoIterator for &'a List<P> {
    type Item = &'a Value<P>;
    type IntoIter = rpds::list::Iter<'a, Value<P>, P>;
    fn into_iter(self) -> rpds::list::Iter<'a, Value<P>, P> {
        self.0.iter()
    }
}
