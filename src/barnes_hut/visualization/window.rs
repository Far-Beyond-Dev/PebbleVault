use glutin_window::GlutinWindow;
use opengl_graphics::OpenGL;
use piston::window::WindowSettings;

pub fn create_window(width: u32, height: u32) -> GlutinWindow {
    let opengl = OpenGL::V3_2;
    WindowSettings::new("Barnes-Hut Simulation", [width, height])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap()
}