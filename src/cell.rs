//! Video cell: wraps a single libmpv instance with an OpenGL FBO for rendering.

use std::ffi::CString;
use std::ptr;

use crate::ffi::*;

/// A single video cell containing an mpv instance and its render context.
pub struct VideoCell {
    pub handle: *mut MpvHandle,
    pub render_ctx: *mut MpvRenderContext,
    /// OpenGL FBO ID for this cell's video texture
    pub fbo: u32,
    /// Texture attached to the FBO
    pub texture: u32,
    pub file_path: String,
    pub is_playing: bool,
    pub speed: f64,
    /// Cell position/size in pixels (set by grid layout)
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

unsafe impl Send for VideoCell {}

impl VideoCell {
    /// Create a new video cell with the given file path.
    /// `get_proc_address` is the OpenGL function loader callback.
    pub fn new(
        file_path: &str,
        get_proc_address: extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_char) -> *mut std::ffi::c_void,
        cell_w: i32,
        cell_h: i32,
    ) -> Result<Self, String> {
        unsafe {
            // ── Create mpv handle ────────────────────────
            let handle = mpv_create();
            if handle.is_null() {
                return Err("mpv_create failed".into());
            }

            // Enable hardware decoding
            let hwdec = CString::new("auto-safe").unwrap();
            mpv_set_option_string(handle, b"hwdec\0".as_ptr() as *const _, hwdec.as_ptr());

            // Disable audio output (we don't play audio in grid view)
            let ao = CString::new("no").unwrap();
            mpv_set_option_string(handle, b"audio-display\0".as_ptr() as *const _, ao.as_ptr());
            let ao_null = CString::new("null").unwrap();
            mpv_set_option_string(handle, b"ao\0".as_ptr() as *const _, ao_null.as_ptr());

            // Minimize latency
            let profile = CString::new("low-latency").unwrap();
            mpv_set_option_string(handle, b"profile\0".as_ptr() as *const _, profile.as_ptr());

            // Let us control the render loop
            let yes = CString::new("yes").unwrap();
            mpv_set_option_string(handle, b"idle\0".as_ptr() as *const _, yes.as_ptr());
            mpv_set_option_string(
                handle,
                b"terminal\0".as_ptr() as *const _,
                CString::new("no").unwrap().as_ptr(),
            );

            // Initialize
            let ret = mpv_initialize(handle);
            if ret < 0 {
                mpv_destroy(handle);
                return Err(format!("mpv_initialize failed: {ret}"));
            }

            // ── Create render context ────────────────────
            let mut params: Vec<MpvRenderParam> = Vec::new();

            // API type
            let api = CString::new("opengl").unwrap();
            params.push(MpvRenderParam {
                type_: MPV_RENDER_PARAM_API_TYPE,
                data: api.as_ptr() as *mut _,
            });

            // OpenGL init params
            let gl_params = MpvOpenGLInitParams {
                get_proc_address,
                get_proc_address_ctx: ptr::null_mut(),
            };
            params.push(MpvRenderParam {
                type_: MPV_RENDER_PARAM_OPENGL_INIT_PARAMS,
                data: &gl_params as *const _ as *mut _,
            });

            // Advanced control (we own the GL context)
            params.push(MpvRenderParam {
                type_: MPV_RENDER_PARAM_ADVANCED_CONTROL,
                data: &1i32 as *const _ as *mut _,
            });

            let mut render_ctx: *mut MpvRenderContext = ptr::null_mut();
            let ret = mpv_render_context_create(
                &mut render_ctx,
                handle,
                params.as_mut_ptr(),
            );
            if ret < 0 {
                mpv_destroy(handle);
                return Err(format!("mpv_render_context_create failed: {ret}"));
            }

            // ── Create FBO and texture ──────────────────
            let (fbo, texture) = create_fbo(cell_w, cell_h);

            // ── Load the file ────────────────────────────
            let cmd = CString::new(format!("loadfile \"{file_path}\"")).unwrap();
            mpv_command_string(handle, cmd.as_ptr());

            Ok(Self {
                handle,
                render_ctx,
                fbo,
                texture,
                file_path: file_path.to_string(),
                is_playing: true,
                speed: 1.0,
                x: 0,
                y: 0,
                w: cell_w,
                h: cell_h,
            })
        }
    }

