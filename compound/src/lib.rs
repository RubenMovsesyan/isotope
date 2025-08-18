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
}

// For adding multiple molecules to the same entity
pub trait MoleculeBundle {
    fn add_to_entity(self, compound: &Compound, entity: Entity);
}

// impl<T: Send + Sync + 'static> MoleculeBundle for T {
//     fn add_to_entity(self, compound: &Compound, entity: Entity) {
//         compound.add_molecule(entity, self);
//     }
// }

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
