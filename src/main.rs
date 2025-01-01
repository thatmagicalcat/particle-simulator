use legion::*;
use rand::Rng;
use rand::thread_rng;
use sfml::{graphics::*, system::*, window::*};

use glam::DVec2;
use world::SubWorld;

mod collision;
mod components;
mod systems;

use systems as sys;

use components::*;

const GRAVITY: f64 = 100.0;

/// space wasted by window decorations (approximate value)
const WINDOW_PADDING: u32 = 10;

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
        .add_system(sys::handle_collisions_system())
        .add_system(sys::handle_mouse_collision_system())
        .flush()
        .add_system(sys::update_positions_system())
        .add_system(sys::check_wall_collision_system())
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
                    <&mut MouseTracker>::query().for_each_mut(&mut world, |mt| {
                        mt.pos = DVec2::new(-100., -100.);
                        resources.insert(*mt);
                    });
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
