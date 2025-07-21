use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, RwLock},
};

use anyhow::{Result, anyhow};
use log::warn;

pub struct SharedMatter<T>(Arc<RwLock<T>>);

unsafe impl<T> Send for SharedMatter<T> {}
unsafe impl<T> Sync for SharedMatter<T> {}

impl<T> SharedMatter<T> {
    pub fn new(t: T) -> Self {
        Self(Arc::new(RwLock::new(t)))
    }

    pub fn with_read<F, R>(&self, callback: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let matter = match self.0.read() {
            Ok(matter) => matter,
            Err(poisoned) => {
                warn!("Lock was poisoned, recovering...");
                poisoned.into_inner()
            }
        };

        callback(&matter)
    }

    pub fn with_write<F, R>(&self, callback: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut matter = match self.0.write() {
            Ok(matter) => matter,
            Err(poisoned) => {
                warn!("Lock was poisoned, recovering...");
                poisoned.into_inner()
            }
        };

        callback(&mut matter)
    }
}

impl<T> Clone for SharedMatter<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub struct MatterVault {
    matter: Arc<RwLock<HashMap<TypeId, HashMap<String, Box<dyn Any>>>>>,
}

impl MatterVault {
    pub fn new() -> Self {
        Self {
            matter: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn read<T: 'static, S, F, R>(&self, specifier: S, callback: F) -> Result<R>
    where
        S: AsRef<str>,
        F: FnOnce(&T) -> R,
    {
        let map = match self.matter.read() {
            Ok(map) => map,
            Err(poisoned) => {
                warn!("Matter Manager has been poisoned, recovering...");
                poisoned.into_inner()
            }
        };

        Ok(map
            .get(&TypeId::of::<T>())
            .ok_or(anyhow!("Data Label Does Not Exist"))?
            .get(specifier.as_ref())
            .ok_or(anyhow!("Specifier Does Not Exist"))?
            .downcast_ref::<SharedMatter<T>>()
            .ok_or(anyhow!("Failed to downcast to type"))?
            .with_read(callback))
    }

    pub fn write<T: 'static, S, F, R>(&self, specifier: S, callback: F) -> Result<R>
    where
        S: AsRef<str>,
        F: FnOnce(&mut T) -> R,
    {
        let map = match self.matter.read() {
            Ok(map) => map,
            Err(poisoned) => {
                warn!("Matter Manager has been poisoned, recovering...");
                poisoned.into_inner()
            }
        };

        Ok(map
            .get(&TypeId::of::<T>())
            .ok_or(anyhow!("Data Label Does Not Exist"))?
            .get(specifier.as_ref())
            .ok_or(anyhow!("Specifier Does Not Exist"))?
            .downcast_ref::<SharedMatter<T>>()
            .ok_or(anyhow!("Failed to downcast to type"))?
            .with_write(callback))
    }

    pub fn add<T: 'static, S>(&self, specifier: S, value: T)
    where
        S: AsRef<str>,
    {
        let mut map = match self.matter.write() {
            Ok(map) => map,
            Err(poisoned) => {
                warn!("Matter Manager has been poisoned, recovering...");
                poisoned.into_inner()
            }
        };

        let type_id = TypeId::of::<T>();
        let specifier_str = specifier.as_ref().to_string();
        let shared_matter = Box::new(SharedMatter::new(value));

        map.entry(type_id)
            .or_insert_with(HashMap::new)
            .insert(specifier_str, shared_matter);
    }
}
