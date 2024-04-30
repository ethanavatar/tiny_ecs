use crate::world::World;

pub struct Systems {
    systems: Vec<Box<dyn Fn(&mut World)>>,
}

impl Systems {
    pub fn new() -> Self {
        Systems {
            systems: Vec::new(),
        }
    }

    pub fn add_system<F: 'static>(&mut self, system: F)
    where
        F: Fn(&mut World),
    {
        self.systems.push(Box::new(system));
    }

    pub fn run(&self, world: &mut World) {
        self.systems.iter().for_each(|s| s(world));
    }
}
