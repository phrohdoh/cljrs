pub mod list;
pub mod map;
pub mod set;
pub mod vect;

use archery::{ArcK, RcK, SharedPointer, SharedPointerKind};
use cljrs_core::symbol::Symbol;
use std::hash::Hash;
use std::sync::Arc;

use crate::{
    protocols::ifn,
    value::{
        list::{List, RawList},
        map::{Map, RawMap},
        set::{RawSet, Set},
        vect::{RawVect, Vect},
    },
};

pub type RcValue = Value<RcK>;
pub type ArcValue = Value<ArcK>;

pub type ValuePtr<P> = SharedPointer<Value<P>, P>;
pub type RcValuePtr = ValuePtr<RcK>;
pub type ArcValuePtr = ValuePtr<ArcK>;

pub type ValuePtrs<P> = Vec<SharedPointer<Value<P>, P>>;
pub type RcValuePtrs = Vec<ValuePtr<RcK>>;
pub type ArcValuePtrs = Vec<ValuePtr<ArcK>>;

// TODO: impl Clone https://github.com/orium/archery/issues/26
pub enum Value<P: SharedPointerKind> {
    Nil,
    Bool(bool),
    Num(isize),
    Str(String),
    Keyword(Symbol),
    Symbol(Symbol),
    List(List<P>),
    Vect(Vect<P>),
    Set(Set<P>),
    Map(Map<P>),
    IFn(Arc<dyn ifn::TFn<P>>),
}

impl<P: SharedPointerKind + std::fmt::Debug> std::fmt::Debug for Value<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "Nil"),
            Self::Bool(arg0) => f.debug_tuple("Bool").field(arg0).finish(),
            Self::Num(arg0) => f.debug_tuple("Num").field(arg0).finish(),
            Self::Str(arg0) => f.debug_tuple("Str").field(arg0).finish(),
            Self::Keyword(arg0) => f.debug_tuple("Keyword").field(arg0).finish(),
            Self::Symbol(arg0) => f.debug_tuple("Symbol").field(arg0).finish(),
            Self::List(arg0) => f.debug_tuple("List").field(arg0).finish(),
            Self::Vect(arg0) => f.debug_tuple("Vect").field(arg0).finish(),
            Self::Set(arg0) => f.debug_tuple("Set").field(arg0).finish(),
            Self::Map(arg0) => f.debug_tuple("Map").field(arg0).finish(),
            Self::IFn(arg0) => {
                let raw = Arc::into_raw(Arc::clone(arg0));
                let fmt = write!(f, "#object[fn,{:?}]", raw);
                unsafe {
                    drop(Arc::from_raw(raw));
                }
                fmt
            },
        }
    }
}

impl<P: SharedPointerKind> Value<P> {
    pub fn is_nil(&self) -> bool {
        match self {
            Self::Nil => true,
            _ => false,
        }
    }
    pub fn is_bool(&self) -> bool {
        match self {
            Self::Bool(..) => true,
            _ => false,
        }
    }
    pub fn is_num(&self) -> bool {
        match self {
            Self::Num(..) => true,
            _ => false,
        }
    }
    pub fn is_str(&self) -> bool {
        match self {
            Self::Str(..) => true,
            _ => false,
        }
    }
    pub fn is_keyword(&self) -> bool {
        match self {
            Self::Keyword(..) => true,
            _ => false,
        }
    }
    pub fn is_symbol(&self) -> bool {
        match self {
            Self::Symbol(..) => true,
            _ => false,
        }
    }
    pub fn is_list(&self) -> bool {
        match self {
            Self::List(..) => true,
            _ => false,
        }
    }
    pub fn is_vect(&self) -> bool {
        match self {
            Self::Vect(..) => true,
            _ => false,
        }
    }
    pub fn is_set(&self) -> bool {
        match self {
            Self::Set(..) => true,
            _ => false,
        }
    }
    pub fn is_map(&self) -> bool {
        match self {
            Self::Map(..) => true,
            _ => false,
        }
    }
    pub fn is_ifn(&self) -> bool {
        match self {
            Value::IFn(_) => true,
            _ => false,
        }
    }
}

