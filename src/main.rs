mod world;
use world::World;
use world::Systems;

#[derive(Debug)]
#[allow(dead_code)]
struct Health(i32);

#[derive(Debug)]
#[allow(dead_code)]
struct Position(f32, f32);

fn hurt_all(world: &mut World) {
    let mut health = world.borrow_components::<Health>().unwrap();
    for h in health.iter_mut().flatten() {
        h.0 -= 50;
    }
}

fn kill_dead(world: &mut World) {
    let mut dead_entities = Vec::new();
    {
        let health = world.borrow_components::<Health>().unwrap();
        let positions = world.borrow_components::<Position>().unwrap();

        let it = health
            .iter()
            .zip(positions.iter())
            .collect::<Vec<_>>();

        for (entity_id, (h, p)) in it.iter().enumerate() {
            if let (Some(h), Some(p)) = (h, p) {
                if h.0 <= 0 {
                    println!("Entity (Id: {}) at {:?} died", entity_id, p);
                    dead_entities.push(entity_id);
                }
            }
        }
    }

    for entity_id in dead_entities {
        println!("Removing entity {}", entity_id);
        world.remove_entity(entity_id);
    }
}

fn print_components(world: &mut World) {
    let health = world.borrow_components::<Health>().unwrap();
    let positions = world.borrow_components::<Position>().unwrap();

    let it = health
        .iter()
        .zip(positions.iter())
        .collect::<Vec<_>>();

    for (entity_id, (h, p)) in it.iter().enumerate() {
        if let (Some(h), Some(p)) = (h, p) {
            println!(
                "Entity {}: Health: {:?}, Position: {:?}",
                entity_id, h.0, p
            );
        }
    }
}

fn main() {
    let mut world = World::new();
    let entity1 = world.new_entity();
    world.add_component(entity1, Health(50));
    world.add_component(entity1, Position(1.0, 1.0));

    let entity2 = world.new_entity();
    world.add_component(entity2, Health(100));
    world.add_component(entity2, Position(2.0, 1.0));

    let mut systems = Systems::new();
    systems.add_system(hurt_all);
    systems.add_system(print_components);
    systems.add_system(kill_dead);

    systems.run(&mut world);

    println!("After update 1");
    println!("----------------------");
    println!("entity count: {}", world.count_entities());
    print_components(&mut world);

    let entity3 = world.new_entity();
    world.add_component(entity3, Health(69));
    world.add_component(entity3, Position(3.0, 1.0));

    println!("After adding entity 3");
    println!("entity count: {}", world.count_entities());
    print_components(&mut world);
}
