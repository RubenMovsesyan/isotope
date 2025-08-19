use std::{
    any::{Any, TypeId},
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::{
        Arc, RwLock,
        atomic::{AtomicU64, Ordering},
    },
};

use log::warn;

type EntityId = AtomicU64;
pub type Entity = u64;

// Component Wrapper that allows safe concurrent access
pub struct MoleculeCell<T: Send + Sync + 'static> {
    data: RwLock<T>,
}

impl<T: Send + Sync + 'static> MoleculeCell<T> {
    fn new(data: T) -> Self {
        MoleculeCell {
            data: RwLock::new(data),
        }
    }

    fn read(&self) -> impl Deref<Target = T> + '_ {
        match self.data.read() {
            Ok(data) => data,
            Err(poisoned) => {
                warn!("Molecule Cell Poisoned... Recovering");
                poisoned.into_inner()
            }
        }
    }

    fn write(&self) -> impl DerefMut<Target = T> + '_ {
        match self.data.write() {
            Ok(data) => data,
            Err(poisoned) => {
                warn!("Molecule Cell Poisoned... Recovering");
                poisoned.into_inner()
            }
        }
    }
}

// Storage for a single componenet type
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

#[derive(Debug, Default)]
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

    pub fn create_entity(&self) -> Entity {
        let entity_id = self.num_entities.fetch_add(1, Ordering::Relaxed);
        entity_id
    }

    // Only should be accessed inside this struct
    fn get_or_create_storage<T: Send + Sync + 'static>(&self) -> Arc<RwLock<MoleculeStorage<T>>> {
        let mut storages = match self.storages.write() {
            Ok(storages) => storages,
            Err(poisoned) => {
                warn!("Compound Storages Poisoned... Recovering");
                poisoned.into_inner()
            }
        };

        let type_id = TypeId::of::<T>();

        if !storages.contains_key(&type_id) {
            let storage = MoleculeStorage::<T>::new();
            storages.insert(type_id, Box::new(Arc::new(RwLock::new(storage))));
        }

        // Safety: We just inserted the storage, so it must exist
        let storage = unsafe {
            storages
                .get(&type_id)
                .unwrap_unchecked()
                .downcast_ref::<Arc<RwLock<MoleculeStorage<T>>>>()
                .unwrap_unchecked()
        };

        storage.clone()
    }

    unsafe fn get_storage<T: Send + Sync + 'static>(&self) -> Arc<RwLock<MoleculeStorage<T>>> {
        let storages = match self.storages.read() {
            Ok(storages) => storages,
            Err(poisoned) => {
                warn!("Compound Storages Poisoned... Recovering");
                poisoned.into_inner()
            }
        };

        let type_id = TypeId::of::<T>();

        let storage = unsafe {
            storages
                .get(&type_id)
                .unwrap_unchecked()
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

    pub fn build_compound(&self, entity: Entity, bundle: impl MoleculeBundle) {
        bundle.add_to_entity(self, entity);
    }

    pub fn spawn(&self, bundle: impl MoleculeBundle) -> Entity {
        let entity = self.create_entity();

        self.build_compound(entity, bundle);

        entity
    }

    // Singular Molecule accessors ====================================
    pub fn iter_mol<T, F>(&self, mut f: F)
    where
        T: Send + Sync + 'static,
        F: FnMut(Entity, &T) + Send + Sync,
    {
        let storage = self.get_or_create_storage::<T>();
        let storage_guard = match storage.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Storage poisoned, recovering...");
                poisoned.into_inner()
            }
        };

        for (entity, cell) in &storage_guard.compounds {
            let data = cell.read();
            f(*entity, &*data);
        }
    }

    pub fn iter_without_mol<W, T, F>(&self, mut f: F)
    where
        T: Send + Sync + 'static,
        W: Send + Sync + 'static,
        F: FnMut(Entity, &T) + Send + Sync,
    {
        let t_storage = self.get_or_create_storage::<T>();
        let t_storage_guard = match t_storage.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Storage poisoned, recovering...");
                poisoned.into_inner()
            }
        };

        let w_storage = self.get_or_create_storage::<W>();
        let w_storage_guard = match w_storage.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Storage poisoned, recovering...");
                poisoned.into_inner()
            }
        };

        for (entity, t_cell) in &t_storage_guard.compounds {
            if let Some(_) = w_storage_guard.compounds.get(entity) {
            } else {
                let t_data = t_cell.read();
                f(*entity, &*t_data);
            }
        }
    }

    pub fn iter_mut_mol<T, F>(&self, mut f: F)
    where
        T: Send + Sync + 'static,
        F: FnMut(Entity, &mut T) + Send + Sync,
    {
        let storage = self.get_or_create_storage::<T>();
        let storage_guard = match storage.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Storage poisoned, recovering...");
                poisoned.into_inner()
            }
        };

        for (entity, cell) in &storage_guard.compounds {
            let mut data = cell.write();
            f(*entity, &mut *data);
        }
    }

    pub fn iter_mut_without_mol<W, T, F>(&self, mut f: F)
    where
        T: Send + Sync + 'static,
        W: Send + Sync + 'static,
        F: FnMut(Entity, &mut T) + Send + Sync,
    {
        let t_storage = self.get_or_create_storage::<T>();
        let t_storage_guard = match t_storage.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Storage poisoned, recovering...");
                poisoned.into_inner()
            }
        };

        let w_storage = self.get_or_create_storage::<W>();
        let w_storage_guard = match w_storage.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Storage poisoned, recovering...");
                poisoned.into_inner()
            }
        };

        for (entity, t_cell) in &t_storage_guard.compounds {
            if let Some(_) = w_storage_guard.compounds.get(entity) {
            } else {
                let mut t_data = t_cell.write();
                f(*entity, &mut *t_data);
            }
        }
    }

    // Dual Molecule accessors ==============================
    pub fn iter_duo<T1, T2, F>(&self, mut f: F)
    where
        T1: Send + Sync + 'static,
        T2: Send + Sync + 'static,
        F: FnMut(Entity, &T1, &T2),
    {
        let t1_storage = self.get_or_create_storage::<T1>();
        let t2_storage = self.get_or_create_storage::<T2>();

        let t1_storage_guard = match t1_storage.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Storage poisoned, recovering...");
                poisoned.into_inner()
            }
        };

        let t2_storage_guard = match t2_storage.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Storage poisoned, recovering...");
                poisoned.into_inner()
            }
        };

        for (entity, t1_cell) in &t1_storage_guard.compounds {
            if let Some(t2_cell) = t2_storage_guard.compounds.get(entity) {
                let t1_data = t1_cell.read();
                let t2_data = t2_cell.read();
                f(*entity, &*t1_data, &*t2_data);
            }
        }
    }

    pub fn iter_without_duo<W, T1, T2, F>(&self, mut f: F)
    where
        T1: Send + Sync + 'static,
        T2: Send + Sync + 'static,
        W: Send + Sync + 'static,
        F: FnMut(Entity, &T1, &T2),
    {
        let t1_storage = self.get_or_create_storage::<T1>();
        let t2_storage = self.get_or_create_storage::<T2>();

        let t1_storage_guard = match t1_storage.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Storage poisoned, recovering...");
                poisoned.into_inner()
            }
        };

        let t2_storage_guard = match t2_storage.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Storage poisoned, recovering...");
                poisoned.into_inner()
            }
        };

        let w_storage = self.get_or_create_storage::<W>();
        let w_storage_guard = match w_storage.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Storage poisoned, recovering...");
                poisoned.into_inner()
            }
        };

        for (entity, t1_cell) in &t1_storage_guard.compounds {
            if let Some(t2_cell) = t2_storage_guard.compounds.get(entity) {
                if let Some(_) = w_storage_guard.compounds.get(entity) {
                } else {
                    let t1_data = t1_cell.read();
                    let t2_data = t2_cell.read();

                    f(*entity, &*t1_data, &*t2_data);
                }
            }
        }
    }

    pub fn iter_mut_duo<T1, T2, F>(&self, mut f: F)
    where
        T1: Send + Sync + 'static,
        T2: Send + Sync + 'static,
        F: FnMut(Entity, &mut T1, &mut T2),
    {
        let t1_storage = self.get_or_create_storage::<T1>();
        let t2_storage = self.get_or_create_storage::<T2>();

        let t1_storage_guard = match t1_storage.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Storage poisoned, recovering...");
                poisoned.into_inner()
            }
        };

        let t2_storage_guard = match t2_storage.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Storage poisoned, recovering...");
                poisoned.into_inner()
            }
        };

        for (entity, t1_cell) in &t1_storage_guard.compounds {
            if let Some(t2_cell) = t2_storage_guard.compounds.get(entity) {
                let mut t1_data = t1_cell.write();
                let mut t2_data = t2_cell.write();
                f(*entity, &mut *t1_data, &mut *t2_data);
            }
        }
    }

    pub fn iter_mut_without_duo<W, T1, T2, F>(&self, mut f: F)
    where
        T1: Send + Sync + 'static,
        T2: Send + Sync + 'static,
        W: Send + Sync + 'static,
        F: FnMut(Entity, &mut T1, &mut T2),
    {
        let t1_storage = self.get_or_create_storage::<T1>();
        let t2_storage = self.get_or_create_storage::<T2>();

        let t1_storage_guard = match t1_storage.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Storage poisoned, recovering...");
                poisoned.into_inner()
            }
        };

        let t2_storage_guard = match t2_storage.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Storage poisoned, recovering...");
                poisoned.into_inner()
            }
        };

        let w_storage = self.get_or_create_storage::<W>();
        let w_storage_guard = match w_storage.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Storage poisoned, recovering...");
                poisoned.into_inner()
            }
        };

        for (entity, t1_cell) in &t1_storage_guard.compounds {
            if let Some(t2_cell) = t2_storage_guard.compounds.get(entity) {
                if let Some(_) = w_storage_guard.compounds.get(entity) {
                } else {
                    let mut t1_data = t1_cell.write();
                    let mut t2_data = t2_cell.write();

                    f(*entity, &mut *t1_data, &mut *t2_data);
                }
            }
        }
    }

    // Trio Molecule accessors ================================
    pub fn iter_trio<T1, T2, T3, F>(&self, mut f: F)
    where
        T1: Send + Sync + 'static,
        T2: Send + Sync + 'static,
        T3: Send + Sync + 'static,
        F: FnMut(Entity, &T1, &T2, &T3),
    {
        let t1_storage = self.get_or_create_storage::<T1>();
        let t2_storage = self.get_or_create_storage::<T2>();
        let t3_storage = self.get_or_create_storage::<T3>();

        let t1_storage_guard = match t1_storage.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Storage poisoned, recovering...");
                poisoned.into_inner()
            }
        };
        let t2_storage_guard = match t2_storage.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Storage poisoned, recovering...");
                poisoned.into_inner()
            }
        };
        let t3_storage_guard = match t3_storage.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Storage poisoned, recovering...");
                poisoned.into_inner()
            }
        };

        for (entity, t1_cell) in &t1_storage_guard.compounds {
            if let Some(t2_cell) = t2_storage_guard.compounds.get(entity) {
                if let Some(t3_cell) = t3_storage_guard.compounds.get(entity) {
                    let t1_data = t1_cell.read();
                    let t2_data = t2_cell.read();
                    let t3_data = t3_cell.read();

                    f(*entity, &*t1_data, &*t2_data, &*t3_data);
                }
            }
        }
    }

    pub fn iter_without_trio<W, T1, T2, T3, F>(&self, mut f: F)
    where
        T1: Send + Sync + 'static,
        T2: Send + Sync + 'static,
        T3: Send + Sync + 'static,
        W: Send + Sync + 'static,
        F: FnMut(Entity, &T1, &T2, &T3),
    {
        let t1_storage = self.get_or_create_storage::<T1>();
        let t2_storage = self.get_or_create_storage::<T2>();
        let t3_storage = self.get_or_create_storage::<T3>();

        let t1_storage_guard = match t1_storage.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Storage poisoned, recovering...");
                poisoned.into_inner()
            }
        };
        let t2_storage_guard = match t2_storage.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Storage poisoned, recovering...");
                poisoned.into_inner()
            }
        };
        let t3_storage_guard = match t3_storage.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Storage poisoned, recovering...");
                poisoned.into_inner()
            }
        };

        let w_storage = self.get_or_create_storage::<W>();
        let w_storage_guard = match w_storage.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Storage poisoned, recovering...");
                poisoned.into_inner()
            }
        };

        for (entity, t1_cell) in &t1_storage_guard.compounds {
            if let Some(t2_cell) = t2_storage_guard.compounds.get(entity) {
                if let Some(t3_cell) = t3_storage_guard.compounds.get(entity) {
                    if let Some(_) = w_storage_guard.compounds.get(entity) {
                    } else {
                        let t1_data = t1_cell.read();
                        let t2_data = t2_cell.read();
                        let t3_data = t3_cell.read();

                        f(*entity, &*t1_data, &*t2_data, &*t3_data);
                    }
                }
            }
        }
    }

    pub fn iter_mut_trio<T1, T2, T3, F>(&self, mut f: F)
    where
        T1: Send + Sync + 'static,
        T2: Send + Sync + 'static,
        T3: Send + Sync + 'static,
        F: FnMut(Entity, &mut T1, &mut T2, &mut T3),
    {
        let t1_storage = self.get_or_create_storage::<T1>();
        let t2_storage = self.get_or_create_storage::<T2>();
        let t3_storage = self.get_or_create_storage::<T3>();

        let t1_storage_guard = match t1_storage.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Storage poisoned, recovering...");
                poisoned.into_inner()
            }
        };
        let t2_storage_guard = match t2_storage.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Storage poisoned, recovering...");
                poisoned.into_inner()
            }
        };
        let t3_storage_guard = match t3_storage.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Storage poisoned, recovering...");
                poisoned.into_inner()
            }
        };

        for (entity, t1_cell) in &t1_storage_guard.compounds {
            if let Some(t2_cell) = t2_storage_guard.compounds.get(entity) {
                if let Some(t3_cell) = t3_storage_guard.compounds.get(entity) {
                    let mut t1_data = t1_cell.write();
                    let mut t2_data = t2_cell.write();
                    let mut t3_data = t3_cell.write();

                    f(*entity, &mut *t1_data, &mut *t2_data, &mut *t3_data);
                }
            }
        }
    }

    pub fn iter_mut_without_trio<W, T1, T2, T3, F>(&self, mut f: F)
    where
        T1: Send + Sync + 'static,
        T2: Send + Sync + 'static,
        T3: Send + Sync + 'static,
        W: Send + Sync + 'static,
        F: FnMut(Entity, &mut T1, &mut T2, &mut T3),
    {
        let t1_storage = self.get_or_create_storage::<T1>();
        let t2_storage = self.get_or_create_storage::<T2>();
        let t3_storage = self.get_or_create_storage::<T3>();

        let t1_storage_guard = match t1_storage.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Storage poisoned, recovering...");
                poisoned.into_inner()
            }
        };
        let t2_storage_guard = match t2_storage.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Storage poisoned, recovering...");
                poisoned.into_inner()
            }
        };
        let t3_storage_guard = match t3_storage.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Storage poisoned, recovering...");
                poisoned.into_inner()
            }
        };

        let w_storage = self.get_or_create_storage::<W>();
        let w_storage_guard = match w_storage.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Storage poisoned, recovering...");
                poisoned.into_inner()
            }
        };

        for (entity, t1_cell) in &t1_storage_guard.compounds {
            if let Some(t2_cell) = t2_storage_guard.compounds.get(entity) {
                if let Some(t3_cell) = t3_storage_guard.compounds.get(entity) {
                    if let Some(_) = w_storage_guard.compounds.get(entity) {
                    } else {
                        let mut t1_data = t1_cell.write();
                        let mut t2_data = t2_cell.write();
                        let mut t3_data = t3_cell.write();

                        f(*entity, &mut *t1_data, &mut *t2_data, &mut *t3_data);
                    }
                }
            }
        }
    }
}

