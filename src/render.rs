//! OpenGL renderer: compositing video textures into a grid layout.
//! Uses glViewport per cell — no MVP matrix needed.

use crate::cell::VideoCell;

/// Simple vertex shader: unit quad, no transform
const VERTEX_SHADER: &str = r#"
#version 330 core
layout(location = 0) in vec2 aPos;
layout(location = 1) in vec2 aTexCoord;
out vec2 vTexCoord;
void main() {
    gl_Position = vec4(aPos, 0.0, 1.0);
    vTexCoord = aTexCoord;
}
"#;

/// Fragment shader: sample video texture
const FRAGMENT_SHADER: &str = r#"
#version 330 core
in vec2 vTexCoord;
out vec4 FragColor;
uniform sampler2D uTexture;
void main() {
    FragColor = texture(uTexture, vTexCoord);
}
"#;

/// Solid-color fragment shader for focus border
const SOLID_FRAGMENT: &str = r#"
#version 330 core
out vec4 FragColor;
uniform vec4 uColor;
void main() {
    FragColor = uColor;
}
"#;

pub struct RenderState {
    pub program: u32,
    pub solid_program: u32,
    vao: u32,
    vbo: u32,
    ebo: u32,
    /// Play-button triangle VAO (3 vertices, for welcome screen)
    logo_vao: u32,
    logo_vbo: u32,
}

