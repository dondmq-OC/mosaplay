//! GridPlayer — Multi-video grid player powered by libmpv + OpenGL + SDL2.
//!
//! Play 1–16+ videos simultaneously in a configurable grid layout,
//! with async per-video control (play/pause/seek/speed) via keyboard.

mod cell;
mod ffi;
mod grid;
mod render;

use std::ffi::{c_char, c_void, CStr};
use std::time::Instant;

use cell::VideoCell;
use grid::{calculate_grid, cell_position};
use render::RenderState;

// ── OpenGL proc-address callback for libmpv ─────────────────
use std::sync::Mutex;
type GlLoaderFn = Box<dyn Fn(&str) -> *mut c_void + Send + 'static>;
static GL_LOADER: Mutex<Option<GlLoaderFn>> = Mutex::new(None);

extern "C" fn get_proc_address(_ctx: *mut c_void, name: *const c_char) -> *mut c_void {
    unsafe {
        let name = CStr::from_ptr(name).to_str().unwrap_or("");
        if let Some(f) = GL_LOADER.lock().unwrap().as_ref() {
            f(name)
        } else {
            std::ptr::null_mut()
        }
    }
}

// ── Application state ───────────────────────────────────────
struct App {
    cells: Vec<VideoCell>,
    focused: usize,
    grid_layout: grid::GridLayout,
    margin: u32,
    running: bool,
    last_title_update: Instant,
}

impl App {
    fn update_layout(&mut self, screen_w: u32, screen_h: u32) {
        self.grid_layout = calculate_grid(self.cells.len() as u32, screen_w, screen_h);
        let m = self.margin;

        for (i, cell) in self.cells.iter_mut().enumerate() {
            let (x, y) = cell_position(i as u32, &self.grid_layout, m);
            let w = self.grid_layout.cell_w as i32 - 2 * m as i32;
            let h = self.grid_layout.cell_h as i32 - 2 * m as i32;
            if w != cell.w || h != cell.h {
                cell.resize(w.max(64), h.max(64));
            }
            cell.x = x;
            cell.y = y;
        }
    }

    fn build_title(&self) -> String {
        if self.cells.is_empty() {
            return "GridPlayer".into();
        }
        let cell = &self.cells[self.focused % self.cells.len()];
        let status = if cell.is_playing { "▶" } else { "⏸" };
        let time = cell.get_time();
        let dur = cell.get_duration();
        format!(
            "GridPlayer [{}/{}] {} {} | {:.0}s/{:.0}s | {:.2}x",
            self.focused + 1,
            self.cells.len(),
            status,
            cell.filename(),
            time,
            dur,
            cell.speed
        )
    }
}

