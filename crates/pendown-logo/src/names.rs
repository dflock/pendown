use std::{collections::HashMap, rc::Rc};

/// An interned procedure or variable name. Variables and procedures are kept
/// in vectors indexed by it, so looking one up is an array access.
pub type Sym = u32;

/// Names are case-insensitive, looked up by their lowercase form (ADR 0004).
/// For display, a name keeps the spelling of the first `TO` or `MAKE` that
/// created it. Until then it uses the spelling it was first seen with.
#[derive(Default)]
pub struct Names {
    map: HashMap<String, Sym>,
    spelling: Vec<Rc<str>>,
    /// Whether a `TO` or `MAKE` has set the spelling since the last Run.
    defined: Vec<bool>,
}

impl Names {
    /// Looks up a name that's being used, not defined.
    pub fn intern(&mut self, name: &str) -> Sym {
        let key = name.to_lowercase();
        if let Some(&sym) = self.map.get(&key) {
            return sym;
        }
        let sym = self.spelling.len() as Sym;
        self.spelling.push(name.into());
        self.defined.push(false);
        self.map.insert(key, sym);
        sym
    }

    /// Looks up a name that a `TO` (or one of its inputs) or a `MAKE` is
    /// creating. The first one decides the spelling.
    pub fn define(&mut self, name: &str) -> Sym {
        let sym = self.intern(name);
        if !self.defined[sym as usize] {
            self.defined[sym as usize] = true;
            self.spelling[sym as usize] = name.into();
        }
        sym
    }

    /// Run clears every procedure and variable (ADR 0006), so the next
    /// definitions decide the spellings again.
    pub fn forget_definitions(&mut self) {
        self.defined.iter_mut().for_each(|d| *d = false);
    }

    pub fn spelling(&self, sym: Sym) -> &Rc<str> {
        &self.spelling[sym as usize]
    }

    pub fn len(&self) -> usize {
        self.spelling.len()
    }
}