// For adding multiple molecules to the same entity
pub trait MoleculeBundle {
    fn add_to_entity(self, compound: &Compound, entity: Entity);
}

// Macro to implement for multiple sizes of tuples (add more if needed)
macro_rules! impl_molecule_bundle_for_tuple {
    ($($T:ident), *) => {
        #[allow(non_snake_case)]
        impl<$($T: Send + Sync + 'static),*> MoleculeBundle for ($($T,)*) {
            fn add_to_entity(self, compound: &Compound, entity: Entity) {
                let ($($T,)*) = self;
                $(
                    compound.add_molecule(entity, $T);
                )*
            }
        }
    };
}

impl_molecule_bundle_for_tuple!(A);
impl_molecule_bundle_for_tuple!(A, B);
impl_molecule_bundle_for_tuple!(A, B, C);
impl_molecule_bundle_for_tuple!(A, B, C, D);
impl_molecule_bundle_for_tuple!(A, B, C, D, E);
impl_molecule_bundle_for_tuple!(A, B, C, D, E, F);
impl_molecule_bundle_for_tuple!(A, B, C, D, E, F, G);
impl_molecule_bundle_for_tuple!(A, B, C, D, E, F, G, H);
impl_molecule_bundle_for_tuple!(A, B, C, D, E, F, G, H, I);
impl_molecule_bundle_for_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_molecule_bundle_for_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_molecule_bundle_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);

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

        let compound = Compound::new();

        _ = compound.spawn((
            Label {
                name: "John".to_string(),
                id: 0,
            },
            Whiskers {
                color: "Black".to_string(),
                number: 8,
            },
        ));

        _ = compound.spawn((
            Label {
                name: "Sparky".to_string(),
                id: 1,
            },
            Collar {
                name: "Sparky".to_string(),
                address: "1 main St.".to_string(),
            },
        ));

        _ = compound.spawn((
            Label {
                name: "Snivvy".to_string(),
                id: 2,
            },
            Collar {
                name: "Snivvy".to_string(),
                address: "2 main St.".to_string(),
            },
        ));

        compound.iter_mol(|_entity, label: &Label| {
            println!("Name: {}", label.name);
            println!("Id: {}", label.id);
        });

        compound.iter_duo(|_entity, label: &Label, collar: &Collar| {
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
                compound_other_thread
                    .read()
                    .unwrap()
                    .iter_mut_mol(|_entity, label: &mut Label| {
                        label.id += 1;
                        println!("Label From other thread: {} {}", label.name, label.id);
                    });
            }
        });

        while !*running.read().unwrap() {}

        for _ in 0..1000 {
            compound.read().unwrap().iter_mut_duo(
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
