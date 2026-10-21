use std::{collections::HashMap, rc::Rc};

/// An interned procedure or variable name. Variables and procedures are kept
/// in vectors indexed by it, so looking one up is an array access.
pub type Sym = u32;

/// Names are case-insensitive, looked up by their lowercase form, and keep
/// the spelling they were first seen with for display (ADR 0004).
#[derive(Default)]
pub struct Names {
    map: HashMap<String, Sym>,
    spelling: Vec<Rc<str>>,
}

impl Names {
    pub fn intern(&mut self, name: &str) -> Sym {
        let key = name.to_lowercase();
        if let Some(&sym) = self.map.get(&key) {
            return sym;
        }
        let sym = self.spelling.len() as Sym;
        self.spelling.push(name.into());
        self.map.insert(key, sym);
        sym
    }

    pub fn spelling(&self, sym: Sym) -> &Rc<str> {
        &self.spelling[sym as usize]
    }

    pub fn len(&self) -> usize {
        self.spelling.len()
    }
}
