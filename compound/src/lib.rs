//! # Compound - A Thread-Safe Entity Component System (ECS)
//!
//! This module provides a concurrent Entity Component System implementation with a chemistry-inspired
//! naming convention. Entities are composed of "molecules" (components) that can be safely accessed
//! and modified from multiple threads simultaneously.
//!
//! ## Key Concepts
//! - **Entity**: A unique identifier for a game object or data container
//! - **Molecule**: A component that can be attached to an entity (analogous to components in traditional ECS)
//! - **Compound**: The main ECS world that manages all entities and their molecules
//! - **MoleculeBundle**: A collection of molecules that can be added to an entity together
//!
//! ## Thread Safety
//! All operations in this ECS are thread-safe through the use of `RwLock`s and atomic operations.
//! Multiple threads can read/write different component types simultaneously without blocking each other.

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

/// Internal type alias for the atomic counter used to generate entity IDs.
/// This ensures thread-safe entity ID generation without locks.
type EntityId = AtomicU64;

/// A unique identifier for an entity in the ECS.
///
/// Entities are represented as simple u64 values that uniquely identify
/// a collection of molecules (components) in the compound (world).
/// Entity IDs are generated sequentially and are guaranteed to be unique
/// within a single Compound instance.
pub type Entity = u64;

/// A thread-safe wrapper around a component that allows concurrent read/write access.
///
/// `MoleculeCell` uses a `RwLock` internally to provide safe concurrent access to
/// component data. It handles poisoned locks gracefully by recovering the inner value
/// and logging a warning.
///
/// # Type Parameters
/// - `T`: The component type, must be `Send + Sync + 'static`
///
/// # Thread Safety
/// Multiple readers can access the data simultaneously, while writers get exclusive access.
/// Lock poisoning is handled automatically with recovery and logging.
pub struct MoleculeCell<T: Send + Sync + 'static> {
    data: RwLock<T>,
}

impl<T: Send + Sync + 'static> MoleculeCell<T> {
    /// Creates a new `MoleculeCell` containing the given data.
    ///
    /// # Arguments
    /// - `data`: The component data to wrap
    ///
    /// # Returns
    /// A new `MoleculeCell` instance with the data protected by a `RwLock`
    ///
    /// # Example
    /// ```ignore
    /// let cell = MoleculeCell::new(MyComponent { value: 42 });
    /// ```
    fn new(data: T) -> Self {
        MoleculeCell {
            data: RwLock::new(data),
        }
    }

    /// Acquires a read lock on the component data.
    ///
    /// This method allows multiple concurrent readers but blocks writers.
    /// If the lock is poisoned (a writer panicked while holding the lock),
    /// it recovers the data and logs a warning.
    ///
    /// # Returns
    /// A RAII guard that dereferences to `&T`
    ///
    /// # Panics
    /// This method should not panic under normal circumstances as it handles
    /// poisoned locks gracefully.
    fn read(&self) -> impl Deref<Target = T> + '_ {
        match self.data.read() {
            Ok(data) => data,
            Err(poisoned) => {
                warn!("Molecule Cell Poisoned... Recovering");
                poisoned.into_inner()
            }
        }
    }

    /// Acquires a write lock on the component data.
    ///
    /// This method provides exclusive access to the component data, blocking
    /// all other readers and writers. If the lock is poisoned, it recovers
    /// the data and logs a warning.
    ///
    /// # Returns
    /// A RAII guard that dereferences to `&mut T`
    ///
    /// # Panics
    /// This method should not panic under normal circumstances as it handles
    /// poisoned locks gracefully.
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

/// Storage container for all components of a specific type.
///
/// `MoleculeStorage` maintains a mapping from entity IDs to their corresponding
/// component data wrapped in `MoleculeCell`s. This allows efficient lookup and
/// iteration over all entities that have a specific component type.
///
/// # Type Parameters
/// - `T`: The component type stored in this container
///
/// # Internal Structure
/// Uses a `HashMap` for O(1) average-case lookup by entity ID.
pub struct MoleculeStorage<T: Send + Sync + 'static> {
    compounds: HashMap<Entity, MoleculeCell<T>>,
}

impl<T: Send + Sync + 'static> MoleculeStorage<T> {
    /// Creates a new, empty `MoleculeStorage` instance.
    ///
    /// # Returns
    /// An empty storage container ready to store components of type `T`
    ///
    /// # Example
    /// ```ignore
    /// let storage: MoleculeStorage<Position> = MoleculeStorage::new();
    /// ```
    fn new() -> Self {
        Self {
            compounds: HashMap::new(),
        }
    }
}

