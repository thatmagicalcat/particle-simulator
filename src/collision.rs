use glam::DVec2;

/// Returns vf1 and vf2 respectively
pub fn process_collision(
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