impl RenderState {
    pub fn new() -> Result<Self, String> {
        unsafe {
            let program = create_shader_program(VERTEX_SHADER, FRAGMENT_SHADER)?;
            let solid_program = create_shader_program(VERTEX_SHADER, SOLID_FRAGMENT)?;

            // Unit quad: fills [-1,1] → maps to whatever viewport is set
            #[rustfmt::skip]
            let vertices: [f32; 16] = [
                -1.0, -1.0,  0.0, 0.0,
                 1.0, -1.0,  1.0, 0.0,
                 1.0,  1.0,  1.0, 1.0,
                -1.0,  1.0,  0.0, 1.0,
            ];
            let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];

            let mut vao = 0; let mut vbo = 0; let mut ebo = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);
            gl::BindVertexArray(vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, (vertices.len() * std::mem::size_of::<f32>()) as isize, vertices.as_ptr() as *const _, gl::STATIC_DRAW);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (indices.len() * std::mem::size_of::<u32>()) as isize, indices.as_ptr() as *const _, gl::STATIC_DRAW);

            let stride = (4 * std::mem::size_of::<f32>()) as i32;
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, std::ptr::null());
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (2 * std::mem::size_of::<f32>()) as *const _);
            gl::EnableVertexAttribArray(1);
            gl::BindVertexArray(0);

            // Play-button triangle VAO (right-pointing ▶ shape)
            // Position(2) + texcoord(2), same stride as quad VAO
            #[rustfmt::skip]
            let tri: [f32; 12] = [
                -0.45,  0.0,   0.0, 0.0,  // left point
                 0.55,  0.50,  0.0, 0.0,  // top-right
                 0.55, -0.50,  0.0, 0.0,  // bottom-right
            ];
            let mut logo_vao = 0; let mut logo_vbo = 0;
            gl::GenVertexArrays(1, &mut logo_vao);
            gl::GenBuffers(1, &mut logo_vbo);
            gl::BindVertexArray(logo_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, logo_vbo);
            gl::BufferData(gl::ARRAY_BUFFER, (tri.len() * std::mem::size_of::<f32>()) as isize, tri.as_ptr() as *const _, gl::STATIC_DRAW);
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, std::ptr::null());
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (2 * std::mem::size_of::<f32>()) as *const _);
            gl::EnableVertexAttribArray(1);
            gl::BindVertexArray(0);

            Ok(Self { program, solid_program, vao, vbo, ebo, logo_vao, logo_vbo })
        }
    }

    pub fn render_grid(
        &self, cells: &[VideoCell], focused_idx: usize, drag_from: Option<usize>,
        screen_w: i32, screen_h: i32,
    ) {
        // ── Welcome screen when empty ──────────────────
        if cells.is_empty() {
            // Rasterize message text to a texture
            let msg = "Drop files or press Cmd+O to open";
            let cx = screen_w / 2;
            let msg_y = screen_h * 2 / 5;
            let char_w = 8i32;
            let char_h = 8i32;
            let spacing = 2i32;
            let total_w = (msg.len() as i32) * (char_w + spacing) - spacing;
            let total_h = char_h;

            // Allocate RGBA buffer
            let buf_w = total_w.max(1);
            let buf_h = total_h.max(1);
            let mut pixels = vec![0u8; (buf_w * buf_h * 4) as usize];

            // Rasterize using font8x8
            for (ci, c) in msg.chars().enumerate() {
                let idx = c as usize;
                if idx < 128 {
                    let glyph = font8x8::legacy::BASIC_LEGACY[idx]; // 8 rows, each = 1 byte
                    let x0 = ci as i32 * (char_w + spacing);
                    for row in 0..8 {
                        let byte = glyph[row];
                        for col in 0..8 {
                            if (byte >> (7 - col)) & 1 != 0 {
                                let px = (x0 + col as i32) as usize;
                                let py = (7 - row) as usize; // flip Y for OpenGL
                                if px < buf_w as usize && py < buf_h as usize {
                                    let idx2 = (py * buf_w as usize + px) * 4;
                                    pixels[idx2] = 220;     // R
                                    pixels[idx2 + 1] = 200; // G
                                    pixels[idx2 + 2] = 180; // B
                                    pixels[idx2 + 3] = 230; // A
                                }
                            }
                        }
                    }
                }
            }

            // Upload as OpenGL texture
            unsafe {
                let mut tex = 0u32;
                gl::GenTextures(1, &mut tex);
                gl::BindTexture(gl::TEXTURE_2D, tex);
                gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, buf_w, buf_h, 0,
                    gl::RGBA, gl::UNSIGNED_BYTE, pixels.as_ptr() as *const _);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

                gl::ClearColor(0.06, 0.06, 0.08, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);

                // Render the text texture
                let tw = buf_w;
                let th = buf_h;
                let scale = 3i32; // pixel scale for visibility
                gl::Viewport(cx - tw * scale / 2, msg_y - th * scale / 2, tw * scale, th * scale);
                gl::Scissor(cx - tw * scale / 2, msg_y - th * scale / 2, tw * scale, th * scale);
                gl::Enable(gl::SCISSOR_TEST);
                gl::UseProgram(self.program);
                gl::BindVertexArray(self.vao);
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, tex);
                let tex_loc = gl::GetUniformLocation(self.program, b"uTexture\0".as_ptr() as *const _);
                gl::Uniform1i(tex_loc, 0);
                gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());

                gl::DeleteTextures(1, &mut tex);

                gl::Disable(gl::SCISSOR_TEST);
                gl::BindVertexArray(0);
                gl::UseProgram(0);
                return;
            }
        }

        // Color palette for cell corner markers (10 distinct colors)
        let palette: [(f32, f32, f32); 10] = [
            (0.98, 0.40, 0.30), (0.30, 0.80, 0.40), (0.30, 0.50, 0.98),
            (0.95, 0.80, 0.20), (0.70, 0.30, 0.90), (0.20, 0.85, 0.75),
            (0.95, 0.55, 0.10), (0.50, 0.50, 0.90), (0.90, 0.25, 0.55),
            (0.40, 0.70, 0.30),
        ];

        unsafe {
            gl::ClearColor(0.06, 0.06, 0.08, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            for (i, cell) in cells.iter().enumerate() {
                let gl_y = screen_h - cell.y - cell.h;

                gl::Viewport(cell.x, gl_y, cell.w, cell.h);
                gl::Scissor(cell.x, gl_y, cell.w, cell.h);
                gl::Enable(gl::SCISSOR_TEST);

                // Draw video texture
                gl::UseProgram(self.program);
                gl::BindVertexArray(self.vao);
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, cell.texture);
                let tex_loc = gl::GetUniformLocation(self.program, b"uTexture\0".as_ptr() as *const _);
                gl::Uniform1i(tex_loc, 0);
                gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());

                // Cell corner marker (colored square, top-left)
                gl::UseProgram(self.solid_program);
                let (r, g, b) = palette[i % 10];
                let color_loc = gl::GetUniformLocation(self.solid_program, b"uColor\0".as_ptr() as *const _);
                let sz = (cell.w.min(cell.h) as f32 * 0.08) as i32;
                if sz > 6 {
                    gl::Uniform4f(color_loc, r, g, b, 0.80);
                    gl::Scissor(cell.x, gl_y + cell.h - sz, sz, sz);
                    gl::Viewport(cell.x, gl_y + cell.h - sz, sz, sz);
                    gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
                    // Restore
                    gl::Viewport(cell.x, gl_y, cell.w, cell.h);
                    gl::Scissor(cell.x, gl_y, cell.w, cell.h);
                }

                // Drag-source highlight (expanded glow)
                if drag_from == Some(i) {
                    let dsz = (cell.w.min(cell.h) as f32 * 0.018) as i32;
                    gl::Uniform4f(color_loc, 0.50, 0.50, 1.0, 0.20);
                    gl::Viewport(cell.x - dsz, gl_y - dsz, cell.w + dsz * 2, cell.h + dsz * 2);
                    gl::Scissor(cell.x - dsz, gl_y - dsz, cell.w + dsz * 2, cell.h + dsz * 2);
                    gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
                    gl::Viewport(cell.x, gl_y, cell.w, cell.h);
                    gl::Scissor(cell.x, gl_y, cell.w, cell.h);
                }

                // Focus border: draw 4 thin filled bars around the cell edge
                // Uses quads instead of glLineWidth (not supported >1 on macOS)
                if i == focused_idx {
                    let bw = (cell.w.min(cell.h) as f32 * 0.012) as i32;  // glow width
                    let iw = (bw / 3).max(2);  // inner line width
                    if bw > 3 {
                        // Outer glow — 4 bars
                        // Top bar
                        gl::Viewport(cell.x - bw, gl_y + cell.h, cell.w + bw * 2, bw);
                        gl::Scissor(cell.x - bw, gl_y + cell.h, cell.w + bw * 2, bw);
                        gl::Uniform4f(color_loc, 1.0, 0.55, 0.0, 0.30);
                        gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
                        // Bottom bar
                        gl::Viewport(cell.x - bw, gl_y - bw, cell.w + bw * 2, bw);
                        gl::Scissor(cell.x - bw, gl_y - bw, cell.w + bw * 2, bw);
                        gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
                        // Left bar
                        gl::Viewport(cell.x - bw, gl_y, bw, cell.h);
                        gl::Scissor(cell.x - bw, gl_y, bw, cell.h);
                        gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
                        // Right bar
                        gl::Viewport(cell.x + cell.w, gl_y, bw, cell.h);
                        gl::Scissor(cell.x + cell.w, gl_y, bw, cell.h);
                        gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());

                        // Inner sharp line — same 4 bars, thinner
                        gl::Uniform4f(color_loc, 1.0, 0.55, 0.0, 0.90);
                        gl::Viewport(cell.x - iw, gl_y + cell.h, cell.w + iw * 2, iw);
                        gl::Scissor(cell.x - iw, gl_y + cell.h, cell.w + iw * 2, iw);
                        gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
                        gl::Viewport(cell.x - iw, gl_y - iw, cell.w + iw * 2, iw);
                        gl::Scissor(cell.x - iw, gl_y - iw, cell.w + iw * 2, iw);
                        gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
                        gl::Viewport(cell.x - iw, gl_y, iw, cell.h);
                        gl::Scissor(cell.x - iw, gl_y, iw, cell.h);
                        gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
                        gl::Viewport(cell.x + cell.w, gl_y, iw, cell.h);
                        gl::Scissor(cell.x + cell.w, gl_y, iw, cell.h);
                        gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());

                        // Restore cell viewport
                        gl::Viewport(cell.x, gl_y, cell.w, cell.h);
                        gl::Scissor(cell.x, gl_y, cell.w, cell.h);
                    }
                }
            }

            // ── Status bar at bottom ────────────────────────
            gl::Disable(gl::SCISSOR_TEST);
            gl::Viewport(0, 0, screen_w, screen_h);
            gl::UseProgram(self.solid_program);
            let color_loc = gl::GetUniformLocation(self.solid_program, b"uColor\0".as_ptr() as *const _);
            let bar_h = 24i32;
            let bar_y = screen_h - bar_h;

            // Background bar
            gl::Viewport(0, bar_y, screen_w, bar_h);
            gl::Scissor(0, bar_y, screen_w, bar_h);
            gl::Enable(gl::SCISSOR_TEST);
            gl::Uniform4f(color_loc, 0.10, 0.10, 0.14, 0.85);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());

            // Focused cell info
            if !cells.is_empty() && focused_idx < cells.len() {
                let cell = &cells[focused_idx];
                // File name bar
                let name_w = (cell.filename().len() as i32 * 8).min(screen_w / 3);
                let (r, g, b) = palette[focused_idx % 10];
                gl::Uniform4f(color_loc, r, g, b, 0.60);
                gl::Viewport(0, bar_y, name_w, bar_h);
                gl::Scissor(0, bar_y, name_w, bar_h);
                gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());

                // Volume bar
                let vol = cell.volume() as f32 / 150.0;
                let vol_w = (screen_w as f32 * 0.08 * vol) as i32;
                let vol_x = screen_w - vol_w - 60;
                gl::Uniform4f(color_loc, 0.20, 0.70, 0.40, 0.70);
                gl::Viewport(vol_x, bar_y + 4, vol_w, bar_h - 8);
                gl::Scissor(vol_x, bar_y + 4, vol_w, bar_h - 8);
                gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());

                // Speed indicator
                let spd = (cell.speed / 4.0) as f32;
                let spd_w = (screen_w as f32 * 0.06 * spd) as i32;
                let spd_x = screen_w - spd_w - 10;
                gl::Uniform4f(color_loc, 0.30, 0.50, 0.90, 0.60);
                gl::Viewport(spd_x, bar_y + 4, spd_w, bar_h - 8);
                gl::Scissor(spd_x, bar_y + 4, spd_w, bar_h - 8);
                gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
            }

            gl::Disable(gl::SCISSOR_TEST);
            gl::Viewport(0, 0, screen_w, screen_h);
            gl::BindVertexArray(0);
            gl::UseProgram(0);
        }
    }
}

