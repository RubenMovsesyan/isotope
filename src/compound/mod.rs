#![allow(dead_code)]

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::{
        Arc, RwLock,
        atomic::{AtomicU64, Ordering},
    },
};

use anyhow::{Result, anyhow};

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

#[derive(Debug)]
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
        // Safety: storages will already exist at this point
        let mut storages = unsafe { self.storages.write().unwrap_unchecked() };
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

    // Gets the storage and Err if it doesn't exist
    fn get_storage<T: Send + Sync + 'static>(&self) -> Result<Arc<RwLock<MoleculeStorage<T>>>> {
        // Safety: storages will already exist at this point
        let storages = unsafe { self.storages.write().unwrap_unchecked() };

        let type_id = TypeId::of::<T>();

        if !storages.contains_key(&type_id) {
            return Err(anyhow!("Storage does not exist"));
        }

        // Safe because we exit function if it doesn't exist

        let any_storage = unsafe { storages.get(&type_id).unwrap_unchecked() };

        let storage = unsafe {
            any_storage
                .downcast_ref::<Arc<RwLock<MoleculeStorage<T>>>>()
                .unwrap_unchecked()
        };

        Ok(storage.clone())
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
        F: FnMut(Entity, &T) + Send + Sync,
    {
        let storage = self.get_or_create_storage::<T>();
        let storage_guard = unsafe { storage.read().unwrap_unchecked() };

        for (entity, cell) in &storage_guard.compounds {
            let data = cell.get();
            f(*entity, &*data);
        }
    }

    pub fn for_each_duo<T1, T2, F>(&self, mut f: F)
    where
        T1: Send + Sync + 'static,
        T2: Send + Sync + 'static,
        F: FnMut(Entity, &T1, &T2) + Send + Sync,
    {
        let t1_storage = self.get_or_create_storage::<T1>();
        let t2_storage = self.get_or_create_storage::<T2>();

        let t1_storage_guard = unsafe { t1_storage.read().unwrap_unchecked() };
        let t2_storage_guard = unsafe { t2_storage.read().unwrap_unchecked() };

        for (entity, t1_cell) in &t1_storage_guard.compounds {
            if let Some(t2_cell) = t2_storage_guard.compounds.get(entity) {
                let t1_data = t1_cell.get();
                let t2_data = t2_cell.get();

                f(*entity, &*t1_data, &*t2_data);
            }
        }
    }

    pub fn for_each_trio<T1, T2, T3, F>(&self, mut f: F)
    where
        T1: Send + Sync + 'static,
        T2: Send + Sync + 'static,
        T3: Send + Sync + 'static,
        F: FnMut(Entity, &T1, &T2, &T3) + Send + Sync,
    {
        let t1_storage = self.get_or_create_storage::<T1>();
        let t2_storage = self.get_or_create_storage::<T2>();
        let t3_storage = self.get_or_create_storage::<T3>();

        let t1_storage_guard = unsafe { t1_storage.read().unwrap_unchecked() };
        let t2_storage_guard = unsafe { t2_storage.read().unwrap_unchecked() };
        let t3_storage_guard = unsafe { t3_storage.read().unwrap_unchecked() };

        for (entity, t1_cell) in &t1_storage_guard.compounds {
            if let Some(t2_cell) = t2_storage_guard.compounds.get(entity) {
                if let Some(t3_cell) = t3_storage_guard.compounds.get(entity) {
                    let t1_data = t1_cell.get();
                    let t2_data = t2_cell.get();
                    let t3_data = t3_cell.get();

                    f(*entity, &*t1_data, &*t2_data, &*t3_data);
                }
            }
        }
    }

    pub fn for_each_molecule_mut<T, F>(&self, mut f: F)
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

    pub fn for_each_duo_mut<T1, T2, F>(&self, mut f: F)
    where
        T1: Send + Sync + 'static,
        T2: Send + Sync + 'static,
        F: FnMut(Entity, &mut T1, &mut T2) + Send + Sync,
    {
        let t1_storage = self.get_or_create_storage::<T1>();
        let t2_storage = self.get_or_create_storage::<T2>();

        let t1_storage_guard = unsafe { t1_storage.read().unwrap_unchecked() };
        let t2_storage_guard = unsafe { t2_storage.read().unwrap_unchecked() };

        for (entity, t1_cell) in &t1_storage_guard.compounds {
            if let Some(t2_cell) = t2_storage_guard.compounds.get(entity) {
                let mut t1_data = t1_cell.get_mut();
                let mut t2_data = t2_cell.get_mut();

                f(*entity, &mut *t1_data, &mut *t2_data);
            }
        }
    }

    pub fn for_each_trio_mut<T1, T2, T3, F>(&self, mut f: F)
    where
        T1: Send + Sync + 'static,
        T2: Send + Sync + 'static,
        T3: Send + Sync + 'static,
        F: FnMut(Entity, &mut T1, &mut T2, &mut T3) + Send + Sync,
    {
        let t1_storage = self.get_or_create_storage::<T1>();
        let t2_storage = self.get_or_create_storage::<T2>();
        let t3_storage = self.get_or_create_storage::<T3>();

        let t1_storage_guard = unsafe { t1_storage.read().unwrap_unchecked() };
        let t2_storage_guard = unsafe { t2_storage.read().unwrap_unchecked() };
        let t3_storage_guard = unsafe { t3_storage.read().unwrap_unchecked() };

        for (entity, t1_cell) in &t1_storage_guard.compounds {
            if let Some(t2_cell) = t2_storage_guard.compounds.get(entity) {
                if let Some(t3_cell) = t3_storage_guard.compounds.get(entity) {
                    let mut t1_data = t1_cell.get_mut();
                    let mut t2_data = t2_cell.get_mut();
                    let mut t3_data = t3_cell.get_mut();

                    f(*entity, &mut *t1_data, &mut *t2_data, &mut *t3_data);
                }
            }
        }
    }
}

