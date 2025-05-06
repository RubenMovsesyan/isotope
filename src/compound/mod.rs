use std::{
    any::Any,
    sync::{Arc, Mutex},
};

use anyhow::{Result, anyhow};

pub trait ComponentVec {
    fn push_none(&self);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: 'static> ComponentVec for Mutex<Vec<Option<T>>> {
    fn push_none(&self) {
        self.lock().unwrap().push(None);
    }

    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }
}

pub struct Compound {
    element_count: usize,
    component_vecs: Vec<Arc<dyn ComponentVec>>,
}

impl Compound {
    pub fn new() -> Self {
        Self {
            element_count: 0,
            component_vecs: Vec::new(),
        }
    }

    pub fn new_entity(&mut self) -> usize {
        let entity_id = self.element_count;

        for component_vec in self.component_vecs.iter() {
            // component_vec.lock().unwrap().push_none();
            component_vec.push_none();
        }
        self.element_count += 1;

        entity_id
    }

    pub fn add_component<T: 'static>(&mut self, element: usize, component: T) {
        for component_type in self.component_vecs.iter() {
            if let Some(component_vec) = component_type
                .as_any()
                .downcast_ref::<Mutex<Vec<Option<T>>>>()
            {
                if let Ok(mut component_vec) = component_vec.lock() {
                    component_vec[element] = Some(component);
                    return;
                }
            }
        }

        let mut new_component_vec: Mutex<Vec<Option<T>>> =
            Mutex::new(Vec::with_capacity(self.element_count));

        for _ in 0..self.element_count {
            new_component_vec.push_none();
        }

        new_component_vec
            .as_any_mut()
            .downcast_mut::<Mutex<Vec<Option<T>>>>()
            .unwrap()
            .lock()
            .unwrap()[element] = Some(component);
        self.component_vecs.push(Arc::new(new_component_vec));
    }

    pub fn query<T: 'static>(&mut self) -> Option<Arc<dyn ComponentVec>> {
        for component_type in self.component_vecs.iter() {
            if let Some(_component_vec) = component_type
                .as_any()
                .downcast_ref::<Mutex<Vec<Option<T>>>>()
            {
                // drop(component_vec);

                return Some(component_type.clone());
            }
        }

        None
    }
}

#[cfg(test)]
mod ecs_test {
    use super::*;

    #[test]
    fn test_query() {
        let mut ecs = Compound::new();

        let dog = ecs.new_entity();
        let cat = ecs.new_entity();

        struct Animal {
            name: String,
            ty: String,
        }

        struct Collar {
            name: String,
        }

        struct Whiskers {
            count: u32,
        }

        println!("here");
        ecs.add_component(
            dog,
            Animal {
                name: String::from("Sparky"),
                ty: String::from("dog"),
            },
        );

        println!("here 2");
        ecs.add_component(
            cat,
            Animal {
                name: String::from("Luna"),
                ty: String::from("cat"),
            },
        );

        println!("here 3");
        ecs.add_component(
            dog,
            Collar {
                name: String::from("Sparky"),
            },
        );

        println!("here 4");
        ecs.add_component(cat, Whiskers { count: 10 });

        let animals = ecs.query::<Animal>().unwrap();

        println!(
            "Num animals: {}",
            animals
                .as_any()
                .downcast_ref::<Mutex<Vec<Option<Animal>>>>()
                .unwrap()
                .lock()
                .unwrap()
                .len()
        );
    }
}