impl Drop for RenderState {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
            gl::DeleteProgram(self.solid_program);
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteBuffers(1, &self.ebo);
            gl::DeleteVertexArrays(1, &self.logo_vao);
            gl::DeleteBuffers(1, &self.logo_vbo);
        }
    }
}

unsafe fn compile_shader(source: &str, shader_type: u32) -> Result<u32, String> {
    let shader = gl::CreateShader(shader_type);
    let len = source.len() as i32;
    let src = source.as_ptr() as *const i8;
    gl::ShaderSource(shader, 1, &src, &len);
    gl::CompileShader(shader);
    let mut success = 0i32;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
    if success == 0 {
        let mut buf = vec![0u8; 512];
        let mut len = 0i32;
        gl::GetShaderInfoLog(shader, 512, &mut len, buf.as_mut_ptr() as *mut _);
        let msg = String::from_utf8_lossy(&buf[..len as usize]);
        gl::DeleteShader(shader);
        return Err(msg.into_owned());
    }
    Ok(shader)
}

unsafe fn create_shader_program(vs_src: &str, fs_src: &str) -> Result<u32, String> {
    let vs = compile_shader(vs_src, gl::VERTEX_SHADER)?;
    let fs = compile_shader(fs_src, gl::FRAGMENT_SHADER)?;
    let program = gl::CreateProgram();
    gl::AttachShader(program, vs);
    gl::AttachShader(program, fs);
    gl::LinkProgram(program);
    let mut success = 0i32;
    gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
    if success == 0 {
        let mut buf = vec![0u8; 512];
        let mut len = 0i32;
        gl::GetProgramInfoLog(program, 512, &mut len, buf.as_mut_ptr() as *mut _);
        let msg = String::from_utf8_lossy(&buf[..len as usize]);
        gl::DeleteProgram(program);
        gl::DeleteShader(vs);
        gl::DeleteShader(fs);
        return Err(msg.into_owned());
    }
    gl::DeleteShader(vs);
    gl::DeleteShader(fs);
    Ok(program)
}
