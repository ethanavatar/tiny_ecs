use std::cell::{RefCell, RefMut};
use std::collections::HashMap;

use crate::component::Component;
use crate::component_storage::ComponentStorage;

pub struct Entity {
    id: usize,
    has_components: Vec<std::any::TypeId>,
}

pub struct World {
    entity_count: usize,
    entities: Vec<Entity>,
    components: HashMap<std::any::TypeId, Box<dyn ComponentStorage>>,
    free_entity_slots: Vec<usize>,
}

impl World {
    pub fn new() -> Self {
        World {
            entity_count: 0,
            entities: Vec::new(),
            components: HashMap::new(),
            free_entity_slots: Vec::new(),
        }
    }

    pub fn new_entity(
        &mut self,
    ) -> Entity {
        if let Some(entity_id) = self.free_entity_slots.pop() {
            return Entity {
                id: entity_id,
                has_components: Vec::new(),
            };
        }

        for (_id, c) in self.components.iter_mut() {
            c.push_none();
        }

        self.entity_count += 1;
        Entity {
            id: self.entity_count - 1,
            has_components: Vec::new(),
        }
    }

    pub fn remove_entity(&mut self, entity_id: usize) {
        for (_id, c) in self.components.iter_mut() {
            c.none_at(entity_id);
        }

        self.free_entity_slots.push(entity_id);
    }

    pub fn count_entities(&self) -> usize {
        self.entity_count - self.free_entity_slots.len()
    }

    pub fn add_component<T: Component>(
        &mut self,
        entity: &Entity,
        component: T,
    ) {
        let component_id = std::any::TypeId::of::<T>();
        let c = self.components.entry(component_id)
            .or_insert_with(|| {
                let new_storage: Vec<Option<T>> = vec![None; self.entity_count];
                Box::new(RefCell::new(new_storage))
            });

        if let Some(c) = c.as_any_mut()
            .downcast_mut::<RefCell<Vec<Option<T>>>>()
        {
            c.get_mut()[entity.id] = Some(component);
            return;
        }

    }

    fn borrow_storage<T: Component>(
        &self,
    ) -> Option<&RefCell<Vec<Option<T>>>> {
        let component_id = std::any::TypeId::of::<T>();
        let c = self.components.get(&component_id)?;
        if let Some(c) = c.as_any()
            .downcast_ref::<RefCell<Vec<Option<T>>>>()
        {
            return Some(c);
        }

        None
    }

    pub fn borrow_components<T: Component>(
        &self,
    ) -> Option<RefMut<Vec<Option<T>>>> {
        self.borrow_storage::<T>()
            .map(|c| c.borrow_mut())
    }

    pub fn borrow_components_mut<T: Component>(
        &self,
    ) -> Option<RefMut<Vec<Option<T>>>> {
        self.borrow_storage::<T>()
            .map(|c| c.borrow_mut())
    }
}

