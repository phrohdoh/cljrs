use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    /// Clojure example:
    /// ```clojure
    /// :foo
    /// ```
    Unqualified { name: String },
    //
    /// Clojure example:
    /// ```clojure
    /// :foo/bar
    /// ```
    Qualified { namespace: String, name: String },
    //
    /// Clojure example:
    /// ```clojure
    /// (ns something.xyz)
    /// ::foo
    /// ```
    SelfQualified { name: String },
    //
    /// Clojure example:
    /// ```clojure
    /// (ns something.xyz (:require [something.abc :as abc]))
    /// ::abc/foo
    /// ```
    AliasQualified { alias: String, name: String },
}

impl Keyword {
    pub fn unqualified(name: String) -> Self {
        Self::Unqualified { name }
    }
    pub fn qualified(namespace: String, name: String) -> Self {
        Self::Qualified { namespace, name }
    }
    pub fn self_qualified(name: String) -> Self {
        Self::SelfQualified { name }
    }
    pub fn alias_qualified(alias: String, name: String) -> Self {
        Self::AliasQualified { alias, name }
    }
}

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}",
            match self {
                Self::Unqualified { name } => format!(":{name}"),
                Self::Qualified { namespace, name } => format!(":{namespace}/{name}"),
                Self::SelfQualified { name } => format!("::{name}"),
                Self::AliasQualified { alias, name } => format!("::{alias}/{name}"),
            }
        )
    }
}