#[cfg(test)]
mod ecs_test {
    use std::thread;

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
        let snake = compound.create_entity();

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

        compound.add_molecule(
            snake,
            Label {
                name: "Snivvy".to_string(),
                id: 2,
            },
        );

        compound.add_molecule(
            snake,
            Collar {
                name: "Snivvy".to_string(),
                address: "2 main St.".to_string(),
            },
        );

        compound.for_each_molecule(|_entity, label: &Label| {
            println!("Name: {}", label.name);
            println!("Id: {}", label.id);
        });

        compound.for_each_duo(|_entity, label: &Label, collar: &Collar| {
            println!(
                "Name: {} Id: {} other name: {} address: {}",
                label.name, label.id, collar.name, collar.address
            );
        });
    }

    #[test]
    fn test_ecs_async() {
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

        let compound = Arc::new(RwLock::new(Compound::new()));
        let running = Arc::new(RwLock::new(false));

        let cat = compound.write().unwrap().create_entity();
        let dog = compound.write().unwrap().create_entity();

        compound.read().unwrap().add_molecule(
            cat,
            Label {
                name: "John".to_string(),
                id: 0,
            },
        );

        compound.read().unwrap().add_molecule(
            cat,
            Whiskers {
                color: "Black".to_string(),
                number: 8,
            },
        );

        compound.read().unwrap().add_molecule(
            dog,
            Label {
                name: "Sparky".to_string(),
                id: 1,
            },
        );

        compound.read().unwrap().add_molecule(
            dog,
            Collar {
                name: "Sparky".to_string(),
                address: "1 main St.".to_string(),
            },
        );

        let compound_other_thread = compound.clone();
        let running_clone = running.clone();

        let other_thread = thread::spawn(move || {
            let snake = compound_other_thread.write().unwrap().create_entity();

            compound_other_thread.read().unwrap().add_molecule(
                snake,
                Label {
                    name: "Snivvy".to_string(),
                    id: 2,
                },
            );

            compound_other_thread.read().unwrap().add_molecule(
                snake,
                Collar {
                    name: "Snivvy".to_string(),
                    address: "2 main St.".to_string(),
                },
            );

            *running_clone.write().unwrap() = true;

            for _ in 0..1000 {
                compound_other_thread.read().unwrap().for_each_molecule_mut(
                    |_entity, label: &mut Label| {
                        label.id += 1;
                        println!("Label From other thread: {} {}", label.name, label.id);
                    },
                );
            }
        });

        while !*running.read().unwrap() {}

        for _ in 0..1000 {
            compound.read().unwrap().for_each_duo_mut(
                |_entity, label: &mut Label, collar: &mut Collar| {
                    if label.id > 101 {
                        label.id = 101;
                    }

                    println!(
                        "Collar: {} {}, Id: {}",
                        collar.name, collar.address, label.id
                    );
                },
            );
        }

        _ = other_thread.join();
    }
}
