use std::collections::HashMap;

// mod table;

#[derive(Debug, Clone, Copy)]
struct EntityHandle(u32);

struct Component<T> {
    entity: EntityHandle,
    data: T,
}

use std::any::TypeId;
use std::slice;

#[derive(Debug)]
struct DECS {
    next_entity: u32,
    // components stored in blobs.
    components: HashMap<TypeId, Vec<u8>>,
    entities: Vec<EntityData>,
}

#[derive(Debug)]
struct EntityData {
    handle: EntityHandle,
    components: Vec<TypeId>,
}

impl EntityData {
    fn new(handle: EntityHandle) -> Self {
        EntityData {
            handle,
            components: Vec::new(),
        }
    }
}

impl DECS {
    const _INVALID_HANDLE: EntityHandle = EntityHandle(u32::MAX);

    fn new() -> Self {
        Self {
            next_entity: 0,
            components: HashMap::new(),
            entities: Vec::new(),
        }
    }
    fn create_entity(&mut self) -> EntityHandle {
        let new_handle = EntityHandle(self.next_entity);

        // TODO WT: This would do well to probably be a hashmap as we're gonna be checking it often
        self.entities.push(EntityData::new(new_handle));

        self.next_entity += 1;
        new_handle
    }

    fn register_component<T: Sized + std::any::Any>(&mut self) {
        self.components.insert(TypeId::of::<Component<T>>(), Vec::new());
    }

    fn add_component<T: Sized + std::any::Any>(&mut self, entity: EntityHandle, component: T) {
        let type_id = TypeId::of::<Component<T>>();
        let blob = self.components.get_mut(&type_id).expect("Component was not registered");

        let entity_data = self.entities.iter_mut().find(|entity_data| entity_data.handle.0 == entity.0).expect("Entity was not found");

        if entity_data.components.iter().find(|id| **id == type_id).is_some() {
            panic!("Entity already had that component on it");
        }

        entity_data.components.push(type_id);

        println!("Size of new Component<T>: {}", std::mem::size_of::<Component<T>>());

        let old_len = blob.len();
        blob.resize(old_len + std::mem::size_of::<Component<T>>(), 0);
        let last_element_sized_t = blob.as_mut_ptr() as usize + old_len;

        let wrapped = Component {
            entity,
            data: component,
        };

        unsafe {
            std::ptr::write(last_element_sized_t as *mut Component<T>, wrapped);
        }
    }

    fn get_component<T: Sized + std::any::Any>(&self, entity: EntityHandle) -> Option<&T> {
        if let Some(blob) = self.components.get(&TypeId::of::<Component<T>>()) {
            unsafe {
                let typed_array_size = blob.len() / std::mem::size_of::<T>();
                let slice = std::slice::from_raw_parts(blob.as_ptr() as *const Component<T>, typed_array_size);

                for i in 0..slice.len() {
                    let data = &slice[i];
                    if data.entity.0 == entity.0 {
                        return Some(&data.data);
                    }
                }

                None
            }
        } else {

            None
        }
    }

    fn get_mut_component<T: Sized + std::any::Any>(&mut self, entity: EntityHandle) -> Option<&mut T> {
        if let Some(blob) = self.components.get_mut(&TypeId::of::<Component<T>>()) {
            unsafe {
                let typed_array_size = blob.len() / std::mem::size_of::<T>();
                let slice = std::slice::from_raw_parts_mut(blob.as_ptr() as *mut Component<T>, typed_array_size);

                for data in slice {
                    if data.entity.0 == entity.0 {
                        return Some(&mut data.data);
                    }
                    drop(data);
                }

                None
            }
        } else {

            None
        }
    }

    fn remove_component<T: Sized + std::any::Any>(&mut self, entity: EntityHandle) {
        let blob = self.components.get_mut(&TypeId::of::<Component<T>>()).expect("Component not registered, cannot remove");

        unsafe {
            let as_t = slice::from_raw_parts_mut(blob.as_mut_ptr() as *mut Component<T>, blob.len() / std::mem::size_of::<Component<T>>());
            let mut index = None;
            for i in 0..as_t.len() {
                if as_t[i].entity.0 == entity.0 {
                    index = Some(i);
                    break;
                }
            }

            if let Some(i) = index {
                as_t.swap(i, as_t.len() - 1);
                blob.drain(blob.len() - std::mem::size_of::<Component<T>>()..);
            }


            // also need to remove the component from the entitydata

            let entity_data = self.entities.iter_mut().find(|entity_data| entity_data.handle.0 == entity.0).expect("Entity was not found");

            index = None;
            for (i, &t) in entity_data.components.iter().enumerate() {
                if t == TypeId::of::<Component<T>>() {
                    index = Some(i);
                }
            }

            if let Some(i) = index {
                let len = entity_data.components.len();
                entity_data.components.swap(i, len - 1);

                entity_data.components.pop();
            }
        }
    }
}

#[derive(Debug)]
struct Name {
    name: String,
}

#[derive(Debug)]
struct Age(u8);

fn main() {
    let mut decs = DECS::new();
    // decs.register_component::<Name>();

    decs.register_component::<Age>();

    let entity = decs.create_entity();
    let entity1 = decs.create_entity();
    let entity2 = decs.create_entity();

    println!("New entity!: {:?}", &entity);
    println!("Decs: {:?}", &decs);

    decs.add_component(entity, Age(24));
    println!("Decs: {:?}", &decs);
    decs.add_component(entity1, Age(12));
    println!("Decs: {:?}", &decs);
    decs.add_component(entity2, Age(43));
    println!("Decs: {:?}", &decs);

    let e0_age = decs.get_component::<Age>(entity).expect("Entity did not have Age");

    println!("Entity 0 age: {:#?}", &e0_age);

    decs.get_mut_component::<Age>(entity2).expect("Entity did not have Age").0 = 55;


    println!("Change e2 age to 55: {:#?}", &decs);

    // decs.add_component(entity, Age(200));

    decs.remove_component::<Age>(entity);
    println!("Remove Age from e0{:#?}", &decs);
}
