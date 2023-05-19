use std::fmt;
use cljrs_core::symbol::Symbol;
use archery::{ArcK, RcK, SharedPointer, SharedPointerKind};
use super::keyword::Keyword;

pub type List<P> = Vec<ValuePtr<P>>;
pub type Vect<P> = Vec<ValuePtr<P>>;
pub type Set<P>  = Vec<ValuePtr<P>>;
pub type Map<P>  = Vec<(ValuePtr<P>, ValuePtr<P>)>;

// TODO: impl Clone https://github.com/orium/archery/issues/26
#[derive(Debug)]
pub enum Value<P: SharedPointerKind> {
    Nil,
    Bool(bool),
    Num(f64),
    Str(String),
    Keyword(Keyword),
    Symbol(Symbol),
    List(List<P>),
    Vect(Vect<P>),
    Set(Set<P>),
    Map(Map<P>),
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
}

impl<P: SharedPointerKind> Value<P> {
    pub fn try_as_list(&self) -> Option<&List<P>> {
        match self {
            Self::List(list) => Some(list),
            _ => None,
        }
    }
    pub fn as_list(&self) -> &List<P> {
        self.try_as_list().unwrap()
    }
    pub fn try_into_list(self) -> Option<List<P>> {
        match self {
            Self::List(list) => Some(list),
            _ => None,
        }
    }
    pub fn into_list(self) -> List<P> {
        self.try_into_list().unwrap()
    }

    pub fn try_as_vect(&self) -> Option<&Vect<P>> {
        match self {
            Self::Vect(vect) => Some(vect),
            _ => None,
        }
    }
    pub fn as_vect(&self) -> &Vect<P> {
        self.try_as_vect().unwrap()
    }
    pub fn try_into_vect(self) -> Option<Vect<P>> {
        match self {
            Self::Vect(vect) => Some(vect),
            _ => None,
        }
    }
    pub fn into_vect(self) -> Vect<P> {
        self.try_into_vect().unwrap()
    }

    pub fn try_as_set(&self) -> Option<&Set<P>> {
        match self {
            Self::Set(set) => Some(set),
            _ => None,
        }
    }
    pub fn as_set(&self) -> &Set<P> {
        self.try_as_set().unwrap()
    }
    pub fn try_into_set(self) -> Option<Set<P>> {
        match self {
            Self::Set(set) => Some(set),
            _ => None,
        }
    }
    pub fn into_set(self) -> Set<P> {
        self.try_into_set().unwrap()
    }

    pub fn try_as_map(&self) -> Option<&Map<P>> {
        match self {
            Self::Map(map) => Some(map),
            _ => None,
        }
    }
    pub fn as_map(&self) -> &Map<P> {
        self.try_as_map().unwrap()
    }
    pub fn try_into_map(self) -> Option<Map<P>> {
        match self {
            Self::Map(map) => Some(map),
            _ => None,
        }
    }
    pub fn into_map(self) -> Map<P> {
        self.try_into_map().unwrap()
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
            Self::List(arg0) => Self::List(arg0.clone()),
            Self::Vect(arg0) => Self::Vect(arg0.clone()),
            Self::Set(arg0) => Self::Set(arg0.clone()),
            Self::Map(arg0) => Self::Map(arg0.clone()),
        }
    }

    pub fn string(s: String) -> Self {
        Self::Str(s)
    }

    pub fn empty_list() -> Self {
        Self::List(vec![])
    }
    pub fn list_from_values<I>(iter: I) -> Self where I: IntoIterator<Item = Self> {
        Self::List(iter.into_iter().map(ValuePtr::from).collect())
    }
    pub fn list_from_value_ptrs<I>(iter: I) -> Self where I: IntoIterator<Item = ValuePtr<P>> {
        Self::List(iter.into_iter().collect())
    }
    pub fn list_ptr_from_values<I>(iter: I) -> ValuePtr<P> where I: IntoIterator<Item = Self> {
        ValuePtr::from(Self::List(iter.into_iter().map(ValuePtr::from).collect()))
    }
    pub fn list_ptr_from_value_ptrs<I>(iter: I) -> ValuePtr<P> where I: IntoIterator<Item = ValuePtr<P>> {
        ValuePtr::from(Self::List(iter.into_iter().collect()))
    }

    pub fn empty_vect() -> Self {
        Self::Vect(vec![])
    }
    pub fn vect_from_values<I>(iter: I) -> Self where I: IntoIterator<Item = Self> {
        Self::Vect(iter.into_iter().map(ValuePtr::from).collect())
    }
    pub fn vect_from_value_ptrs<I>(iter: I) -> Self where I: IntoIterator<Item = ValuePtr<P>> {
        Self::Vect(iter.into_iter().collect())
    }
    pub fn vect_ptr_from_values<I>(iter: I) -> ValuePtr<P> where I: IntoIterator<Item = Self> {
        ValuePtr::from(Self::Vect(iter.into_iter().map(ValuePtr::from).collect()))
    }
    pub fn vect_ptr_from_value_ptrs<I>(iter: I) -> ValuePtr<P> where I: IntoIterator<Item = ValuePtr<P>> {
        ValuePtr::from(Self::Vect(iter.into_iter().collect()))
    }

    pub fn empty_set() -> Self {
        Self::Set(vec![])
    }
    pub fn set_from_values<I>(iter: I) -> Self where I: IntoIterator<Item = Self> {
        Self::Set(iter.into_iter().map(ValuePtr::from).collect())
    }
    pub fn set_from_value_ptrs<I>(iter: I) -> Self where I: IntoIterator<Item = ValuePtr<P>> {
        Self::Set(iter.into_iter().collect())
    }
    pub fn set_ptr_from_values<I>(iter: I) -> ValuePtr<P> where I: IntoIterator<Item = Self> {
        ValuePtr::from(Self::Set(iter.into_iter().map(ValuePtr::from).collect()))
    }
    pub fn set_ptr_from_value_ptrs<I>(iter: I) -> ValuePtr<P> where I: IntoIterator<Item = ValuePtr<P>> {
        ValuePtr::from(Self::Set(iter.into_iter().collect()))
    }

    pub fn empty_map() -> Self {
        Self::Map(vec![])
    }
    pub fn map_from_value_pairs<I>(iter: I) -> Self where I: IntoIterator<Item = (Self, Self)> {
        Self::Map(iter.into_iter().map(|(k, v)| (ValuePtr::from(k), ValuePtr::from(v))).collect())
    }
    pub fn map_from_value_ptr_pairs<I>(iter: I) -> Self where I: IntoIterator<Item = (ValuePtr<P>, ValuePtr<P>)> {
        Self::Map(iter.into_iter().collect())
    }
    pub fn map_ptr_from_values<I>(iter: I) -> ValuePtr<P> where I: IntoIterator<Item = (Self, Self)> {
        ValuePtr::from(Self::Map(iter.into_iter().map(|(k, v)| (ValuePtr::from(k), ValuePtr::from(v))).collect()))
    }
    pub fn map_ptr_from_value_ptrs<I>(iter: I) -> ValuePtr<P> where I: IntoIterator<Item = (ValuePtr<P>, ValuePtr<P>)> {
        ValuePtr::from(Self::Map(iter.into_iter().collect()))
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

    pub fn keyword(keyword: Keyword) -> Self {
        Self::Keyword(keyword)
    }
    pub fn unqualified_keyword(name: String) -> Self {
        Self::Keyword(Keyword::unqualified(name))
    }
    pub fn qualified_keyword(namespace: String, name: String) -> Self {
        Self::Keyword(Keyword::qualified(namespace, name))
    }
    pub fn self_qualified_keyword(name: String) -> Self {
        Self::Keyword(Keyword::self_qualified(name))
    }
    pub fn alias_qualified_keyword(alias: String, name: String) -> Self {
        Self::Keyword(Keyword::alias_qualified(alias, name))
    }

    pub fn to_ptr(self) -> ValuePtr<P> {
        ValuePtr::from(self)
    }
}