/// The main ECS world that manages all entities and their components.
///
/// `Compound` is the central data structure of the ECS. It maintains:
/// - A counter for generating unique entity IDs
/// - Type-erased storage for all component types
///
/// All operations on `Compound` are thread-safe, allowing concurrent access
/// from multiple systems or threads.
///
/// # Example
/// ```ignore
/// let compound = Compound::new();
///
/// // Create an entity with components
/// let entity = compound.spawn((
///     Position { x: 0.0, y: 0.0 },
///     Velocity { x: 1.0, y: 0.0 }
/// ));
///
/// // Iterate over entities with specific components
/// compound.iter_duo::<Position, Velocity, _>(|entity, pos, vel| {
///     println!("Entity {} at ({}, {})", entity, pos.x, pos.y);
/// });
/// ```
#[derive(Debug, Default)]
pub struct Compound {
    /// Atomic counter for generating unique entity IDs
    num_entities: EntityId,
    /// Type-erased storage for all component types, indexed by TypeId
    storages: RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
}

impl Compound {
    /// Creates a new, empty `Compound` instance.
    ///
    /// # Returns
    /// A new ECS world with no entities or components
    ///
    /// # Example
    /// ```ignore
    /// let compound = Compound::new();
    /// assert_eq!(compound.num_entities.load(Ordering::Relaxed), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            num_entities: AtomicU64::new(0),
            storages: RwLock::new(HashMap::new()),
        }
    }

    /// Creates a new entity and returns its unique identifier.
    ///
    /// This method atomically increments the internal entity counter to ensure
    /// each entity gets a unique ID even when called from multiple threads.
    ///
    /// # Returns
    /// A unique `Entity` ID that can be used to add components
    ///
    /// # Thread Safety
    /// This method is thread-safe and can be called concurrently
    ///
    /// # Example
    /// ```ignore
    /// let entity = compound.create_entity();
    /// compound.add_molecule(entity, Position { x: 0.0, y: 0.0 });
    /// ```
    pub fn create_entity(&self) -> Entity {
        let entity_id = self.num_entities.fetch_add(1, Ordering::Relaxed);
        entity_id
    }

    /// Gets or creates the storage for a specific component type.
    ///
    /// This internal method ensures that storage exists for a component type,
    /// creating it if necessary. It handles lock poisoning gracefully.
    ///
    /// # Type Parameters
    /// - `T`: The component type to get storage for
    ///
    /// # Returns
    /// An `Arc<RwLock<MoleculeStorage<T>>>` that can be shared across threads
    ///
    /// # Safety
    /// This method uses unsafe code for performance but is safe because:
    /// - It ensures the storage exists before accessing it
    /// - Type safety is maintained through TypeId
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

    /// Gets the storage for a specific component type without creating it.
    ///
    /// # Type Parameters
    /// - `T`: The component type to get storage for
    ///
    /// # Returns
    /// An `Arc<RwLock<MoleculeStorage<T>>>` for the requested component type
    ///
    /// # Safety
    /// This method is unsafe because it assumes the storage exists. Calling this
    /// without ensuring the storage exists will cause undefined behavior.
    /// Use `get_or_create_storage` for safe access.
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

    /// Adds a component (molecule) to an entity.
    ///
    /// If the entity already has a component of this type, it will be replaced.
    ///
    /// # Arguments
    /// - `entity`: The entity to add the component to
    /// - `molecule`: The component data to add
    ///
    /// # Type Parameters
    /// - `T`: The type of component to add
    ///
    /// # Thread Safety
    /// This method is thread-safe and can be called concurrently for different
    /// component types or different entities.
    ///
    /// # Example
    /// ```ignore
    /// let entity = compound.create_entity();
    /// compound.add_molecule(entity, Health { current: 100, max: 100 });
    /// compound.add_molecule(entity, Name("Player".to_string()));
    /// ```
    pub fn add_molecule<T: Send + Sync + 'static>(&self, entity: Entity, molecule: T) {
        unsafe {
            self.get_or_create_storage::<T>()
                .write()
                .unwrap_unchecked()
                .compounds
                .insert(entity, MoleculeCell::new(molecule));
        };
    }

    /// Adds multiple components to an entity using a bundle.
    ///
    /// This is a convenience method that delegates to the bundle's
    /// `add_to_entity` implementation.
    ///
    /// # Arguments
    /// - `entity`: The entity to add components to
    /// - `bundle`: A bundle of components (typically a tuple)
    ///
    /// # Example
    /// ```ignore
    /// let entity = compound.create_entity();
    /// compound.build_compound(entity, (
    ///     Position { x: 10.0, y: 20.0 },
    ///     Velocity { x: 1.0, y: 0.0 },
    ///     Health { current: 100, max: 100 }
    /// ));
    /// ```
    pub fn build_compound(&self, entity: Entity, bundle: impl MoleculeBundle) {
        bundle.add_to_entity(self, entity);
    }

    /// Creates a new entity and adds components to it in one operation.
    ///
    /// This is the most common way to create entities with their initial components.
    ///
    /// # Arguments
    /// - `bundle`: A bundle of components to add to the new entity
    ///
    /// # Returns
    /// The newly created entity's ID
    ///
    /// # Example
    /// ```ignore
    /// let player = compound.spawn((
    ///     Position { x: 0.0, y: 0.0 },
    ///     Velocity { x: 0.0, y: 0.0 },
    ///     Health { current: 100, max: 100 },
    ///     Name("Player".to_string())
    /// ));
    /// ```
    pub fn spawn(&self, bundle: impl MoleculeBundle) -> Entity {
        let entity = self.create_entity();

        self.build_compound(entity, bundle);

        entity
    }

    // Singular Molecule accessors ====================================

    /// Iterates over all entities that have a specific component type.
    ///
    /// Provides read-only access to the component data. Multiple threads can
    /// iterate over the same component type simultaneously.
    ///
    /// # Type Parameters
    /// - `T`: The component type to iterate over
    /// - `F`: The closure type
    ///
    /// # Arguments
    /// - `f`: A closure that receives the entity ID and a reference to the component
    ///
    /// # Thread Safety
    /// Multiple threads can call this method simultaneously for the same or different
    /// component types.
    ///
    /// # Example
    /// ```ignore
    /// compound.iter_mol::<Position, _>(|entity, pos| {
    ///     println!("Entity {} is at ({}, {})", entity, pos.x, pos.y);
    /// });
    /// ```
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

    /// Iterates over entities with component T but without component W.
    ///
    /// This is useful for filtering entities based on the absence of a component.
    /// For example, finding all entities with Position but without Velocity.
    ///
    /// # Type Parameters
    /// - `W`: The component type that entities should NOT have
    /// - `T`: The component type to iterate over
    /// - `F`: The closure type
    ///
    /// # Arguments
    /// - `f`: A closure that receives the entity ID and a reference to component T
    ///
    /// # Example
    /// ```ignore
    /// // Find all static entities (have Position but not Velocity)
    /// compound.iter_without_mol::<Velocity, Position, _>(|entity, pos| {
    ///     println!("Static entity {} at ({}, {})", entity, pos.x, pos.y);
    /// });
    /// ```
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

    /// Iterates over all entities with a component, providing mutable access.
    ///
    /// This method provides exclusive write access to each component. While
    /// iterating, other threads can still read/write different component types.
    ///
    /// # Type Parameters
    /// - `T`: The component type to iterate over
    /// - `F`: The closure type
    ///
    /// # Arguments
    /// - `f`: A closure that receives the entity ID and a mutable reference to the component
    ///
    /// # Example
    /// ```ignore
    /// compound.iter_mut_mol::<Position, _>(|entity, pos| {
    ///     pos.x += 1.0; // Move all entities to the right
    ///     pos.y += 1.0;
    /// });
    /// ```
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

    /// Iterates over entities with component T but without W, with mutable access to T.
    ///
    /// Provides mutable access to component T for entities that don't have component W.
    ///
    /// # Type Parameters
    /// - `W`: The component type that entities should NOT have
    /// - `T`: The component type to iterate over with mutable access
    /// - `F`: The closure type
    ///
    /// # Arguments
    /// - `f`: A closure that receives the entity ID and a mutable reference to component T
    ///
    /// # Example
    /// ```ignore
    /// // Apply gravity only to entities without Flying component
    /// compound.iter_mut_without_mol::<Flying, Velocity, _>(|entity, vel| {
    ///     vel.y -= 9.81 * delta_time;
    /// });
    /// ```
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

    /// Iterates over entities that have both of two component types.
    ///
    /// Provides read-only access to both components. Only entities that have
    /// both component types will be included in the iteration.
    ///
    /// # Type Parameters
    /// - `T1`: The first component type
    /// - `T2`: The second component type
    /// - `F`: The closure type
    ///
    /// # Arguments
    /// - `f`: A closure that receives the entity ID and references to both components
    ///
    /// # Example
    /// ```ignore
    /// compound.iter_duo::<Position, Velocity, _>(|entity, pos, vel| {
    ///     println!("Entity {} at ({}, {}) moving at ({}, {})",
    ///              entity, pos.x, pos.y, vel.x, vel.y);
    /// });
    /// ```
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

    /// Iterates over entities with components T1 and T2 but without component W.
    ///
    /// Useful for filtering entity iterations based on the absence of a component.
    ///
    /// # Type Parameters
    /// - `W`: The component type that entities should NOT have
    /// - `T1`: The first required component type
    /// - `T2`: The second required component type
    /// - `F`: The closure type
    ///
    /// # Arguments
    /// - `f`: A closure that receives the entity ID and references to T1 and T2
    ///
    /// # Example
    /// ```ignore
    /// // Process all moving entities that aren't frozen
    /// compound.iter_without_duo::<Frozen, Position, Velocity, _>(
    ///     |entity, pos, vel| {
    ///         // Update position based on velocity
    ///         println!("Moving entity {} from ({}, {})", entity, pos.x, pos.y);
    ///     }
    /// );
    /// ```
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

    /// Iterates over entities with two components, providing mutable access to both.
    ///
    /// This method provides write access to both components simultaneously.
    ///
    /// # Type Parameters
    /// - `T1`: The first component type
    /// - `T2`: The second component type
    /// - `F`: The closure type
    ///
    /// # Arguments
    /// - `f`: A closure that receives the entity ID and mutable references to both components
    ///
    /// # Example
    /// ```ignore
    /// // Update position based on velocity
    /// compound.iter_mut_duo::<Position, Velocity, _>(|entity, pos, vel| {
    ///     pos.x += vel.x * delta_time;
    ///     pos.y += vel.y * delta_time;
    /// });
    /// ```
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

    /// Iterates over entities with T1 and T2 but without W, with mutable access.
    ///
    /// Provides mutable access to components T1 and T2 for entities that don't have W.
    ///
    /// # Type Parameters
    /// - `W`: The component type that entities should NOT have
    /// - `T1`: The first component type (mutable access)
    /// - `T2`: The second component type (mutable access)
    /// - `F`: The closure type
    ///
    /// # Arguments
    /// - `f`: A closure that receives the entity ID and mutable references to T1 and T2
    ///
    /// # Example
    /// ```ignore
    /// // Apply physics to non-static entities
    /// compound.iter_mut_without_duo::<Static, Position, Velocity, _>(
    ///     |entity, pos, vel| {
    ///         pos.x += vel.x * dt;
    ///         pos.y += vel.y * dt;
    ///         vel.y -= gravity * dt;
    ///     }
    /// );
    /// ```
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
    /// Iterates over all entities that have all three specified component types.
    ///
    /// Provides read-only access to all three components. Only entities that have
    /// all three component types will be included in the iteration. This is useful
    /// for systems that need to process entities with multiple related components.
    ///
    /// # Type Parameters
    /// - `T1`: The first component type
    /// - `T2`: The second component type
    /// - `T3`: The third component type
    /// - `F`: The closure type
    ///
    /// # Arguments
    /// - `f`: A closure that receives the entity ID and references to all three components
    ///
    /// # Thread Safety
    /// Multiple threads can call this method simultaneously for the same or different
    /// component types, as it only acquires read locks on the component storages.
    ///
    /// # Example
    /// ```ignore
    /// // Process entities with position, velocity, and acceleration
    /// compound.iter_trio::<Position, Velocity, Acceleration, _>(
    ///     |entity, pos, vel, acc| {
    ///         println!("Entity {} at ({}, {}) moving at ({}, {}) accelerating at ({}, {})",
    ///                  entity, pos.x, pos.y, vel.x, vel.y, acc.x, acc.y);
    ///     }
    /// );
    /// ```
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

    /// Iterates over entities with components T1, T2, and T3 but without component W.
    ///
    /// Provides read-only access to the three components for entities that have all three
    /// required components but do not have the excluded component W. This is useful for
    /// filtering entity iterations based on the absence of a specific component while
    /// requiring multiple other components.
    ///
    /// # Type Parameters
    /// - `W`: The component type that entities should NOT have
    /// - `T1`: The first required component type
    /// - `T2`: The second required component type
    /// - `T3`: The third required component type
    /// - `F`: The closure type
    ///
    /// # Arguments
    /// - `f`: A closure that receives the entity ID and references to T1, T2, and T3
    ///
    /// # Thread Safety
    /// Multiple threads can call this method simultaneously for the same or different
    /// component types, as it only acquires read locks on the component storages.
    ///
    /// # Example
    /// ```ignore
    /// // Process all entities with physics components but not marked as Static
    /// compound.iter_without_trio::<Static, Position, Velocity, Acceleration, _>(
    ///     |entity, pos, vel, acc| {
    ///         println!("Dynamic entity {} at ({}, {}) with velocity ({}, {}) and acceleration ({}, {})",
    ///                  entity, pos.x, pos.y, vel.x, vel.y, acc.x, acc.y);
    ///     }
    /// );
    /// ```
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

    /// Iterates over entities with three components, providing mutable access to all three.
    ///
    /// This method provides exclusive write access to all three components simultaneously.
    /// Only entities that have all three component types will be included in the iteration.
    /// This is useful for systems that need to modify multiple related components together,
    /// such as physics systems that update position, velocity, and acceleration.
    ///
    /// # Type Parameters
    /// - `T1`: The first component type
    /// - `T2`: The second component type
    /// - `T3`: The third component type
    /// - `F`: The closure type
    ///
    /// # Arguments
    /// - `f`: A closure that receives the entity ID and mutable references to all three components
    ///
    /// # Thread Safety
    /// While this method is thread-safe, it acquires write locks on individual components
    /// as it iterates. Other threads can still read/write different component types or
    /// different entities' components of the same types.
    ///
    /// # Example
    /// ```ignore
    /// // Update physics for entities with position, velocity, and acceleration
    /// compound.iter_mut_trio::<Position, Velocity, Acceleration, _>(
    ///     |entity, pos, vel, acc| {
    ///         // Update velocity based on acceleration
    ///         vel.x += acc.x * delta_time;
    ///         vel.y += acc.y * delta_time;
    ///         
    ///         // Update position based on velocity
    ///         pos.x += vel.x * delta_time;
    ///         pos.y += vel.y * delta_time;
    ///         
    ///         // Apply damping to acceleration
    ///         acc.x *= 0.99;
    ///         acc.y *= 0.99;
    ///     }
    /// );
    /// ``` 
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

    /// Iterates over entities with T1, T2, and T3 but without W, with mutable access.
    ///
    /// Provides mutable access to components T1, T2, and T3 for entities that have all three
    /// required components but do not have the excluded component W. This is useful for
    /// systems that need to modify multiple related components together while filtering out
    /// entities with a specific marker or state component.
    ///
    /// # Type Parameters
    /// - `W`: The component type that entities should NOT have
    /// - `T1`: The first component type (mutable access)
    /// - `T2`: The second component type (mutable access)
    /// - `T3`: The third component type (mutable access)
    /// - `F`: The closure type
    ///
    /// # Arguments
    /// - `f`: A closure that receives the entity ID and mutable references to T1, T2, and T3
    ///
    /// # Thread Safety
    /// While this method is thread-safe, it acquires write locks on individual components
    /// as it iterates. Other threads can still read/write different component types or
    /// different entities' components of the same types.
    ///
    /// # Example
    /// ```ignore
    /// // Update physics for entities with position, velocity, and acceleration
    /// // but skip entities marked as Frozen
    /// compound.iter_mut_without_trio::<Frozen, Position, Velocity, Acceleration, _>(
    ///     |entity, pos, vel, acc| {
    ///         // Update velocity based on acceleration
    ///         vel.x += acc.x * delta_time;
    ///         vel.y += acc.y * delta_time;
    ///         
    ///         // Update position based on velocity
    ///         pos.x += vel.x * delta_time;
    ///         pos.y += vel.y * delta_time;
    ///         
    ///         // Apply gravity
    ///         acc.y -= 9.81;
    ///     }
    /// );
    /// ``` 
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
/// A trait for bundling multiple components (molecules) together for convenient entity creation.
///
/// `MoleculeBundle` allows you to group multiple components that are commonly added together
/// to entities. This trait is automatically implemented for tuples of up to 12 components,
/// making it easy to spawn entities with multiple components in a single operation.
///
/// # Purpose
/// 
/// Instead of calling `add_molecule` multiple times for each component, you can pass a tuple
/// of components to methods like `spawn` or `build_compound`, which will automatically add
/// all components to the entity.
///
/// # Implementation
/// 
/// This trait is implemented for tuples of varying sizes (1 to 12 components) through a macro.
/// Each component in the tuple must be `Send + Sync + 'static` to ensure thread safety.
///
/// # Thread Safety
/// 
/// All components in a bundle must be thread-safe (`Send + Sync`) as they may be accessed
/// from multiple threads concurrently through the ECS's internal locking mechanisms.
///
/// # Examples
///
/// ## Using a bundle to spawn an entity
/// ```ignore
/// // Create an entity with multiple components at once
/// let player = compound.spawn((
///     Position { x: 0.0, y: 0.0 },
///     Velocity { x: 0.0, y: 0.0 },
///     Health { current: 100, max: 100 },
///     Name("Player".to_string())
/// ));
/// ```
///
/// ## Adding a bundle to an existing entity
/// ```ignore
/// let entity = compound.create_entity();
/// 
/// // Add multiple components as a bundle
/// compound.build_compound(entity, (
///     Position { x: 10.0, y: 20.0 },
///     Velocity { x: 1.0, y: 0.0 }
/// ));
/// ```
///
/// ## Custom bundle types
/// While this trait is automatically implemented for tuples, you could also implement it
/// for custom types if you want named bundles:
///
/// ```ignore
/// struct PlayerBundle {
///     position: Position,
///     velocity: Velocity,
///     health: Health,
///     name: Name,
/// }
///
/// impl MoleculeBundle for PlayerBundle {
///     fn add_to_entity(self, compound: &Compound, entity: Entity) {
///         compound.add_molecule(entity, self.position);
///         compound.add_molecule(entity, self.velocity);
///         compound.add_molecule(entity, self.health);
///         compound.add_molecule(entity, self.name);
///     }
/// }
/// ```
pub trait MoleculeBundle {
    fn add_to_entity(self, compound: &Compound, entity: Entity);
}

