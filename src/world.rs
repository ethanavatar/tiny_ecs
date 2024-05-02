use std::any::TypeId;
use std::marker::PhantomData;
use std::collections::HashMap;
use std::cell::{RefCell, RefMut};

use crate::component::Component;
use crate::component_storage::ComponentStorage;

#[derive(Debug, Clone)]
pub struct ComponentPointer<T: Component> {
    entity_id: usize,
    component_type: PhantomData<T>,
}

impl<T: Component> ComponentPointer<T> {
    pub fn to(entity_id: usize) -> Self {
        ComponentPointer { entity_id, component_type: PhantomData }
    }

    pub fn get(&self, world: &World) -> Option<T> {
        world.get_component::<T>(self.entity_id)
    }
}

pub struct World {
    entity_count: usize,
    components: HashMap<TypeId, Box<dyn ComponentStorage>>,
    free_entity_slots: Vec<usize>,
}

impl World {
    pub fn new() -> Self {
        World {
            entity_count: 0,
            components: HashMap::new(),
            free_entity_slots: Vec::new(),
        }
    }

    pub fn new_entity(
        &mut self,
    ) -> usize {
        if let Some(entity_id) = self.free_entity_slots.pop() {
            return entity_id;
        }

        self.components.iter_mut()
            .for_each(|(_, c)| c.push_none());

        self.entity_count += 1;
        self.entity_count - 1
    }

    pub fn remove_entity(&mut self, entity_id: usize) {
        self.components.iter_mut()
            .for_each(|(_, c)| c.none_at(entity_id));

        self.free_entity_slots.push(entity_id);
    }

    pub fn count_entities(&self) -> usize {
        self.entity_count - self.free_entity_slots.len()
    }

    pub fn get_component<T: Component>(
        &self,
        entity_id: usize,
    ) -> Option<T> {
        let component_id = TypeId::of::<T>();
        self.components.get(&component_id)
            .map(|c| c.as_any()
                .downcast_ref::<RefCell<Vec<Option<T>>>>()
                .map(|c| c.borrow()[entity_id].clone())
                .flatten())
            .flatten()
    }

    pub fn replace_component<T: Component>(
        &mut self,
        entity_id: usize,
        component: T,
    ) {
        let component_id = TypeId::of::<T>();
        if let Some(bucket) = self.components.get_mut(&component_id) {
            bucket.as_any_mut()
                .downcast_mut::<RefCell<Vec<Option<T>>>>()
                .map(|c| c.get_mut()[entity_id] = Some(component));
        } else {
            self.add_component(entity_id, component);
        }
    }

    pub fn entity_has<T: Component>(&self, entity_id: usize) -> bool {
        self.get_component::<T>(entity_id).is_some()
    }

    pub fn add_component<T: Component>(
        &mut self,
        entity_id: usize,
        component: T,
    ) -> ComponentPointer<T> {
        let component_id = TypeId::of::<T>();
        self.components.entry(component_id)
            .or_insert_with(|| {
                let new_storage: Vec<Option<T>> = vec![None; self.entity_count];
                Box::new(RefCell::new(new_storage))
            })
            .as_any_mut()
            .downcast_mut::<RefCell<Vec<Option<T>>>>()
            .map(|c| c.get_mut()[entity_id] = Some(component));

        ComponentPointer::<T>::to(entity_id)
    }

    pub fn remove_component<T: Component>(
        &mut self,
        entity_id: usize,
    ) {
        let component_id = TypeId::of::<T>();
        self.components.get(&component_id)
            .map(|c| c.none_at(entity_id));
    }

    fn borrow_storage<T: Component>(
        &self,
    ) -> Option<&RefCell<Vec<Option<T>>>> {
        let component_id = TypeId::of::<T>();
        self.components.get(&component_id)?
            .as_any()
            .downcast_ref::<RefCell<Vec<Option<T>>>>()
            .map(|c| return c)
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