// ── Entry point ─────────────────────────────────────────────
fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Parse command-line arguments: video files to load
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args = if args.is_empty() {
        // No args — try to open file dialog
        show_usage_info();
        match open_file_dialog() {
            Some(files) => files,
            None => {
                eprintln!("No files selected. Exiting.");
                std::process::exit(0);
            }
        }
    } else {
        args
    };

    if args.is_empty() {
        std::process::exit(0);
    }

    // ── Init SDL2 ───────────────────────────────────────────
    let sdl = sdl2::init().map_err(|e| format!("SDL init: {e}"))?;
    let video_subsystem = sdl.video().map_err(|e| format!("SDL video: {e}"))?;

    // Request OpenGL 3.3 core
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);
    gl_attr.set_double_buffer(true);

    // Determine initial window size
    let display_bounds = video_subsystem
        .display_bounds(0)
        .unwrap_or(sdl2::rect::Rect::new(0, 0, 1920, 1080));
    let initial_w = (display_bounds.w as f64 * 0.85) as u32;
    let initial_h = (display_bounds.h as f64 * 0.85) as u32;

    let mut window = video_subsystem
        .window("GridPlayer", initial_w, initial_h)
        .opengl()
        .resizable()
        .build()
        .map_err(|e| format!("Window: {e}"))?;

    let gl_context = window
        .gl_create_context()
        .map_err(|e| format!("GL context: {e}"))?;
    window
        .gl_make_current(&gl_context)
        .map_err(|e| format!("GL make current: {e}"))?;

    // Set swap interval to 1 (VSync)
    video_subsystem.gl_set_swap_interval(1).ok();

    // Load OpenGL functions
    gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const _);

    // Store GL loader for mpv callbacks (use raw SDL2 FFI to avoid Send issues)
    {
        let mut loader = GL_LOADER.lock().unwrap();
        if loader.is_none() {
            *loader = Some(Box::new(|name: &str| -> *mut c_void {
                let cname = std::ffi::CString::new(name).unwrap();
                unsafe {
                    sdl2::sys::SDL_GL_GetProcAddress(cname.as_ptr()) as *mut c_void
                }
            }));
        }
    }

    unsafe {
        println!("OpenGL: {}", gl_get_string(gl::VENDOR));
        println!("GPU: {}", gl_get_string(gl::RENDERER));
        println!("GL version: {}", gl_get_string(gl::VERSION));
    }

    // ── Initialize render state ─────────────────────────────
    let render_state = RenderState::new().map_err(|e| format!("Shader: {e}"))?;

    // ── Calculate initial grid ──────────────────────────────
    let grid_layout = calculate_grid(args.len() as u32, initial_w, initial_h);

    // ── Create video cells ──────────────────────────────────
    let mut cells: Vec<VideoCell> = Vec::new();
    let m = 4u32; // margin in pixels
    let (cw, ch) = (grid_layout.cell_w, grid_layout.cell_h);

    for (i, path) in args.iter().enumerate() {
        let (x, y) = cell_position(i as u32, &grid_layout, m);
        let w = cw as i32 - 2 * m as i32;
        let h = ch as i32 - 2 * m as i32;

        println!("Loading [{}/{}]: {path}", i + 1, args.len());

        match VideoCell::new(path, get_proc_address, w.max(64), h.max(64)) {
            Ok(mut cell) => {
                cell.x = x;
                cell.y = y;
                cells.push(cell);
            }
            Err(e) => {
                eprintln!("  Failed to load '{path}': {e}");
            }
        }
    }

    if cells.is_empty() {
        eprintln!("No videos could be loaded.");
        std::process::exit(1);
    }

    println!("Loaded {}/{} videos.", cells.len(), args.len());

    let mut app = App {
        cells,
        focused: 0,
        grid_layout,
        margin: m,
        running: true,
        last_title_update: Instant::now(),
    };

    let mut screen_w = initial_w as i32;
    let mut screen_h = initial_h as i32;

    let mut event_pump = sdl.event_pump().map_err(|e| format!("Event pump: {e}"))?;

    window.set_title(&app.build_title()).ok();

    let mut fullscreen_toggle = false;

    // ── Main loop ───────────────────────────────────────────
    while app.running {
        // Handle SDL events
        for event in event_pump.poll_iter() {
            use sdl2::event::Event;
            use sdl2::keyboard::Keycode;

            match event {
                Event::Quit { .. } => {
                    app.running = false;
                }

                Event::Window {
                    win_event: sdl2::event::WindowEvent::Resized(w, h),
                    ..
                } => {
                    if w > 0 && h > 0 {
                        screen_w = w;
                        screen_h = h;
                        unsafe {
                            gl::Viewport(0, 0, w, h);
                        }
                        app.update_layout(w as u32, h as u32);
                    }
                }

                // Mouse hover → auto-focus cell under cursor
                Event::MouseMotion { x, y, .. } => {
                    for (i, cell) in app.cells.iter().enumerate() {
                        if x >= cell.x && x < cell.x + cell.w
                            && y >= cell.y && y < cell.y + cell.h
                        {
                            app.focused = i;
                            break;
                        }
                    }
                }

                Event::KeyDown {
                    keycode: Some(key),
                    keymod,
                    ..
                } => {
                    let ctrl = keymod.contains(sdl2::keyboard::Mod::LCTRLMOD)
                        || keymod.contains(sdl2::keyboard::Mod::RCTRLMOD);
                    let cmd = keymod.contains(sdl2::keyboard::Mod::LGUIMOD)
                        || keymod.contains(sdl2::keyboard::Mod::RGUIMOD);
                    let shift = keymod.contains(sdl2::keyboard::Mod::LSHIFTMOD)
                        || keymod.contains(sdl2::keyboard::Mod::RSHIFTMOD);

                    match key {
                        Keycode::Escape => app.running = false,
                        Keycode::Q if cmd => app.running = false,

                        // Focus navigation
                        Keycode::Tab => {
                            if shift {
                                if app.focused == 0 {
                                    app.focused = app.cells.len() - 1;
                                } else {
                                    app.focused -= 1;
                                }
                            } else {
                                app.focused = (app.focused + 1) % app.cells.len();
                            }
                        }

                        // Play/Pause focused cell
                        Keycode::Space => {
                            if !app.cells.is_empty() {
                                app.cells[app.focused].toggle_pause();
                            }
                        }

                        // Seek
                        Keycode::Left => {
                            if !app.cells.is_empty() {
                                let delta = if ctrl { -30.0 } else { -5.0 };
                                app.cells[app.focused].seek(delta);
                            }
                        }
                        Keycode::Right => {
                            if !app.cells.is_empty() {
                                let delta = if ctrl { 30.0 } else { 5.0 };
                                app.cells[app.focused].seek(delta);
                            }
                        }

                        // Volume control
                        Keycode::LeftBracket => {
                            if !app.cells.is_empty() {
                                app.cells[app.focused].adjust_volume(-5.0);
                            }
                        }
                        Keycode::RightBracket => {
                            if !app.cells.is_empty() {
                                app.cells[app.focused].adjust_volume(5.0);
                            }
                        }
                        Keycode::M => {
                            if !app.cells.is_empty() {
                                let cell = &app.cells[app.focused];
                                if cell.volume() > 0.0 {
                                    cell.set_volume(0.0);
                                } else {
                                    cell.set_volume(50.0);
                                }
                            }
                        }

                        // Speed control
                        Keycode::Up => {
                            if !app.cells.is_empty() {
                                app.cells[app.focused].adjust_speed(0.25);
                            }
                        }
                        Keycode::Down => {
                            if !app.cells.is_empty() {
                                app.cells[app.focused].adjust_speed(-0.25);
                            }
                        }

                        // Direct cell selection (1-9, 0 for 10th)
                        Keycode::Num1 | Keycode::Kp1 => {
                            app.focused = 0.min(app.cells.len().saturating_sub(1));
                        }
                        Keycode::Num2 | Keycode::Kp2 => {
                            app.focused = 1.min(app.cells.len().saturating_sub(1));
                        }
                        Keycode::Num3 | Keycode::Kp3 => {
                            app.focused = 2.min(app.cells.len().saturating_sub(1));
                        }
                        Keycode::Num4 | Keycode::Kp4 => {
                            app.focused = 3.min(app.cells.len().saturating_sub(1));
                        }
                        Keycode::Num5 | Keycode::Kp5 => {
                            app.focused = 4.min(app.cells.len().saturating_sub(1));
                        }
                        Keycode::Num6 | Keycode::Kp6 => {
                            app.focused = 5.min(app.cells.len().saturating_sub(1));
                        }
                        Keycode::Num7 | Keycode::Kp7 => {
                            app.focused = 6.min(app.cells.len().saturating_sub(1));
                        }
                        Keycode::Num8 | Keycode::Kp8 => {
                            app.focused = 7.min(app.cells.len().saturating_sub(1));
                        }
                        Keycode::Num9 | Keycode::Kp9 => {
                            app.focused = 8.min(app.cells.len().saturating_sub(1));
                        }
                        Keycode::Num0 | Keycode::Kp0 => {
                            app.focused = 9.min(app.cells.len().saturating_sub(1));
                        }

                        // Speed presets
                        Keycode::Minus => {
                            if !app.cells.is_empty() {
                                app.cells[app.focused].adjust_speed(-0.1);
                            }
                        }
                        Keycode::Equals | Keycode::Plus | Keycode::KpPlus => {
                            if !app.cells.is_empty() {
                                app.cells[app.focused].adjust_speed(0.1);
                            }
                        }

                        // Reset speed
                        Keycode::Backspace => {
                            if !app.cells.is_empty() {
                                app.cells[app.focused].set_speed(1.0);
                            }
                        }

                        // Pause all / Play all
                        Keycode::P => {
                            for cell in &mut app.cells {
                                if cell.is_playing {
                                    cell.toggle_pause();
                                }
                            }
                        }
                        Keycode::A => {
                            for cell in &mut app.cells {
                                if !cell.is_playing {
                                    cell.toggle_pause();
                                }
                            }
                        }

                        // Fullscreen toggle
                        Keycode::F => {
                            fullscreen_toggle = true;
                        }

                        // Cycle grid layouts
                        Keycode::G => {
                            let n = app.cells.len() as u32;
                            let layouts: &[(u32, u32)] =
                                &[(0, 0), (3, 3), (4, 3), (4, 4), (2, 2)];
                            let current = (app.grid_layout.cols, app.grid_layout.rows);
                            let mut next_layout = None;
                            for (i, l) in layouts.iter().enumerate() {
                                if *l == current {
                                    let idx = (i + 1) % layouts.len();
                                    next_layout = Some(layouts[idx]);
                                    break;
                                }
                            }
                            if let Some(next) = next_layout {
                                if next.0 == 0 {
                                    app.grid_layout = calculate_grid(
                                        n,
                                        screen_w as u32,
                                        screen_h as u32,
                                    );
                                } else {
                                    let (cols, rows) = next;
                                    let cw = screen_w as u32 / cols;
                                    let ch = screen_h as u32 / rows;
                                    app.grid_layout = grid::GridLayout {
                                        cols,
                                        rows,
                                        cell_w: cw,
                                        cell_h: ch,
                                    };
                                }
                                app.update_layout(screen_w as u32, screen_h as u32);
                            }
                        }

                        _ => {}
                    }
                }

                _ => {}
            }
        }

        // ── Handle deferred mutable window operations ───────
        if fullscreen_toggle {
            let state = window.fullscreen_state();
            if state == sdl2::video::FullscreenType::Off {
                window
                    .set_fullscreen(sdl2::video::FullscreenType::Desktop)
                    .ok();
            } else {
                window.set_fullscreen(sdl2::video::FullscreenType::Off).ok();
            }
            fullscreen_toggle = false;
        }

        // ── Render ──────────────────────────────────────────
        for cell in &app.cells {
            cell.render();
        }

        render_state.render_grid(
            &app.cells,
            app.focused,
            screen_w,
            screen_h,
            app.margin as i32,
        );

        window.gl_swap_window();

        // Update title periodically
        if app.last_title_update.elapsed().as_secs_f64() > 0.5 {
            window.set_title(&app.build_title()).ok();
            app.last_title_update = Instant::now();
        }

        // Throttle: avoid busy-waiting
        std::thread::sleep(std::time::Duration::from_millis(4));
    }

    // ── Cleanup ─────────────────────────────────────────────
    println!("Shutting down...");
    drop(app.cells);
    drop(render_state);
    drop(gl_context);
    println!("Done.");

    Ok(())
}

fn show_usage_info() {
    eprintln!("GridPlayer v0.1.0 — Multi-video grid player");
    eprintln!("Usage: gridplayer <video1> [video2] ...");
    eprintln!("Keys: Space=play/pause  ←→=seek  ↑↓=speed  Tab=focus");
    eprintln!("      1-9=select  F=fullscreen  G=grid  Esc=quit");
}

fn open_file_dialog() -> Option<Vec<String>> {
    rfd::FileDialog::new()
        .set_title("选择要播放的视频文件")
        .add_filter("视频文件", &["mp4", "mkv", "avi", "mov", "webm", "wmv", "flv", "m4v", "ts", "m2ts"])
        .add_filter("所有文件", &["*"])
        .pick_files()
        .map(|paths| paths.into_iter().filter_map(|p| p.to_str().map(|s| s.to_string())).collect())
}

unsafe fn gl_get_string(name: u32) -> String {
    let ptr = gl::GetString(name);
    if ptr.is_null() {
        return "unknown".into();
    }
    let cstr = CStr::from_ptr(ptr as *const _);
    cstr.to_string_lossy().into_owned()
}
