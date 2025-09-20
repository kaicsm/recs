use recs::{Component, registry::Registry};

#[derive(Component)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Velocity {
    dx: i32,
    dy: i32,
}

fn main() {
    let mut registry = Registry::new();

    let entity = registry.spawn((Position { x: 0, y: 0 }, Velocity { dx: 1, dy: 0 }));

    for _ in 0..100 {
        movement_system(&mut registry);

        let pos = registry.get_component::<Position>(entity).unwrap();
        println!("{}", pos.x);
    }
}

fn movement_system(registry: &mut Registry) {
    for (pos, vel) in registry.query::<(&mut Position, &Velocity)>() {
        pos.x += vel.dx;
        pos.y += vel.dy;
    }
}
