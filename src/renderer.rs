//! Used to pre-render circle images for faster rendering

use sfml::graphics::*;

pub fn circle(radius: u32, color: Color, output_directory: impl AsRef<std::path::Path>) -> sfml::cpp::FBox<Image> {
    let path = output_directory.as_ref();
    assert!(path.is_dir(), "should be a directory");

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

// pub fn load_images(
//     dir: impl AsRef<std::path::Path>,
// ) -> HashMap<(i32, (u8, u8, u8, u8)), sfml::cpp::FBox<Texture>> {
//     let dir = dir.as_ref();

//     HashMap::from_iter(
//         std::fs::read_dir(dir)
//             .unwrap()
//             .filter_map(|i| i.ok())
//             .filter(|i| i.metadata().unwrap().is_file())
//             .map(|file| {
//                 let binding = file.path();

//                 let name = binding.file_name().unwrap().to_str().unwrap();

//                 let split = name.split(['r', '#', '.']).collect::<Vec<_>>();
//                 let radius: i32 = split[1].parse().unwrap();
//                 let color_hex = split[2];

//                 let &[r, g, b, a] = (0..8)
//                     .step_by(2)
//                     .map(|i| u8::from_str_radix(&color_hex[i..i + 2], 16).unwrap())
//                     .collect::<Vec<_>>()
//                     .as_slice()
//                 else {
//                     panic!()
//                 };

//                 let binding = file.path();
//                 let mut texture = Texture::new().unwrap();

//                 texture
//                     .load_from_file(
//                         binding.to_str().unwrap(),
//                         Rect::new(0, 0, radius * 2, radius * 2),
//                     )
//                     .unwrap();

//                 (
//                     (radius - RADIUS_INCREMENT as i32, (r, g, b, a)),
//                     texture,
//                 )
//             }),
//     )
// }
