use std::{cell::{RefCell, RefMut}, collections::HashMap};

trait ComponentStorage {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn push_none(&mut self);
    fn none_at(&self, index: usize);
}

impl<T: 'static> ComponentStorage for RefCell<Vec<Option<T>>> {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
    fn push_none(&mut self) {
        self.get_mut().push(None);
    }
    fn none_at(&self, index: usize) {
        self.borrow_mut()[index] = None;
    }
}

pub struct World {
    entity_count: usize,
    components: HashMap<std::any::TypeId, Box<dyn ComponentStorage>>,
    free_entities: Vec<usize>,
}

impl World {
    pub fn new() -> Self {
        World {
            entity_count: 0,
            components: HashMap::new(),
            free_entities: Vec::new(),
        }
    }

    pub fn new_entity(
        &mut self,
    ) -> usize {
        if let Some(entity_id) = self.free_entities.pop() {
            return entity_id;
        }

        for (_id, c) in self.components.iter_mut() {
            c.push_none();
        }

        self.entity_count += 1;
        self.entity_count - 1
    }

    pub fn remove_entity(&mut self, entity_id: usize) {
        for (_id, c) in self.components.iter_mut() {
            c.none_at(entity_id);
        }

        self.free_entities.push(entity_id);
    }

    pub fn count_entities(&self) -> usize {
        self.entity_count - self.free_entities.len()
    }

    pub fn add_component<T: 'static>(
        &mut self,
        entity_id: usize,
        component: T,
    ) {
        let component_id = std::any::TypeId::of::<T>();
        let c = self.components.entry(component_id)
            .or_insert_with(|| {
                let mut new_storage: Vec<Option<T>> = Vec::with_capacity(self.entity_count);
                for _ in 0..self.entity_count {
                    new_storage.push(None);
                }

                Box::new(RefCell::new(new_storage))
            });

        if let Some(c) = c
            .as_any_mut()
            .downcast_mut::<RefCell<Vec<Option<T>>>>()
        {
            c.get_mut()[entity_id] = Some(component);
            return;
        }

    }

    pub fn borrow_components<T: 'static>(
        &self,
    ) -> Option<RefMut<Vec<Option<T>>>> {
        let component_id = std::any::TypeId::of::<T>();
        let c = self.components.get(&component_id)?;
        if let Some(c) = c
            .as_any()
            .downcast_ref::<RefCell<Vec<Option<T>>>>()
        {
            return Some(c.borrow_mut());
        }

        None
    }
}

pub struct Systems {
    systems: Vec<Box<dyn Fn(&mut World)>>,
}

impl Systems {
    pub fn new() -> Self {
        Systems {
            systems: Vec::new(),
        }
    }

    pub fn add_system<F: 'static>(&mut self, system: F)
    where
        F: Fn(&mut World),
    {
        self.systems.push(Box::new(system));
    }

    pub fn run(&self, world: &mut World) {
        self.systems.iter().for_each(|s| s(world));
    }
}
