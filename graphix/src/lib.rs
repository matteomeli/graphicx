extern crate winit;

use std::fmt;

#[derive(Debug)]
pub struct Config {
    pub width: u32,
    pub height: u32,
    pub use_warp: bool,
    pub is_vsync_enabled: bool,
    pub is_fullscreen: bool,
}

impl Config {
    pub fn new(args: &[String]) -> Config {
        let mut width = 1280;
        let mut height = 720;
        let mut use_warp = false;
        let mut is_vsync_enabled = false;
        let mut is_fullscreen = false;

        for i in 0..args.len() {
            let arg = &args[i];
            match arg.as_ref() {
                "-w" | "--width" => width = args[i + 1].parse::<u32>().unwrap(),
                "-h" | "--height" => height = args[i + 1].parse::<u32>().unwrap(),
                "-warp" | "--warp" => use_warp = true,
                "-v" | "--vsync" => is_vsync_enabled = true,
                "-f" | "--fullscreen" => is_fullscreen = true,
                _ => (),
            }
        }

        Config {
            width,
            height,
            use_warp,
            is_vsync_enabled,
            is_fullscreen,
        }
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "({} {} {} {} {})",
            self.width, self.height, self.use_warp, self.is_vsync_enabled, self.is_fullscreen
        )
    }
}

pub fn set_fullscreen(window: &winit::Window, is_fullscreen: bool) {
    // Maximise window into full screen borderless window (FSBW) rather than real fullsceen
    if is_fullscreen {
        // Turn off decorations
        window.set_decorations(false);
        // Make sure window is on top
        window.set_always_on_top(true);
        // Maximize window
        window.set_maximized(true);
    } else {
        // Turn off decorations
        window.set_decorations(true);
        // Make sure window is on top
        window.set_always_on_top(false);
        // Maximize window
        window.set_maximized(false);
    }
}