// Macro to implement for multiple sizes of tuples (add more if needed)
/// Macro to generate implementations of `MoleculeBundle` for tuples of various sizes.
///
/// This macro automatically implements the `MoleculeBundle` trait for tuples containing
/// 1 to 12 components. It enables convenient entity creation with multiple components
/// by allowing tuples to be passed directly to methods like `spawn` and `build_compound`.
///
/// # How It Works
///
/// The macro takes a list of generic type parameters and generates an implementation
/// that destructures the tuple and calls `add_molecule` for each component in sequence.
/// Each type parameter must satisfy the bounds `Send + Sync + 'static` to ensure
/// thread-safe access within the ECS.
///
/// # Generated Code
///
/// For a tuple `(A, B, C)`, the macro generates:
/// ```ignore
/// impl<A: Send + Sync + 'static, B: Send + Sync + 'static, C: Send + Sync + 'static> 
///     MoleculeBundle for (A, B, C) {
///     fn add_to_entity(self, compound: &Compound, entity: Entity) {
///         let (A, B, C) = self;
///         compound.add_molecule(entity, A);
///         compound.add_molecule(entity, B);
///         compound.add_molecule(entity, C);
///     }
/// }
/// ```
///
/// # Usage
///
/// This macro is invoked multiple times below to implement `MoleculeBundle` for
/// tuples of sizes 1 through 12. The implementations allow ergonomic entity creation:
///
/// ```ignore
/// // Single component
/// compound.spawn((Position { x: 0.0, y: 0.0 },));
///
/// // Multiple components
/// compound.spawn((
///     Position { x: 0.0, y: 0.0 },
///     Velocity { x: 1.0, y: 0.0 },
///     Health { current: 100, max: 100 }
/// ));
/// ```
///
/// # Implementation Details
///
/// - Uses `#[allow(non_snake_case)]` to permit uppercase generic parameters as variable names
/// - Each component is moved into the tuple destructuring, transferring ownership to the ECS
/// - Components are added in the order they appear in the tuple
/// - All operations are thread-safe due to the internal locking in `add_molecule`
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
