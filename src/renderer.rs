//! Used to pre-render circle images for faster rendering

use sfml::graphics::*;

pub fn circle(radius: u32, color: Color) -> sfml::cpp::FBox<Image> {
    let mut render_target = RenderTexture::new(radius * 2, radius * 2).unwrap();

    render_target.clear(Color::TRANSPARENT);

    let mut circle = CircleShape::new(radius as _, 20_000);
    circle.set_fill_color(color);
    circle.set_position((0., 0.));

    render_target.set_smooth(true);
    render_target.draw(&circle);
    render_target.display();

    let texture = render_target.texture();
    texture.copy_to_image().unwrap()
}
