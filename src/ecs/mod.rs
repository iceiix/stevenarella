// Copyright 2016 Matthew Collins
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use types::bit::Set as BSet;
use std::collections::HashMap;
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::mem;
use std::ptr;

/// Used to reference an entity.
#[derive(Clone, Copy)]
pub struct Entity {
    id: usize,
    generation: u32,
}

/// Stores and manages a collection of entities.
pub struct Manager {
    num_components: usize,
    entities: Vec<(Option<BSet>, u32)>,
    free_entities: Vec<usize>,
    components: Vec<Option<ComponentMem>>,

    component_ids: RefCell<HashMap<TypeId, usize>>,
}

/// Used to access compoents on an entity in an efficient
/// way.
#[derive(Clone, Copy)]
pub struct Key<T> {
    id: usize,
    _t: PhantomData<T>,
}

/// Used to search for entities with the requested components.
pub struct Filter {
    bits: BSet,
}

impl Filter {
    /// Creates an empty filter which matches everything
    pub fn new() -> Filter {
        Filter {
            bits: BSet::new(0),
        }
    }

    /// Adds the component to the filter.
    pub fn with<T>(mut self, key: Key<T>) -> Self {
        if self.bits.capacity() <= key.id {
            self.bits.resize(key.id + 1);
        }
        self.bits.set(key.id, true);
        self
    }
}

impl Manager {
    /// Creates a new manager.
    pub fn new() -> Manager {
        Manager {
            num_components: 0,
            entities: vec![(Some(BSet::new(0)), 0)], // Has the world entity pre-defined
            free_entities: vec![],
            components: vec![],

            component_ids: RefCell::new(HashMap::new()),
        }
    }

    /// Returns the world entity. This should never be removed.
    pub fn get_world(&self) -> Entity {
        Entity {
            id: 0,
            generation: 0,
        }
    }

    /// Returns all entities matching the filter
    pub fn find(&self, filter: &Filter) -> Vec<Entity> {
        let mut ret = vec![];
        // Skip the world entity.
        for (i, &(ref set, gen)) in self.entities[1..].iter().enumerate() {
            if let Some(set) = set.as_ref() {
                if set.includes_set(&filter.bits) {
                    ret.push(Entity {
                        id: i + 1,
                        generation: gen,
                    });
                }
            }
        }
        ret
    }

    /// Allocates a new entity without any components.
    pub fn create_entity(&mut self) -> Entity {
        if let Some(id) = self.free_entities.pop() {
            let entity = &mut self.entities[id];
            entity.0 = Some(BSet::new(self.num_components));
            entity.1 += 1;
            return Entity {
                id: id,
                generation: entity.1,
            }
        }
        let id = self.entities.len();
        self.entities.push((
            Some(BSet::new(self.num_components)),
            0
        ));
        Entity {
            id: id,
            generation: 0,
        }
    }

    /// Deallocates an entity and frees its components
    pub fn remove_entity(&mut self, e: Entity) {
        if let Some(set) = self.entities[e.id].0.as_ref() {
            for i in 0 .. COMPONENTS_PER_BLOCK {
                if set.get(i) {
                    self.components[i].as_mut().unwrap().remove(e.id);
                }
            }
            self.free_entities.push(e.id);
        }
        self.entities[e.id].0 = None;
    }

    /// Returns whether an entity reference is valid.
    pub fn is_entity_valid(&self, e: Entity) -> bool {
        match self.entities.get(e.id) {
            Some(val) => val.1 == e.generation && val.0.is_some(),
            None => false,
        }
    }

    /// Gets a key for the component type. Creates one
    /// if the component has never been referenced before.
    pub fn get_key<T: Any>(&self) -> Key<T> {
        let mut ids = self.component_ids.borrow_mut();
        let next_id = ids.len();
        let id = ids.entry(TypeId::of::<T>()).or_insert(next_id);
        Key {
            id: *id,
            _t: PhantomData,
        }
    }

    /// Adds the component to the target entity
    /// # Panics
    /// Panics when the target entity doesn't exist
    pub fn add_component<T>(&mut self, entity: Entity, key: Key<T>, val: T) {
        if self.components.len() <= key.id {
            while self.components.len() <= key.id {
                self.components.push(None);
            }
        }
        if self.components[key.id].is_none() {
            self.components[key.id] = Some(ComponentMem::new::<T>());
            self.num_components += 1;
            for &mut (ref mut set, _) in &mut self.entities {
                if let Some(set) = set.as_mut() {
                    set.resize(self.num_components);
                }
            }
        }
        let components = self.components.get_mut(key.id).and_then(|v| v.as_mut()).unwrap();
        let mut e = self.entities.get_mut(entity.id);
        let set = match e {
            Some(ref mut val) => if val.1 == entity.generation { &mut val.0 } else { panic!("Missing entity") },
            None => panic!("Missing entity"),
        };
        let set = match set.as_mut() {
            Some(val) => val,
            None => panic!("Missing entity"),
        };
        if set.get(key.id) {
            panic!("Duplicate add");
        }
        set.set(key.id, true);
        components.add(entity.id, val);
    }

