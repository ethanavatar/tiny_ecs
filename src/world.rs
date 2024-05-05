use std::any::TypeId;
use std::collections::HashMap;
use std::cell::{RefCell, RefMut};
use std::rc::Rc;

use crate::component::Component;
use crate::component_storage::ComponentStorage;

#[derive(Debug, Clone, PartialEq)]
pub struct ComponentHandle {
    entity_id: usize,
    component_type: TypeId,
}

impl ComponentHandle {
    pub fn to<T: Component>(entity_id: usize) -> Self {
        ComponentHandle { entity_id, component_type: TypeId::of::<T>() }
    }

    pub fn repoint<T: Component>(&mut self, entity_id: usize) {
        self.entity_id = entity_id;
        self.component_type = TypeId::of::<T>();
    }

    pub fn get<T: Component>(&self, world: &World) -> Option<T> {
        world.get_component::<T>(self.entity_id)
    }

    pub fn equals<T: Component>(&self, other: &ComponentHandle) -> bool {
        self.entity_id == other.entity_id
        && self.component_type == other.component_type
    }

    pub fn component_type(&self) -> TypeId {
        self.component_type
    }
}

pub struct World {
    entity_count: usize,
    active_handles: HashMap<TypeId, Vec<Rc<RefCell<ComponentHandle>>>>,
    components: HashMap<TypeId, Box<dyn ComponentStorage>>,
    free_entity_slots: Vec<usize>,
}

impl World {
    pub fn new() -> Self {
        World {
            entity_count: 0,
            active_handles: HashMap::new(),
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

    pub fn repoint_any<OldT: Component, NewT: Component>(
        &mut self,
        old_entity_id: usize,
        new_entity_id: usize,
    ) {
        let old_component_type = TypeId::of::<OldT>();
        let old_handle = ComponentHandle::to::<OldT>(old_entity_id);
        if let Some(handles) = self.active_handles.get_mut(&old_component_type) {
            handles.iter_mut()
                .for_each(|h| {
                    if h.borrow().equals::<OldT>(&old_handle) {
                        h.borrow_mut().repoint::<NewT>(new_entity_id);
                    }
                });
        }

        self.release_orphaned_handles::<OldT>();
    }

    pub fn release_orphaned_handles<T: Component>(&mut self) {

        // FIXME: This implementation feel pretty clunky. Im sure it can be better

        let component_type = TypeId::of::<T>();

        if let Some(handles) = self.active_handles.get_mut(&component_type) {
            handles.retain(|h| {
                let entity_id = h.borrow().entity_id;
                let is_orphan = self.components.get(&component_type)
                    .map(|c| c.as_any()
                        .downcast_ref::<RefCell<Vec<Option<T>>>>()
                        .map(|c| c.borrow()[entity_id].is_none())
                        .unwrap_or(true))
                    .unwrap_or(true);

                !is_orphan
            });
        }
    }

    pub fn entity_has<T: Component>(&self, entity_id: usize) -> bool {
        self.get_component::<T>(entity_id).is_some()
    }

    pub fn add_component<T: Component>(
        &mut self,
        entity_id: usize,
        component: T,
    ) -> Rc<RefCell<ComponentHandle>> {
        let component_id = TypeId::of::<T>();
        self.components.entry(component_id)
            .or_insert_with(|| {
                let new_storage: Vec<Option<T>> = vec![None; self.entity_count];
                Box::new(RefCell::new(new_storage))
            })
            .as_any_mut()
            .downcast_mut::<RefCell<Vec<Option<T>>>>()
            .map(|c| c.get_mut()[entity_id] = Some(component));

        let handle = ComponentHandle::to::<T>(entity_id);
        let handle = Rc::new(RefCell::new(handle));
        self.active_handles.entry(component_id)
            .or_insert_with(Vec::new)
            .push(handle.clone());

        handle
    }

    pub fn remove_component<T: Component>(
        &mut self,
        entity_id: usize,
    ) {
        let component_id = TypeId::of::<T>();
        self.components.get(&component_id)
            .map(|c| c.none_at(entity_id));

        if let Some(handles) = self.active_handles.get_mut(&component_id) {
            handles.retain(|h| h.borrow().entity_id != entity_id);
        }
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

