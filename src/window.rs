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