    /// Render this cell's video frame to its FBO.
    pub fn render(&self) -> bool {
        unsafe {
            // Update render context (swap frame if new one available)
            let flags = mpv_render_context_update(self.render_ctx);
            let should_render = flags & 1 != 0;

            if should_render {
                let fbo_params = MpvOpenGLFbo {
                    fbo: self.fbo as i32,
                    w: self.w,
                    h: self.h,
                    internal_format: 0, // auto
                };

                let flip_y = 1i32;

                let params = [
                    MpvRenderParam {
                        type_: MPV_RENDER_PARAM_OPENGL_FBO,
                        data: &fbo_params as *const _ as *mut _,
                    },
                    MpvRenderParam {
                        type_: MPV_RENDER_PARAM_FLIP_Y,
                        data: &flip_y as *const _ as *mut _,
                    },
                    MpvRenderParam {
                        type_: MPV_RENDER_PARAM_SKIP_RENDERING,
                        data: ptr::null_mut(),
                    },
                ];

                let ret = mpv_render_context_render(
                    self.render_ctx,
                    params.as_ptr() as *mut _,
                );
                if ret >= 0 {
                    mpv_render_context_report_swap(self.render_ctx);
                }
            }

            should_render
        }
    }

    /// Toggle play/pause
    pub fn toggle_pause(&mut self) {
        unsafe {
            let pause = if self.is_playing { "yes" } else { "no" };
            let c = CString::new(format!("set pause {pause}")).unwrap();
            mpv_command_string(self.handle, c.as_ptr());
            self.is_playing = !self.is_playing;
        }
    }

    /// Seek by `seconds` relative to current position
    pub fn seek(&self, seconds: f64) {
        unsafe {
            let c = CString::new(format!("seek {seconds} relative")).unwrap();
            mpv_command_string(self.handle, c.as_ptr());
        }
    }

    /// Set playback speed
    pub fn set_speed(&mut self, speed: f64) {
        self.speed = speed.clamp(0.1, 16.0);
        unsafe {
            let c = CString::new(format!("set speed {}", self.speed)).unwrap();
            mpv_command_string(self.handle, c.as_ptr());
        }
    }

    /// Adjust speed by delta
    pub fn adjust_speed(&mut self, delta: f64) {
        self.set_speed(self.speed + delta);
    }

    /// Get current playback time in seconds
    pub fn get_time(&self) -> f64 {
        unsafe {
            let mut time: f64 = 0.0;
            let ret = mpv_get_property(
                self.handle,
                b"time-pos\0".as_ptr() as *const _,
                MPV_FORMAT_DOUBLE,
                &mut time as *mut _ as *mut _,
            );
            if ret >= 0 { time } else { 0.0 }
        }
    }

    /// Get file duration in seconds
    pub fn get_duration(&self) -> f64 {
        unsafe {
            let mut dur: f64 = 0.0;
            let ret = mpv_get_property(
                self.handle,
                b"duration\0".as_ptr() as *const _,
                MPV_FORMAT_DOUBLE,
                &mut dur as *mut _ as *mut _,
            );
            if ret >= 0 { dur } else { 0.0 }
        }
    }

    /// Get the file name component
    pub fn filename(&self) -> &str {
        std::path::Path::new(&self.file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&self.file_path)
    }

    /// Resize the cell (recreate FBO)
    pub fn resize(&mut self, w: i32, h: i32) {
        unsafe {
            // Delete old FBO/texture
            gl::DeleteFramebuffers(1, &self.fbo);
            gl::DeleteTextures(1, &self.texture);
        }
        self.w = w;
        self.h = h;
        let (fbo, tex) = create_fbo(w, h);
        self.fbo = fbo;
        self.texture = tex;
    }
}

impl Drop for VideoCell {
    fn drop(&mut self) {
        unsafe {
            if !self.render_ctx.is_null() {
                mpv_render_context_free(self.render_ctx);
            }
            if !self.handle.is_null() {
                mpv_terminate_destroy(self.handle);
            }
            if self.fbo != 0 {
                gl::DeleteFramebuffers(1, &self.fbo);
            }
            if self.texture != 0 {
                gl::DeleteTextures(1, &self.texture);
            }
        }
    }
}

/// Create an OpenGL FBO with an attached RGBA texture.
fn create_fbo(w: i32, h: i32) -> (u32, u32) {
    unsafe {
        let mut fbo = 0u32;
        let mut tex = 0u32;

        gl::GenFramebuffers(1, &mut fbo);
        gl::GenTextures(1, &mut tex);

        gl::BindTexture(gl::TEXTURE_2D, tex);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            w,
            h,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            ptr::null(),
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

        gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
        gl::FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            tex,
            0,
        );

        let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
        if status != gl::FRAMEBUFFER_COMPLETE {
            eprintln!("FBO incomplete: {status}");
        }

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        gl::BindTexture(gl::TEXTURE_2D, 0);

        (fbo, tex)
    }
}