impl<P: SharedPointerKind> PartialEq for Value<P> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::Num(l0), Self::Num(r0)) => l0 == r0,
            (Self::Str(l0), Self::Str(r0)) => l0 == r0,
            (Self::Keyword(l0), Self::Keyword(r0)) => l0 == r0,
            (Self::Symbol(l0), Self::Symbol(r0)) => l0 == r0,
            (Self::List(List::<P>(l0)), Self::List(List::<P>(r0))) => l0 == r0,
            (Self::Vect(Vect::<P>(l0)), Self::Vect(Vect::<P>(r0))) => l0 == r0,
            (Self::Set(Set::<P>(l0)), Self::Set(Set::<P>(r0))) => l0 == r0,
            (Self::Map(Map::<P>(l0)), Self::Map(Map::<P>(r0))) => l0 == r0,
            (Self::IFn(_), Self::IFn(_)) => false,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl<P: SharedPointerKind> Value<P> {
    pub fn clone(&self) -> Self {
        match self {
            Self::Nil => Self::Nil,
            Self::Bool(arg0) => Self::Bool(arg0.clone()),
            Self::Num(arg0) => Self::Num(arg0.clone()),
            Self::Str(arg0) => Self::Str(arg0.clone()),
            Self::Keyword(arg0) => Self::Keyword(arg0.clone()),
            Self::Symbol(arg0) => Self::Symbol(arg0.clone()),
            Self::List(list) => Self::List(List::clone(list)),
            Self::Vect(list) => Self::Vect(Vect::clone(list)),
            Self::Set(list) => Self::Set(Set::clone(list)),
            Self::Map(list) => Self::Map(Map::clone(list)),
            Self::IFn(arg0) => Self::IFn(arg0.clone()),
        }
    }

    pub fn string(s: String) -> Self {
        Self::Str(s)
    }

    pub fn empty_list() -> Self {
        Self::List(List::empty())
    }
    pub fn list_from<V: Into<List<P>>>(list: V) -> Self {
        Self::from_list(list.into())
    }
    pub fn from_list(list: List<P>) -> Self {
        Self::List(list)
    }
    pub fn from_list_ref(list: &List<P>) -> Self {
        Self::List(List::clone(list))
    }
    pub fn from_raw_list(raw_list: RawList<P>) -> Self {
        Self::List(List::from_raw(raw_list))
    }
    pub fn list_from_values<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        Self::List(List::from_values(iter))
    }
    pub fn list_ptr_from_values<I>(iter: I) -> ValuePtr<P>
    where
        I: IntoIterator<Item = Self>,
    {
        ValuePtr::from(Self::list_from_values(iter))
    }

    pub fn empty_vect() -> Self {
        Self::Vect(Vect::empty())
    }
    pub fn vect_from<V: Into<Vect<P>>>(vect: V) -> Self {
        Self::from_vect(vect.into())
    }
    pub fn from_vect(vect: Vect<P>) -> Self {
        Self::Vect(vect)
    }
    pub fn from_vect_ref(vect: &Vect<P>) -> Self {
        Self::Vect(Vect::clone(vect))
    }
    pub fn from_raw_vect(raw_vect: RawVect<P>) -> Self {
        Self::Vect(Vect::from_raw(raw_vect))
    }
    pub fn vect_from_values<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        Self::Vect(Vect::from_values(iter))
    }
    pub fn vect_ptr_from_values<I>(iter: I) -> ValuePtr<P>
    where
        I: IntoIterator<Item = Self>,
    {
        ValuePtr::from(Self::vect_from_values(iter))
    }

    pub fn empty_set() -> Self {
        Self::Set(Set::empty())
    }
    pub fn set_from<V: Into<Set<P>>>(set: V) -> Self {
        Self::from_set(set.into())
    }
    pub fn from_set(set: Set<P>) -> Self {
        Self::Set(set)
    }
    pub fn from_set_ref(set: &Set<P>) -> Self {
        Self::Set(Set::clone(set))
    }
    pub fn from_raw_set(raw_set: RawSet<P>) -> Self {
        Self::Set(Set::from_raw(raw_set))
    }
    pub fn set_from_values<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        Self::Set(Set::from_values(iter))
    }
    pub fn set_ptr_from_values<I>(iter: I) -> ValuePtr<P>
    where
        I: IntoIterator<Item = Self>,
    {
        ValuePtr::from(Self::set_from_values(iter))
    }

    pub fn empty_map() -> Self {
        Self::Map(Map::empty())
    }
    pub fn map_from<V: Into<Map<P>>>(map: V) -> Self {
        Self::from_map(map.into())
    }
    pub fn from_map(map: Map<P>) -> Self {
        Self::Map(map)
    }
    pub fn from_map_ref(map: &Map<P>) -> Self {
        Self::Map(Map::clone(map))
    }
    pub fn from_raw_map(raw_map: RawMap<P>) -> Self {
        Self::Map(Map::from_raw(raw_map))
    }
    pub fn map_from_value_pairs<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (Self, Self)>,
    {
        Self::Map(Map::from_value_pairs(iter))
    }
    pub fn map_ptr_from_value_pairs<I>(iter: I) -> ValuePtr<P>
    where
        I: IntoIterator<Item = (Self, Self)>,
    {
        ValuePtr::from(Self::map_from_value_pairs(iter))
    }

    pub fn symbol(symbol: Symbol) -> Self {
        Self::Symbol(symbol)
    }
    pub fn unqualified_symbol(name: String) -> Self {
        Self::Symbol(Symbol::unqualified(name))
    }
    pub fn qualified_symbol(namespace: String, name: String) -> Self {
        Self::Symbol(Symbol::qualified(namespace, name))
    }

    pub fn keyword(symbol: Symbol) -> Self {
        Self::Keyword(symbol)
    }
    pub fn unqualified_keyword(name: String) -> Self {
        Self::Keyword(Symbol::unqualified(name))
    }
    pub fn qualified_keyword(namespace: String, name: String) -> Self {
        Self::Keyword(Symbol::qualified(namespace, name))
    }

    pub fn to_ptr(self) -> ValuePtr<P> {
        ValuePtr::from(self)
    }
}