    /// Same as `add_component` but doesn't require a key. Using a key
    /// is better for frequent lookups.
    pub fn add_component_direct<T: Any>(&mut self, entity: Entity, val: T) {
        let key = self.get_key();
        self.add_component(entity, key, val);
    }

    /// Returns the given component that the key points to if it exists.
    pub fn get_component<T>(&self, entity: Entity, key: Key<T>) -> Option<&T> {
        let components = match self.components.get(key.id).and_then(|v| v.as_ref()) {
            Some(val) => val,
            None => return None,
        };
        let set = match self.entities.get(entity.id).as_ref() {
            Some(ref val) => if val.1 == entity.generation { &val.0 } else { return None },
            None => return None,
        };
        if !set.as_ref().map(|v| v.get(key.id)).unwrap_or(false) {
            return None;
        }

        Some(components.get(entity.id))
    }

    /// Same as `get_component` but doesn't require a key. Using a key
    /// is better for frequent lookups.
    pub fn get_component_direct<T: Any>(&self, entity: Entity) -> Option<&T> {
        let key = self.get_key();
        self.get_component(entity, key)
    }

    /// Returns the given component that the key points to if it exists.
    pub fn get_component_mut<T>(&mut self, entity: Entity, key: Key<T>) -> Option<&mut T> {
        let components = match self.components.get_mut(key.id).and_then(|v| v.as_mut()) {
            Some(val) => val,
            None => return None,
        };
        let set = match self.entities.get(entity.id).as_ref() {
            Some(ref val) => if val.1 == entity.generation { &val.0 } else { return None },
            None => return None,
        };
        if !set.as_ref().map(|v| v.get(key.id)).unwrap_or(false) {
            return None;
        }

        Some(components.get_mut(entity.id))
    }

    /// Same as `get_component_mut` but doesn't require a key. Using a key
    /// is better for frequent lookups.
    pub fn get_component_mut_direct<T: Any>(&mut self, entity: Entity) -> Option<&mut T> {
        let key = self.get_key();
        self.get_component_mut(entity, key)
    }
}

const COMPONENTS_PER_BLOCK: usize = 64;

struct ComponentMem {
    data: Vec<Option<(Vec<u8>, BSet, usize)>>,
    component_size: usize,
    drop_func: Box<Fn(*mut u8)>,
}

impl ComponentMem {
    fn new<T>() -> ComponentMem {
        ComponentMem {
            data: vec![],
            component_size: mem::size_of::<T>(),
            drop_func: Box::new(|data| {
                unsafe {
                    let mut val: T = mem::uninitialized();
                    ptr::copy(data as *mut T, &mut val, 1);
                    mem::drop(val);
                }
            }),
        }
    }

    fn add<T>(&mut self, index: usize, val: T) {
        while self.data.len() < (index / COMPONENTS_PER_BLOCK) + 1 {
            self.data.push(None);
        }
        let idx = index / COMPONENTS_PER_BLOCK;
        let rem = index % COMPONENTS_PER_BLOCK;
        if self.data[idx].is_none() {
            self.data[idx] = Some((vec![0; self.component_size * COMPONENTS_PER_BLOCK], BSet::new(COMPONENTS_PER_BLOCK), 0));
        }
        let data = self.data[idx].as_mut().unwrap();
        let start = rem * self.component_size;
        data.2 += 1;
        data.1.set(rem, true);
        unsafe {
            ptr::write(data.0.as_mut_ptr().offset(start as isize) as *mut T, val);
        }
    }

    fn remove(&mut self, index: usize) {
        let idx = index / COMPONENTS_PER_BLOCK;
        let rem = index % COMPONENTS_PER_BLOCK;

        let count = {
            let data = self.data[idx].as_mut().unwrap();
            let start = rem * self.component_size;
            data.1.set(rem, false);
            // We don't have access to the actual type in this method so
            // we use the drop_func which stores the type in its closure
            // to handle the dropping for us.
            unsafe { (self.drop_func)(data.0.as_mut_ptr().offset(start as isize)); }
            data.2 -= 1;
            data.2
        };
        if count == 0 {
            self.data[idx] = None;
        }
    }

    fn get<T>(&self, index: usize) -> &T {
        let idx = index / COMPONENTS_PER_BLOCK;
        let rem = index % COMPONENTS_PER_BLOCK;
        let data = self.data[idx].as_ref().unwrap();
        let start = rem * self.component_size;
        unsafe {
            mem::transmute(data.0.as_ptr().offset(start as isize))
        }
    }

    fn get_mut<T>(&mut self, index: usize) -> &mut T {
        let idx = index / COMPONENTS_PER_BLOCK;
        let rem = index % COMPONENTS_PER_BLOCK;
        let data = self.data[idx].as_mut().unwrap();
        let start = rem * self.component_size;
        unsafe {
            mem::transmute(data.0.as_mut_ptr().offset(start as isize))
        }
    }
}

impl Drop for ComponentMem {
    fn drop(&mut self) {
        for data in &mut self.data {
            if let Some(data) = data.as_mut() {
                for i in 0 .. COMPONENTS_PER_BLOCK {
                    if data.1.get(i) {
                        let start = i * self.component_size;
                        unsafe { (self.drop_func)(data.0.as_mut_ptr().offset(start as isize)); }
                    }
                }
            }
        }
    }
}
