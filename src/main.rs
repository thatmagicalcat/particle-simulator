use legion::*;
use rand::Rng;
use rand::thread_rng;
use sfml::{graphics::*, system::*, window::*};

use glam::DVec2;
use world::SubWorld;

const GRAVITY: f64 = 100.0;
const WINDOW_PADDING: u32 = 10;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Mass(f64);

#[derive(Clone, Copy, Debug, PartialEq)]
struct Position(DVec2);

#[derive(Clone, Copy, Debug, PartialEq)]
struct Velocity(DVec2);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Id(usize);

#[derive(Clone, Copy, Debug, PartialEq)]
struct Disabled;

#[derive(Clone, Copy, Debug, PartialEq)]
struct MouseTracker {
    pos: DVec2,
    radius: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ShapeInfo {
    radius: f64,
    color: Color,
}

fn main() {
    let mut world = World::default();
    let mut window = RenderWindow::new(
        (800, 800),
        "Particle Simulator",
        Style::CLOSE,
        &ContextSettings {
            antialiasing_level: 2,
            ..Default::default()
        },
    )
    .unwrap();

    let mut info_text = {
        let mut font: Box<sfml::cpp::FBox<Font>> =
            Box::new(Font::from_file("Hack NF.ttf").unwrap());
        font.set_smooth(true);
        let mut text = Text::default();
        text.set_string("");
        text.set_font(Box::leak(font));
        text.set_character_size(20);
        text
    };

    info_text.set_position((10.0, 10.0));
    info_text.set_fill_color(Color::WHITE);

    let mut resources = Resources::default();
    let mut schedule = Schedule::builder()
        .add_system(handle_collisions_system())
        .add_system(handle_mouse_collision_system())
        .flush()
        .add_system(update_positions_system())
        // .add_system(update_velocity_system())
        .add_system(check_wall_collision_system())
        .build();

    // effective radius: radius + outline thickness
    let mt = MouseTracker {
        radius: 54.0,
        pos: DVec2::new(-100., -100.),
    };

    let tracker_entity = world.push((mt, Disabled));

    resources.insert(window.size());
    resources.insert(mt);

    let mut shape = CircleShape::new(0.0, 10);

    let mut mouse_tracker = CircleShape::new(0.0, 1000);
    mouse_tracker.set_origin((50.0, 50.0));
    mouse_tracker.set_radius(50.0);
    mouse_tracker.set_outline_color(Color::rgb(200, 150, 150));
    mouse_tracker.set_fill_color(Color::TRANSPARENT);
    mouse_tracker.set_outline_thickness(4.0);

    let mut clock = Clock::start().unwrap();

    let mut pressed = false;
    let mut num_particles = 0;

    let add_ball = |x, y, world: &mut World, num_particles: &mut u32| {
        *num_particles += 1;
        let _ = world.push((
            Id(id()),
            Mass(50.0),
            Position(DVec2 { x, y }),
            Velocity(DVec2 {
                x: thread_rng().gen_range(-50.0..=50.0),
                y: thread_rng().gen_range(-50.0..=50.0),
            }),
            ShapeInfo {
                radius: thread_rng().gen_range(5.0..=10.0),
                color: Color::rgb(
                    thread_rng().gen_range(0..=255),
                    thread_rng().gen_range(0..=255),
                    thread_rng().gen_range(0..=255),
                ),
            },
        ));
    };

    while window.is_open() {
        let dt = clock.restart();
        while let Some(event) = window.poll_event() {
            #[allow(clippy::single_match)]
            match event {
                Event::Closed => window.close(),
                Event::Resized { .. } => resources.insert(window.size()),

                Event::MouseButtonReleased {
                    button: mouse::Button::Left,
                    ..
                } => pressed = false,

                Event::MouseButtonReleased {
                    button: mouse::Button::Right,
                    ..
                } => {
                    world.entry(tracker_entity).unwrap().add_component(Disabled);
                    <&mut MouseTracker>::query()
                        .for_each_mut(&mut world, |mt| mt.pos = DVec2::new(-100., -100.));
                }

                Event::MouseButtonPressed {
                    button: mouse::Button::Right,
                    ..
                } => world
                    .entry(tracker_entity)
                    .unwrap()
                    .remove_component::<Disabled>(),

                Event::MouseButtonPressed {
                    button: mouse::Button::Left,
                    x,
                    y,
                } => {
                    pressed = true;
                    add_ball(x as _, y as _, &mut world, &mut num_particles);
                }

                Event::MouseMoved { x, y } if pressed => {
                    add_ball(x as _, y as _, &mut world, &mut num_particles);
                    <&mut MouseTracker>::query()
                        .filter(!component::<Disabled>())
                        .for_each_mut(&mut world, |m| {
                            m.pos = DVec2 {
                                x: x as _,
                                y: y as _,
                            };

                            resources.insert(*m);
                        });
                }

                Event::MouseMoved { x, y } => <&mut MouseTracker>::query()
                    .filter(!component::<Disabled>())
                    .for_each_mut(&mut world, |m| {
                        m.pos = DVec2 {
                            x: x as _,
                            y: y as _,
                        };

                        resources.insert(*m);
                    }),

                _ => {}
            }
        }

        resources.insert(dt.as_seconds());
        schedule.execute(&mut world, &mut resources);

        let fps = 1.0 / dt.as_seconds();
        info_text.set_string(&format!("FPS: {:.0}\nParticles: {num_particles}", fps));

        window.clear(Color::BLACK);

        <(&Position, &ShapeInfo)>::query().iter(&world).for_each(
            |(Position(DVec2 { x, y }), ShapeInfo { radius, color })| {
                shape.set_position((*x as _, *y as _));
                shape.set_fill_color(*color);
                shape.set_radius(*radius as _);

                window.draw(&shape);
            },
        );

        <&MouseTracker>::query()
            .filter(!component::<Disabled>())
            .iter(&world)
            .for_each(
                |MouseTracker {
                     pos: DVec2 { x, y },
                     ..
                 }| {
                    mouse_tracker.set_position((*x as _, *y as _));
                    window.draw(&mouse_tracker);
                },
            );

        window.draw(&info_text);
        window.display();
    }
}

#[system(for_each)]
fn update_positions(pos: &mut Position, vel: &Velocity, #[resource] dt: &f32) {
    let dt = *dt as f64;

    pos.0.x += vel.0.x * dt;
    pos.0.y += vel.0.y * dt;
}

#[system]
fn handle_mouse_collision(
    world: &mut SubWorld,
    query: &mut Query<(&mut Position, &mut Velocity, &ShapeInfo)>,
    #[resource] MouseTracker { radius, pos }: &MouseTracker,
) {
    query.for_each_mut(world, |(pos1, vel, shape)| {
        let distance = (pos - pos1.0).length();
        let combined_radius = radius + shape.radius;

        if distance < combined_radius {
            let overlap = combined_radius - distance;
            let normal = (pos - pos1.0).normalize();
            let correction = normal * overlap / 2.0;

            *vel = Velocity(vel.0 - 2.0 * vel.0.dot(normal) * normal);
            pos1.0 -= correction
        }
    });
}

#[system]
fn handle_collisions(
    world: &mut SubWorld,
    query: &mut Query<(&Id, &Mass, &mut Position, &mut Velocity, &ShapeInfo)>,
) {
    let entities = query.iter_mut(world).collect::<Vec<_>>();
    let mut updated = [None; 10000];

    // Check collisions for all pairs
    for i in 0..entities.len() {
        for j in (i + 1)..entities.len() {
            let (id1, mass1, pos1, vel1, shape1) = &entities[i];
            let (id2, mass2, pos2, vel2, shape2) = &entities[j];

            // Calculate distance and combined radius
            let distance = (pos1.0 - pos2.0).length();
            let combined_radius = shape1.radius + shape2.radius;

            if distance < combined_radius {
                // Collision detected, process velocities
                let (new_vel1, new_vel2) =
                    process_collision(vel1.0, vel2.0, pos1.0, pos2.0, mass1.0, mass2.0);

                let overlap = combined_radius - distance;
                let direction = (pos1.0 - pos2.0).normalize();
                let correction = direction * overlap / 2.0;

                let mut pos1 = **pos1;
                let mut pos2 = **pos2;

                // Move the particles apart
                pos1.0 += correction;
                pos2.0 -= correction;

                updated[id1.0] = Some((new_vel1, pos1));
                updated[id2.0] = Some((new_vel2, pos2));
            }
        }
    }

    query.iter_mut(world).for_each(|(id, _, pos, vel, _)| {
        if let Some((new_vel, new_pos)) = updated[id.0] {
            *vel = Velocity(new_vel);
            *pos = new_pos;
        }
    });
}

#[system(for_each)]
fn update_velocity(vel: &mut Velocity, #[resource] dt: &f32) {
    vel.0.y += GRAVITY * *dt as f64;
}

#[system(for_each)]
fn check_wall_collision(
    pos: &mut Position,
    vel: &mut Velocity,
    ShapeInfo { radius, .. }: &ShapeInfo,
    #[resource] size: &Vector2u,
) {
    if pos.0.x < 0.0 {
        vel.0.x *= -1.0;
        pos.0.x = 0.0;
    } else if pos.0.x + radius >= (size.x - WINDOW_PADDING) as f64 {
        vel.0.x *= -1.0;
        pos.0.x = (size.x - WINDOW_PADDING) as f64 - radius;
    }

    if pos.0.y < 0.0 {
        vel.0.y *= -1.0;
        pos.0.y = 0.0;
    } else if pos.0.y + radius >= (size.y - WINDOW_PADDING) as f64 {
        vel.0.y *= -1.0;
        pos.0.y = (size.y - WINDOW_PADDING) as f64 - radius;
    }
}

/// Returns vf1 and vf2 respectively
fn process_collision(
    v1: DVec2,
    v2: DVec2,
    s1: DVec2,
    s2: DVec2,
    m1: f64,
    m2: f64,
) -> (DVec2, DVec2) {
    (
        v1 - (2.0 * m2) / (m1 + m2)
            * ((v1 - v2).dot(s1 - s2) / (s1 - s2).length_squared())
            * (s1 - s2),
        v2 - (2.0 * m1) / (m1 + m1)
            * ((v2 - v1).dot(s2 - s1) / (s2 - s1).length_squared())
            * (s2 - s1),
    )
}

fn id() -> usize {
    static mut INDEX: usize = 0;

    let ret = unsafe { INDEX };
    unsafe { INDEX += 1 };

    ret
}
