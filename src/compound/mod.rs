use std::{
    any::{Any, TypeId},
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::{
        Arc, RwLock,
        atomic::{AtomicU64, Ordering},
    },
};

type EntityId = AtomicU64;
pub type Entity = u64;

// Component Wrapper that allows safe concurrent access
pub struct MoleculeCell<T: Send + Sync + 'static> {
    data: RwLock<T>,
}

impl<T: Send + Sync + 'static> MoleculeCell<T> {
    fn new(data: T) -> Self {
        Self {
            data: RwLock::new(data),
        }
    }

    fn get_mut(&self) -> impl DerefMut<Target = T> + '_ {
        self.data.write().unwrap()
    }

    fn get(&self) -> impl Deref<Target = T> + '_ {
        self.data.read().unwrap()
    }
}

// Storage for a single component type
pub struct MoleculeStorage<T: Send + Sync + 'static> {
    compounds: HashMap<Entity, MoleculeCell<T>>,
}

impl<T: Send + Sync + 'static> MoleculeStorage<T> {
    fn new() -> Self {
        Self {
            compounds: HashMap::new(),
        }
    }
}

pub struct Compound {
    num_entities: EntityId,
    storages: RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
}

impl Compound {
    pub fn new() -> Self {
        Self {
            num_entities: AtomicU64::new(0),
            storages: RwLock::new(HashMap::new()),
        }
    }

    pub fn create_entity(&mut self) -> Entity {
        let entity_id = self.num_entities.fetch_add(1, Ordering::Relaxed);

        entity_id
    }

    // Get or create storage for a component type
    fn get_or_create_storage<T: Send + Sync + 'static>(&self) -> Arc<RwLock<MoleculeStorage<T>>> {
        let mut storages = self.storages.write().unwrap();
        let type_id = TypeId::of::<T>();

        if !storages.contains_key(&type_id) {
            let storage = MoleculeStorage::<T>::new();
            storages.insert(type_id, Box::new(Arc::new(RwLock::new(storage))));
        }

        let any_storage = unsafe {
            // Safe because we create it if it doesn't exist
            storages.get(&type_id).unwrap_unchecked()
        };

        let storage = unsafe {
            // Safe because we create it if it doesn't exist
            any_storage
                .downcast_ref::<Arc<RwLock<MoleculeStorage<T>>>>()
                .unwrap_unchecked()
        };

        storage.clone()
    }

    pub fn add_molecule<T: Send + Sync + 'static>(&self, entity: Entity, molecule: T) {
        unsafe {
            self.get_or_create_storage::<T>()
                .write()
                .unwrap_unchecked()
                .compounds
                .insert(entity, MoleculeCell::new(molecule));
        };
    }

    pub fn for_each_molecule<T, F>(&self, mut f: F)
    where
        T: Send + Sync + 'static,
        F: FnMut(Entity, &mut T) + Send + Sync,
    {
        let storage = self.get_or_create_storage::<T>();
        let storage_guard = unsafe { storage.read().unwrap_unchecked() };

        for (entity, cell) in &storage_guard.compounds {
            let mut data = cell.get_mut();
            f(*entity, &mut *data);
        }
    }
}

#[cfg(test)]
mod ecs_test {
    use super::*;

    #[test]
    fn test_ecs_basics() {
        struct Label {
            name: String,
            id: u32,
        }

        struct Collar {
            name: String,
            address: String,
        }

        struct Whiskers {
            color: String,
            number: u32,
        }

        let mut compound = Compound::new();

        let cat = compound.create_entity();
        let dog = compound.create_entity();

        compound.add_molecule(
            cat,
            Label {
                name: "John".to_string(),
                id: 0,
            },
        );

        compound.add_molecule(
            cat,
            Whiskers {
                color: "Black".to_string(),
                number: 8,
            },
        );

        compound.add_molecule(
            dog,
            Label {
                name: "Sparky".to_string(),
                id: 1,
            },
        );

        compound.add_molecule(
            dog,
            Collar {
                name: "Sparky".to_string(),
                address: "1 main St.".to_string(),
            },
        );

        compound.for_each_molecule(|entity, label: &mut Label| {
            println!("Name: {}", label.name);
            println!("Id: {}", label.id);
        });
    }
}
