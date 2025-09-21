use recs::prelude::*;

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

    registry.add_system(movement_system);

    for _ in 0..100 {
        registry.run_systems();

        let pos = registry.get_component::<Position>(entity).unwrap();
        println!("{}", pos.x);
    }
}

fn movement_system(query: Query<(&mut Position, &Velocity)>) {
    for (pos, vel) in query {
        pos.x += vel.dx;
        pos.y += vel.dy;
    }
}