// deriving PartialEq does not work (?), so using (mostly) auto-generated impl
impl<P: SharedPointerKind> PartialEq for Value<P> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Nil, Self::Nil) => true,
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::Num(l0), Self::Num(r0)) => l0 == r0,
            (Self::Str(l0), Self::Str(r0)) => l0 == r0,
            (Self::Keyword(l0), Self::Keyword(r0)) => l0 == r0,
            (Self::Symbol(l0), Self::Symbol(r0)) => l0 == r0,
            (Self::List(l0), Self::List(r0)) => l0 == r0,
            (Self::Vect(l0), Self::Vect(r0)) => l0 == r0,
            (Self::Set(l0), Self::Set(r0)) => l0 == r0,
            (Self::Map(l0), Self::Map(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl<P: SharedPointerKind> fmt::Display for Value<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Value::Nil => "nil".to_owned(),
                Value::Bool(b) => b.to_string(),
                Value::Num(n) => n.to_string(),
                Value::Str(s) => format!("\"{s}\""),
                Value::Keyword(k) => format!("{}", k),
                Value::Symbol(s) => s.to_string(),
                Value::List(vs) => {
                    let contents = vs
                        .iter()
                        .map(|v| format!("{v}"))
                        .collect::<Vec<_>>()
                        .join(" ");
                    format!("({contents})")
                }
                Value::Vect(vs) => {
                    let contents = vs
                        .iter()
                        .map(|v| format!("{v}"))
                        .collect::<Vec<_>>()
                        .join(" ");
                    format!("[{contents}]")
                }
                Value::Set(vs) => {
                    let contents = vs
                        .iter()
                        .map(|v| format!("{v}"))
                        .collect::<Vec<_>>()
                        .join(" ");
                    format!("#{{{contents}}}")
                }
                Value::Map(kvs) => {
                    let contents = kvs
                        .iter()
                        .map(|(k, v)| format!("{k} {v}"))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{{{contents}}}")
                }
            }
        )
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

pub type RcValue = Value<RcK>;
pub type ArcValue = Value<ArcK>;

pub type ValuePtr<P> = SharedPointer<Value<P>, P>;
pub type RcValuePtr = SharedPointer<Value<RcK>, RcK>;
pub type ArcValuePtr = SharedPointer<Value<ArcK>, ArcK>;

pub type RcValuePtrs = Vec<SharedPointer<Value<RcK>, ArcK>>;
pub type ArcValuePtrs = Vec<SharedPointer<Value<ArcK>, ArcK>>;
