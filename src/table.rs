use std::{any::Any, collections::HashMap};

// #[test]
// mod test {
    // let mut table = PackedTable::new();
    // table.register::<Name>();
    // table.register::<Age>();

    // println!("Table before \n{:?}", &table);

    // let row = table.add_row();
    // println!("row: {}", &row);
    // println!("Table after \n{:?}", &table);
    // table.get_component::<Name>(row).name = String::from("Willow");
    // table.get_component::<Age>(row).age = 24;
    // println!("Table with data \n{:?}", &table);

    // println!("Row {} has", row);
    // println!("{:?}", &table.get_component::<Name>(row));
    // println!("{:?}", &table.get_component::<Age>(row));
// }

#[derive(Debug)]
pub struct PackedTable {
    data: Vec<u8>,
    info: HashMap<std::any::TypeId, (usize, usize)>,
    row_size: usize,
}

#[derive(Debug)]
pub struct LinearTable {
    data: HashMap<std::any::TypeId, Vec<u8>>,
    type_sizes: HashMap<std::any::TypeId, usize>,
}

impl PackedTable {
    pub fn new() -> Self {
        PackedTable {
            data: Vec::new(),
            info: HashMap::new(),
            row_size: 0,
        }
    }

    pub fn register<T: Sized + Any>(&mut self) {
        let type_id = std::any::TypeId::of::<T>();
        let type_size = std::mem::size_of::<T>();
        self.info.insert(type_id, (type_size, self.row_size));
        self.row_size += type_size;
    }

    pub fn add_row(&mut self) -> u8 {
        let len = self.data.len();
        self.data.resize(len + self.row_size, 0);
        (len / self.row_size) as u8
    }

    pub fn get_component<T: Any>(&mut self, row: u8)-> &mut T {
        let info = self.info[&std::any::TypeId::of::<T>()];
        let data = self.data.get_mut(row as usize * self.row_size + info.1).unwrap();
        unsafe {
            &mut *(data as *mut u8 as *mut T)
        }
    }
}

impl LinearTable {
    pub fn new() -> Self {
        LinearTable {
            data: HashMap::new(),
            type_sizes: HashMap::new(),
        }
    }

    pub fn register<T: Sized + Any>(&mut self) {
        let type_id = std::any::TypeId::of::<T>();
        let type_size = std::mem::size_of::<T>();
        self.type_sizes.insert(type_id, type_size);
        self.data.insert(type_id, Vec::new());
    }

    pub fn add_row(&mut self) -> u8 {
        let mut size = 0;
        for (type_id, data) in self.data.iter_mut() {
            size = data.len() as u8;

            let type_size = self.type_sizes[type_id];
            data.resize(data.len() + type_size, 0);
        }

        size
    }

    pub fn get_component<T: Any>(&mut self, row: u8)-> &mut T {
        let type_size = self.type_sizes[&std::any::TypeId::of::<T>()];
        let data = self.data.get_mut(&std::any::TypeId::of::<T>()).unwrap();
        let component_ptr = data.get_mut(row as usize * type_size).unwrap();
        unsafe {
            &mut *(component_ptr as *mut u8 as *mut T)
        }
    }
}