impl RcValue {
    pub fn to_rc(self) -> RcValuePtr {
        RcValuePtr::from(self)
    }
}

impl ArcValue {
    pub fn to_arc(self) -> ArcValuePtr {
        ArcValuePtr::from(self)
    }
}

impl<P: SharedPointerKind> Eq for Value<P> {}

impl<P: SharedPointerKind> Hash for Value<P> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl<P: SharedPointerKind> std::fmt::Display for Value<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Value::Nil => "nil".to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Num(n) => n.to_string(),
                Value::Str(s) => format!("\"{s}\""),
                Value::Keyword(sym) => format!(":{sym}"),
                Value::Symbol(sym) => sym.to_string(),
                Value::List(list) => format!(
                    "({})",
                    list.into_iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(" ")
                ),
                Value::Vect(vect) => format!(
                    "[{}]",
                    vect.into_iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(" ")
                ),
                Value::Set(set) => format!(
                    "#{{{}}}",
                    set.into_iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(" ")
                ),
                Value::Map(map) => format!(
                    "{{{}}}",
                    map.into_iter()
                        .map(|(k, v)| vec![k.to_string(), v.to_string()].join(" "))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                Value::IFn(ifn) => {
                    let raw = Arc::into_raw(Arc::clone(ifn));
                    let fmt = format!("#object[fn,{:?}]", raw);
                    unsafe {
                        drop(Arc::from_raw(raw));
                    }
                    fmt
                }
            }
        )
    }
}
