use std::cell::RefCell;
use crate::component::Component;

pub trait ComponentStorage {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn push_none(&mut self);
    fn none_at(&self, index: usize);
}

impl<T: Component> ComponentStorage for RefCell<Vec<Option<T>>> {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
    fn push_none(&mut self) {
        self.get_mut().push(None);
    }
    fn none_at(&self, index: usize) {
        self.borrow_mut()[index] = None;
    }
}
