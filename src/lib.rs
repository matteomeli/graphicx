#[macro_use]
extern crate bitflags;
extern crate winit;

pub mod dx12;
pub mod window;

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

const MS_PER_UPDATE: f64 = 0.01;
const MAX_UPDATE_CYCLES: u8 = 5;

pub struct GameLoop {
    frame_counter: u64,
    previous_frame_time: std::time::Instant,
    elapsed_time_secs: f64,
    lag_time_secs: f64,
    frame_interpolation: f64,
}

impl GameLoop {
    pub fn new() -> GameLoop {
        let now = std::time::Instant::now();

        GameLoop {
            frame_counter: 0,
            previous_frame_time: now,
            elapsed_time_secs: 0.0,
            lag_time_secs: 0.0,
            frame_interpolation: 1.0,
        }
    }

    pub fn frame(&mut self) {
        self.frame_counter += 1;
        let current_frame_time = std::time::Instant::now();
        let delta_time = current_frame_time - self.previous_frame_time;
        let delta_time_secs =
            delta_time.as_secs() as f64 + f64::from(delta_time.subsec_nanos()) * 1e-9;
        self.previous_frame_time = current_frame_time;
        self.lag_time_secs += delta_time_secs;
        self.elapsed_time_secs += delta_time_secs;

        // Show fps
        if cfg!(debug_assertions) && self.elapsed_time_secs > 1.0 {
            let fps = self.frame_counter as f64 / self.elapsed_time_secs;
            println!("FPS: {}", fps);

            self.frame_counter = 0;
            self.elapsed_time_secs = 0.0;
        }

        self.frame_interpolation = self.lag_time_secs / MS_PER_UPDATE;
    }

    pub fn update(&mut self) {
        let mut update_cyles_count = 0;
        while self.lag_time_secs >= MS_PER_UPDATE && update_cyles_count < MAX_UPDATE_CYCLES {
            // Do update here!

            self.lag_time_secs -= MS_PER_UPDATE;
            update_cyles_count += 1
        }
    }
}

impl Default for GameLoop {
    fn default() -> Self {
        Self::new()
    }
}
