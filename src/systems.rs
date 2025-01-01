use collision::find_overlapping_intervals_2d;

use super::*;

#[system(for_each)]
pub fn update_positions(pos: &mut Position, vel: &Velocity, #[resource] dt: &f32) {
    let dt = *dt as f64;

    pos.0.x += vel.0.x * dt;
    pos.0.y += vel.0.y * dt;
}

#[system]
pub fn handle_mouse_collision(
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
pub fn handle_collisions(
    world: &mut SubWorld,
    main_query: &mut Query<(&Id, &Mass, &mut Position, &mut Velocity, &ShapeInfo)>,
) {
    let mut entities = [const { None }; 10_000];
    main_query.for_each_mut(
        world,
        |(Id(id), mass, pos, vel, ShapeInfo { radius, .. })| {
            entities[*id] = Some((mass, pos, vel, radius));
        },
    );

    let (x_intervals, y_intervals) =
        collision::create_intervals(entities.iter().enumerate().filter_map(|(index, info)| {
            info.as_ref()
                .map(|(_, pos, _, radius)| (index, &**pos, **radius))
        }));

    let overlapping = find_overlapping_intervals_2d(&x_intervals, &y_intervals);

    for (a, b) in overlapping {
        let ((m1, pos1, vel1, radius1), (m2, pos2, vel2, radius2)) =
            (entities[a].as_ref().unwrap(), entities[b].as_ref().unwrap());

        let distance = (pos1.0 - pos2.0).length();
        let combined_radius = **radius1 + **radius2;

        let overlap = combined_radius - distance;
        let direction = (pos1.0 - pos2.0).normalize();
        let correction = direction * overlap / 2.0;

        let (new_vel1, new_vel2) = process_collision(vel1.0, vel2.0, pos1.0, pos2.0, m1.0, m2.0);

        let a = entities[a].as_mut().unwrap();
        a.1.0 += correction;
        a.2.0 = new_vel1;

        let b = entities[b].as_mut().unwrap();
        b.1.0 -= correction;
        b.2.0 = new_vel2;
    }

    // Previous implementation
    // let entities = query.iter_mut(world).collect::<Vec<_>>();
    // let mut updated = [None; 10000];

    // // Check collisions for all pairs
    // for i in 0..entities.len() {
    //     for j in (i + 1)..entities.len() {
    //         let (id1, mass1, pos1, vel1, shape1) = &entities[i];
    //         let (id2, mass2, pos2, vel2, shape2) = &entities[j];

    //         // Calculate distance and combined radius
    //         let distance = (pos1.0 - pos2.0).length();
    //         let combined_radius = shape1.radius + shape2.radius;

    //         if distance < combined_radius {
    //             // Collision detected, process velocities
    //             let (new_vel1, new_vel2) =
    //                 process_collision(vel1.0, vel2.0, pos1.0, pos2.0, mass1.0, mass2.0);

    //             let overlap = combined_radius - distance;
    //             let direction = (pos1.0 - pos2.0).normalize();
    //             let correction = direction * overlap / 2.0;

    //             let mut pos1 = **pos1;
    //             let mut pos2 = **pos2;

    //             // Move the particles apart
    //             pos1.0 += correction;
    //             pos2.0 -= correction;

    //             updated[id1.0] = Some((new_vel1, pos1));
    //             updated[id2.0] = Some((new_vel2, pos2));
    //         }
    //     }
    // }

    // query.iter_mut(world).for_each(|(id, _, pos, vel, _)| {
    //     if let Some((new_vel, new_pos)) = updated[id.0] {
    //         *vel = Velocity(new_vel);
    //         *pos = new_pos;
    //     }
    // });
}

#[system(for_each)]
pub fn update_velocity(vel: &mut Velocity, #[resource] dt: &f32) {
    vel.0.y += GRAVITY * *dt as f64;
}

#[system(for_each)]
pub fn check_wall_collision(
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
