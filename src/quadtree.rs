use super::*;

const QUADCAPACITY: usize = 16;

/// Make sure not to put a big object as the generic
/// type parameter since there's a lot of cloning.
#[derive(Debug, Clone)]
pub struct QuadTree<T: Clone> {
    boundary: Rect<f64>,
    capacity: usize,
    points: Vec<Option<(DVec2, f64, T)>>,
    children: Option<Box<[QuadTree<T>; 4]>>,
}

impl<T: Clone> QuadTree<T> {
    pub fn new(capacity: usize, boundary: Rect<f64>) -> Self {
        Self {
            capacity,
            boundary,
            points: Vec::with_capacity(capacity),
            children: None,
        }
    }

    pub fn len(&self) -> usize {
        self.points.len()
            + self
                .children
                .as_ref()
                .map(|i| i.iter().map(|i| i.len()).sum())
                .unwrap_or(0)
    }

    /// Depth should be 0
    pub fn draw(&self, target: &mut sfml::graphics::RenderWindow, depth: usize) {
        let mut rect = sfml::graphics::RectangleShape::new();

        rect.set_size(self.boundary.size().as_other());
        rect.set_position(self.boundary.position().as_other());

        let color_value = (255.0 * (1.0 / depth as f32)).ceil() as u8;

        const MAX_DEPTH: usize = 10;

        let clamped_depth = depth.min(MAX_DEPTH);
        let scaling_factor = 1.0 - (clamped_depth as f32 / MAX_DEPTH as f32);
        let color = 100.max((255.0 * scaling_factor).round().clamp(0.0, 255.0) as u8);

        rect.set_outline_color(Color::rgb(color, color, color));
        rect.set_outline_thickness(0.5);
        rect.set_fill_color(Color::TRANSPARENT);

        target.draw(&rect);

        let mut cs = CircleShape::new(2.0, 5);
        cs.set_fill_color(Color::RED);

        if let Some(children) = &self.children {
            for child in children.iter() {
                child.draw(target, depth + 1);
            }
        }
    }

    /// Get all the points which lie inside the specified area
    pub fn query(&self, circle_centre: DVec2, circle_radius: f64) -> Vec<T> {
        let mut found = vec![];

        if !cr_intersection(circle_centre, circle_radius, self.boundary) {
            return found;
        }

        found.extend(
            self.points
                .iter()
                .filter_map(|i| (*i).clone())
                .filter(|(i_circle_centre, i_radius, _)| {
                    cc_intersection(*i_circle_centre, *i_radius, circle_centre, circle_radius)
                })
                .map(|(pos, radius, data)| data),
        );

        if let Some(children) = &self.children {
            children
                .iter()
                .map(|c| c.query(circle_centre, circle_radius))
                .for_each(|i| found.extend(i));
        }

        found
    }

    pub fn push(&mut self, point: (DVec2, f64, T)) {
        if !inside_boundary(&self.boundary, &point.0) {
            return;
        }

        if self.points.len() < self.capacity {
            self.points.push(Some(point));
            return;
        }

        // sub-divide into 4 parts

        if self.children.is_none() {
            let Rect {
                left,
                top,
                width,
                height,
            } = self.boundary;

            let hw = width * 0.5; // half width
            let hh = height * 0.5; // half height

            self.children = Some(Box::new([
                // top left
                QuadTree::new(self.capacity, Rect {
                    left,
                    top,
                    width: hw,
                    height: hh,
                }),
                // top right
                QuadTree::new(self.capacity, Rect {
                    left: left + hw,
                    top,
                    width: hw,
                    height: hh,
                }),
                // bottom left
                QuadTree::new(self.capacity, Rect {
                    left,
                    top: top + hh,
                    width: hw,
                    height: hh,
                }),
                // bottom right
                QuadTree::new(self.capacity, Rect {
                    left: left + hw,
                    top: top + hh,
                    width: hw,
                    height: hh,
                }),
            ]));
        }

        self.children
            .as_mut()
            .unwrap()
            .iter_mut()
            .for_each(|c| c.push(point.clone()));
    }
}

fn inside_boundary(boundary: &Rect<f64>, point: &DVec2) -> bool {
    let Rect {
        left,
        top,
        width,
        height,
    } = *boundary;

    (left <= point.x && point.x < left + width) && (top <= point.y && point.y < top + height)
}

// fn inside_circle(point: &DVec2, circle_centre: DVec2, circle_radius: f64) -> bool {
// (circle_centre - point).length_squared() <= circle_radius * circle_radius
// }

/// Checks circle-rectangle intersection
fn cr_intersection(circle_centre: DVec2, circle_radius: f64, rect: Rect<f64>) -> bool {
    let closest_x = rect.left.max(circle_centre.x.min(rect.left + rect.width));
    let closest_y = rect.top.max(circle_centre.y.min(rect.top + rect.height));

    let dx = circle_centre.x - closest_x;
    let dy = circle_centre.y - closest_y;

    let dist_sq = dx * dx + dy * dy;
    dist_sq < (circle_radius * circle_radius)
}

/// Checks circle-circle intersection
fn cc_intersection(c1: DVec2, r1: f64, c2: DVec2, r2: f64) -> bool {
    (c1 - c2).length_squared() <= (r1 + r2).powi(2)
}

//    closest_x = max(rect_x, min(circle_x, rect_x + width))
//    closest_y = max(rect_y, min(circle_y, rect_y + height))

//    # Calculate the squared distance from the circle's center to this closest point
//    distance_squared = (circle_x - closest_x) ** 2 + (circle_y - closest_y) ** 2

//    # Check if the distance is less than or equal to the circle's radius squared
//    return distance_squared <= radius ** 2
