use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::types::MalData;

pub struct Env {
    outer: Option<Rc<RefCell<Env>>>,
    data: HashMap<String, MalData>,
}

impl Env {
    pub fn new(outer: Option<Rc<RefCell<Env>>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            outer,
            data: HashMap::new(),
        }))
    }

    pub fn set(&mut self, key: String, value: MalData) {
        self.data.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<MalData> {
        if let Some(value) = self.data.get(key) {
            Some(value.clone())
        } else if let Some(outer) = &self.outer {
            outer.borrow().get(key).clone()
        } else {
            None
        }
    }
}
