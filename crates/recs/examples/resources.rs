use recs::prelude::*;

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct Velocity {
    dx: f32,
    dy: f32,
}

// Resources
#[derive(Resource, Debug, Clone)]
struct GameTime {
    delta_time: f32,
    total_time: f32,
}

impl Default for GameTime {
    fn default() -> Self {
        Self {
            delta_time: 0.016, // ~60 FPS
            total_time: 0.0,
        }
    }
}

#[derive(Resource, Debug, Clone)]
struct GameConfig {
    gravity: f32,
    max_speed: f32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            gravity: -9.81,
            max_speed: 100.0,
        }
    }
}

#[derive(Resource, Debug, Default)]
struct GameStats {
    entities_moved: usize,
    out_of_bounds_entities: usize,
}

fn main() {
    let mut registry = Registry::new();

    registry.init_resource::<GameTime>();
    registry.init_resource::<GameConfig>();
    registry.init_resource::<GameStats>();

    for i in 0..5 {
        registry.spawn((
            Position {
                x: i as f32 * 50.0,
                y: 300.0,
            },
            Velocity {
                dx: (i as f32 - 2.0) * 20.0,
                dy: 0.0,
            },
        ));
    }

    registry.add_system(time_system);
    registry.add_system(movement_system);
    registry.add_system(stats_system);

    for frame in 0..30 {
        if let Some(time) = registry.get_resource_mut::<GameTime>() {
            time.delta_time = 0.016 + (frame as f32 * 0.0001);
            time.total_time += time.delta_time;
        }

        registry.run_systems();

        if frame % 50 == 0 {
            if let Some(stats) = registry.get_resource::<GameStats>() {
                println!(
                    "Frame {}: Moved: {}, Out of bounds: {}",
                    frame, stats.entities_moved, stats.out_of_bounds_entities
                );
            }
        }
    }

    if let Some(stats) = registry.get_resource::<GameStats>() {
        println!("Final stats: {:?}", stats);
    }
}

fn time_system(time: ResMut<GameTime>) {
    println!(
        "Game time: {:.2}s, Delta: {:.4}s",
        time.total_time, time.delta_time
    );
}

fn movement_system(
    query: Query<(&mut Position, &Velocity)>,
    time: Res<GameTime>,
    config: Res<GameConfig>,
    mut stats: ResMut<GameStats>,
) {
    stats.entities_moved = 0;

    for (pos, vel) in query {
        let gravity_effect = config.gravity * time.delta_time;

        pos.x += vel.dx * time.delta_time;
        pos.y += vel.dy * time.delta_time + gravity_effect * time.delta_time;

        let speed = (vel.dx * vel.dx + vel.dy * vel.dy).sqrt();
        if speed > config.max_speed {}

        stats.entities_moved += 1;
    }
}

fn stats_system(optional_time: OptionalRes<GameTime>, optional_stats: OptionalRes<GameStats>) {
    if let (Some(time), Some(stats)) = (optional_time.as_ref(), optional_stats.as_ref()) {
        if time.total_time as u32 % 2 == 0 {
            println!(
                "Periodic stats check - Entities moved: {}",
                stats.entities_moved
            );
        }
    }
}
