use cljrs_core::symbol::Symbol;

/// ```clojure
/// :foo
/// :foo/bar
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Keyword(Symbol);

impl Keyword {
    pub fn from_symbol(symbol: Symbol) -> Self {
        Self(symbol)
    }
    pub fn unqualified(name: String) -> Self {
        Self(Symbol::unqualified(name))
    }
    pub fn qualified(namespace: String, name: String) -> Self {
        Self(Symbol::qualified(namespace, name))
    }
    pub fn symbol(&self) -> Symbol {
        self.0.clone()
    }
    pub fn namespace(&self) -> Option<String> {
        self.0.try_namespace()
    }
    pub fn name(&self) -> String {
        self.0.name()
    }
}

impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ":{sym}", sym = self.0)
    }
}
