use super::*;

use std::time::Instant;

use egui_sfml::SfEgui;
use egui_sfml::egui;

use legion::*;
use rand::Rng;
use rand::thread_rng;
use sfml::{graphics::*, system::*, window::*};

use glam::DVec2;

use super::systems as sys;

use components::*;

pub fn run() {
    let texture_image = renderer::circle(100, Color::WHITE);
    let mut texture = Texture::from_image(&texture_image, Rect::new(0, 0, 200, 200)).unwrap();
    texture.set_smooth(true);

    let mut world = World::default();
    let mut window = RenderWindow::new(
        (WINDOW_WIDHT, WINDOW_HEIGHT),
        "Particle Simulator",
        Style::CLOSE,
        &ContextSettings {
            // antialiasing_level: 8,
            ..Default::default()
        },
    )
    .unwrap();

    let mut sfegui = SfEgui::new(&window);

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

    let mut mouse_tracker = CircleShape::new(0.0, 1000);
    mouse_tracker.set_origin((50.0, 50.0));
    mouse_tracker.set_radius(50.0);
    mouse_tracker.set_outline_color(Color::rgb(200, 150, 150));
    mouse_tracker.set_fill_color(Color::TRANSPARENT);
    mouse_tracker.set_outline_thickness(4.0);

    let mut clock = Clock::start().unwrap();

    let mut pressed = false;
    let mut num_particles = 0;

    // used in egui
    let mut slower_collision_detection = false;
    let mut draw_quadtree = false;
    let mut quad_capacity = 8;
    let mut point_count = 30;
    let mut fps_limited = false;
    let mut fps_limit = 120;
    let mut particle_radius = 5;
    //

    // let mut shape = CircleShape::new(5.0, 30);
    let mut shape = Sprite::new();
    shape.set_texture(&texture, true);

    let add_ball = |x, y, world: &mut World, num_particles: &mut u32, particle_radius: f64| {
        *num_particles += 1;
        let _ = world.push((
            Id(id()),
            // Mass(thread_rng().gen_range(50.0..=100.0)),
            Mass(1.0),
            Position(DVec2 { x, y }),
            Velocity(DVec2 {
                x: thread_rng().gen_range(-30.0..=30.0),
                y: thread_rng().gen_range(-30.0..=30.0),
            }),
            ShapeInfo {
                radius: particle_radius,
                // radius: 100.0,
                // color: Color::GREEN,
                color: Color::rgb(
                thread_rng().gen_range(0..=255),
                thread_rng().gen_range(0..=255),
                thread_rng().gen_range(0..=255),
                ),
            },
        ));
    };

    while window.is_open() {
        if fps_limited {
            window.set_framerate_limit(fps_limit);
        } else {
            window.set_framerate_limit(0);
        }

        let dt = clock.restart();
        while let Some(event) = window.poll_event() {
            sfegui.add_event(&event);
            match event {
                Event::Closed => window.close(),
                Event::Resized { .. } => resources.insert(window.size()),

                Event::MouseButtonReleased {
                    button: mouse::Button::Right,
                    ..
                } => pressed = false,

                Event::KeyReleased {
                    code: Key::Space, ..
                } => {
                    world.entry(tracker_entity).unwrap().add_component(Disabled);
                    <&mut MouseTracker>::query().for_each_mut(&mut world, |mt| {
                        mt.pos = DVec2::new(-100., -100.);
                        resources.insert(*mt);
                    });
                }

                Event::KeyPressed {
                    code: Key::Space, ..
                } => world
                    .entry(tracker_entity)
                    .unwrap()
                    .remove_component::<Disabled>(),

                Event::MouseButtonPressed {
                    button: mouse::Button::Right,
                    x,
                    y,
                } => {
                    pressed = true;
                    add_ball(
                        x as _,
                        y as _,
                        &mut world,
                        &mut num_particles,
                        particle_radius as f64,
                    );
                }

                Event::MouseMoved { x, y } if pressed => {
                    add_ball(
                        x as _,
                        y as _,
                        &mut world,
                        &mut num_particles,
                        particle_radius as f64,
                    );
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

        let timer = Instant::now();
        let mut query = <(&Id, &Position, &ShapeInfo)>::query();
        let mut qt = quadtree::QuadTree::<usize>::new(quad_capacity, Rect {
            left: 0.,
            top: 0.,
            width: window.size().x as _,
            height: window.size().y as _,
        });

        query.for_each(
            &world,
            |(Id(id), Position(position), ShapeInfo { radius, .. })| {
                qt.push((*position, *radius, *id));
            },
        );

        let qt_build_time = timer.elapsed().as_nanos() as f64 / 1e6;

        window.clear(Color::BLACK);

        if draw_quadtree {
            qt.draw(&mut window, 0);
        }

        resources.insert(qt);
        resources.insert(slower_collision_detection);
        resources.insert(dt.as_seconds());

        schedule.execute(&mut world, &mut resources);

        let fps = (1.0 / dt.as_seconds()) as u32;

        let timer = Instant::now();
        <(&Position, &ShapeInfo)>::query().iter(&world).for_each(
            |(Position(DVec2 { x, y }), ShapeInfo { radius, color })| {
                let scale = *radius as f32 / 100.0;
                shape.set_scale((scale, scale));

                shape.set_position((*x as _, *y as _));
                shape.set_color(*color);

                // shape.set_point_count(point_count);
                shape.set_color(*color);
                // shape.set_radius(*radius as _);
                shape.set_origin((*radius as _, *radius as _));

                window.draw(&shape);
            },
        );

        let draw_time = timer.elapsed().as_nanos() as f64 / 1e6;

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

        let di = sfegui
            .run(&mut window, |_rw, ctx| {
                egui::Window::new("Settings")
                    .default_pos((10.0, 10.0))
                    .collapsible(true)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label(format!("FPS: {}", fps));
                        ui.label(format!("Particles: {num_particles}"));
                        ui.label(format!("Quadtree time: {qt_build_time:.2}ms"));
                        ui.label(format!("Draw time: {draw_time:.2}ms"));
                        ui.separator();
                        ui.checkbox(
                            &mut slower_collision_detection,
                            "Use slower collision detection",
                        );

                        if slower_collision_detection {
                            draw_quadtree = false;
                        }

                        ui.add_enabled(
                            !slower_collision_detection,
                            egui::Checkbox::new(&mut draw_quadtree, "Draw quadtree"),
                        );

                        ui.separator();

                        ui.add_enabled(
                            !slower_collision_detection,
                            egui::Slider::new(&mut quad_capacity, 4..=64).text("Quad capacity"),
                        );

                        ui.add(egui::Slider::new(&mut point_count, 1..=100).text("Point count"));
                        ui.add(
                            egui::Slider::new(&mut particle_radius, 1..=10).text("Point radius"),
                        );

                        ui.horizontal(|ui| {
                            ui.checkbox(&mut fps_limited, "Limit FPS");
                            ui.add_enabled(fps_limited, egui::Slider::new(&mut fps_limit, 1..=1000))
                        });
                    });
            })
            .unwrap();

        sfegui.draw(di, &mut window, None);

        window.display();
    }
}

fn id() -> usize {
    static mut INDEX: usize = 0;

    let ret = unsafe { INDEX };
    unsafe { INDEX += 1 };

    ret
}
