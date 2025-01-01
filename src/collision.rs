//! Contains functions/structs required for the collision detection

use super::*;

#[derive(Debug, Clone, Copy)]
pub struct Interval {
    /// Id of the entity
    id: usize,

    start: f64,
    end: f64,
}

/// X and Y axis respectively
pub fn create_intervals<'a>(
    entities: impl Iterator<Item = (usize, &'a Position, f64)>,
) -> (Vec<Interval>, Vec<Interval>) {
    let mut interval1 = vec![];
    let mut interval2 = vec![];

    entities.for_each(|(id, pos, radius)| {
        interval1.push(Interval {
            id,
            start: pos.0.x - radius,
            end: pos.0.x + radius,
        });

        interval2.push(Interval {
            id,
            start: pos.0.y - radius,
            end: pos.0.y + radius,
        });
    });

    sort_intervals(&mut interval1);
    sort_intervals(&mut interval2);

    (interval1, interval2)
}

fn sort_intervals(intervals: &mut [Interval]) {
    intervals.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());
}

/// Returns the id of the entities overlapping as pairs
pub fn find_overlapping_intervals_2d(
    x_intervals: &[Interval],
    y_intervals: &[Interval],
) -> Vec<(usize, usize)> {
    let mut active_x: Vec<Interval> = Vec::with_capacity(x_intervals.len());
    let mut active_y: Vec<Interval> = Vec::with_capacity(y_intervals.len());
    let mut overlapping: Vec<(usize, usize)> = Vec::with_capacity(x_intervals.len());

    for (x_interval, y_interval) in x_intervals.iter().zip(y_intervals) {
        // Remove ended intervals in X and Y axes
        active_x.retain(|active| active.end > x_interval.start);
        active_y.retain(|active| active.end > y_interval.start);

        // Find overlapping intervals that exist in both axes
        for active_x in &active_x {
            for active_y in &active_y {
                if active_x.id == active_y.id && active_x.id != x_interval.id {
                    overlapping.push((active_x.id, x_interval.id));
                }
            }
        }

        active_x.push(*x_interval);
        active_y.push(*y_interval);
    }

    overlapping
}
