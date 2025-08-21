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

    pub fn read<F, R>(&self, callback: F) -> R
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

    pub fn write<F, R>(&self, callback: F) -> R
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
    matter: RwLock<HashMap<TypeId, HashMap<String, Box<dyn Any>>>>,
}

impl MatterVault {
    pub fn new() -> Self {
        Self {
            matter: RwLock::new(HashMap::new()),
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
            .read(callback))
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
            .write(callback))
    }

    pub fn share<T: 'static, S>(&self, specifier: S) -> Result<SharedMatter<T>>
    where
        S: AsRef<str>,
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
            .clone())
    }

    pub fn add<T: 'static, S>(&self, specifier: S, value: T) -> Result<SharedMatter<T>>
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
        let shared_matter = SharedMatter::new(value);
        let shared_matter_box = Box::new(shared_matter.clone());

        map.entry(type_id)
            .or_insert_with(HashMap::new)
            .insert(specifier_str, shared_matter_box);

        Ok(shared_matter)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic_matter_vault() {
        let matter_vault = MatterVault::new();

        _ = matter_vault.add("first_number", 10u32);
        _ = matter_vault.add("second_number", 11u32);

        assert!(
            matter_vault
                .read("first_number", |number: &u32| {
                    println!("First Number is: {}", number);

                    assert!(*number == 10u32);
                })
                .is_ok()
        );

        assert!(
            matter_vault
                .read("second_number", |number: &u32| {
                    println!("Second Number is: {}", number);

                    assert!(*number == 11u32);
                })
                .is_ok()
        );
    }

    #[test]
    fn test_matter_vault_share() {
        let matter_vault = MatterVault::new();

        _ = matter_vault.add("first_number", 10u32);
        _ = matter_vault.add("second_number", 11u32);

        let first_number: SharedMatter<u32> =
            if let Ok(first_number) = matter_vault.share("first_number") {
                first_number
            } else {
                assert!(false, "Failed to share first_number");
                panic!();
            };

        first_number.read(|number| {
            println!("Shared First Number: {}", number);
        });

        let second_number: SharedMatter<u32> =
            if let Ok(second_number) = matter_vault.share("second_number") {
                second_number
            } else {
                assert!(false, "Failed to share second_number");
                panic!();
            };

        second_number.read(|number| {
            println!("Shared Second Number: {}", number);
        });
    }
}
