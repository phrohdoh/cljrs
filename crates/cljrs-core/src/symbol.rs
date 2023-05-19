use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Symbol {
    /// Clojure example:
    /// ```clojure
    /// foo
    /// ```
    Unqualified { name: String },
    //
    /// Clojure example:
    /// ```clojure
    /// foo/bar
    /// ```
    Qualified { namespace: String, name: String },
}

// constructors
impl Symbol {
    pub fn qualified<NS: Into<String>, N: Into<String>>(namespace: NS, name: N) -> Self {
        let (namespace, name) = (namespace.into(), name.into());
        Self::Qualified { namespace, name }
    }
    pub fn unqualified<N: Into<String>>(name: N) -> Self {
        let name = name.into();
        Self::Unqualified { name }
    }
}

// data readers
impl Symbol {
    pub fn try_namespace(&self) -> Option<String> {
        match self {
            Self::Unqualified { .. } => None,
            Self::Qualified { namespace, .. } => Some(namespace.clone()),
        }
    }
    pub fn name(&self) -> String {
        match self {
            Self::Unqualified { name } => name.clone(),
            Self::Qualified { name, .. } => name.clone(),
        }
    }
}

// potentially panicing data readers
impl Symbol {
    /// panics if this [`Symbol`] has no namespace
    pub fn namespace(&self) -> String {
        self.try_namespace().unwrap()
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Symbol::Unqualified { name } => name.to_owned(),
                Symbol::Qualified { namespace, name } => format!("{namespace}/{name}"),
            }
        )
    }
}
