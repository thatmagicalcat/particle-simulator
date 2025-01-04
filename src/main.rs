mod engine;

mod collision;
mod components;
mod quadtree;
mod systems;

mod renderer;

/// space wasted by window decorations (approximate value)
const WINDOW_PADDING: u32 = 0;

const GRAVITY: f64 = 10.0;
const WINDOW_HEIGHT: u32 = 900;
const WINDOW_WIDHT: u32 = 1600;

fn main() {
    engine::run();
}
