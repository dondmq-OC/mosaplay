//! OpenGL renderer: compositing video textures into a grid layout.

use crate::cell::VideoCell;

/// Vertex shader: full-screen quad with texture coordinates
const VERTEX_SHADER: &str = r#"
#version 330 core
layout(location = 0) in vec2 aPos;
layout(location = 1) in vec2 aTexCoord;
out vec2 vTexCoord;
uniform mat4 uMVP;
void main() {
    gl_Position = uMVP * vec4(aPos, 0.0, 1.0);
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

/// Solid-color fragment shader for focus border / background
const SOLID_FRAGMENT: &str = r#"
#version 330 core
out vec4 FragColor;
uniform vec4 uColor;
void main() {
    FragColor = uColor;
}
"#;

/// GPU resources for rendering
pub struct RenderState {
    pub program: u32,       // main video texture shader
    pub solid_program: u32, // solid color shader
    vao: u32,
    vbo: u32,
    ebo: u32,
}

impl RenderState {
    pub fn new() -> Result<Self, String> {
        unsafe {
            let program = create_shader_program(VERTEX_SHADER, FRAGMENT_SHADER)?;
            let solid_program = create_shader_program(VERTEX_SHADER, SOLID_FRAGMENT)?;

            // Quad vertices: position(2) + texcoord(2)
            #[rustfmt::skip]
            let vertices: [f32; 16] = [
                // pos      texcoord
                -1.0, -1.0,  0.0, 0.0,  // bottom-left
                 1.0, -1.0,  1.0, 0.0,  // bottom-right
                 1.0,  1.0,  1.0, 1.0,  // top-right
                -1.0,  1.0,  0.0, 1.0,  // top-left
            ];
            let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];

            let mut vao = 0;
            let mut vbo = 0;
            let mut ebo = 0;

            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);

            gl::BindVertexArray(vao);

            // VBO
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            let size = (vertices.len() * std::mem::size_of::<f32>()) as isize;
            gl::BufferData(gl::ARRAY_BUFFER, size, vertices.as_ptr() as *const _, gl::STATIC_DRAW);

            // EBO
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            let size = (indices.len() * std::mem::size_of::<u32>()) as isize;
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, size, indices.as_ptr() as *const _, gl::STATIC_DRAW);

            // Position attribute (location 0)
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, (4 * std::mem::size_of::<f32>()) as i32, 0 as *const _);
            gl::EnableVertexAttribArray(0);

            // TexCoord attribute (location 1)
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, (4 * std::mem::size_of::<f32>()) as i32, (2 * std::mem::size_of::<f32>()) as *const _);
            gl::EnableVertexAttribArray(1);

            gl::BindVertexArray(0);

            Ok(Self { program, solid_program, vao, vbo, ebo })
        }
    }

    /// Render all video cells in the grid layout
    pub fn render_grid(
        &self,
        cells: &[VideoCell],
        focused_idx: usize,
        screen_w: i32,
        screen_h: i32,
        margin: i32,
    ) {
        unsafe {
            // Clear background to dark gray
            gl::ClearColor(0.08, 0.08, 0.10, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(self.program);
            gl::BindVertexArray(self.vao);

            for (_i, cell) in cells.iter().enumerate() {
                self.render_cell(cell, screen_w, screen_h, margin);
            }

            // Draw focus border on focused cell
            if !cells.is_empty() {
                let cell = &cells[focused_idx % cells.len()];
                self.render_focus_border(cell, screen_w, screen_h, margin);
            }

            gl::BindVertexArray(0);
            gl::UseProgram(0);
        }
    }

    fn render_cell(&self, cell: &VideoCell, screen_w: i32, screen_h: i32, _margin: i32) {
        unsafe {
            // Calculate normalized device coordinates
            let x = cell.x;
            let y = cell.y;
            let w = cell.w;
            let h = cell.h;

            let left   = (x as f32 / screen_w as f32) * 2.0 - 1.0;
            let right  = ((x + w) as f32 / screen_w as f32) * 2.0 - 1.0;
            let bottom = 1.0 - ((y + h) as f32 / screen_h as f32) * 2.0;
            let top    = 1.0 - (y as f32 / screen_h as f32) * 2.0;

            // Build MVP matrix for this cell (orthographic scale + translate)
            let sx = (right - left) / 2.0;
            let sy = (top - bottom) / 2.0;
            let tx = (right + left) / 2.0;
            let ty = (top + bottom) / 2.0;

            let mvp: [f32; 16] = [
                sx, 0.0, 0.0, 0.0,
                0.0, sy, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                tx, ty, 0.0, 1.0,
            ];

            let loc = gl::GetUniformLocation(self.program, b"uMVP\0".as_ptr() as *const _);
            gl::UniformMatrix4fv(loc, 1, gl::FALSE, mvp.as_ptr());

            // bind cell texture
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, cell.texture);
            let tex_loc = gl::GetUniformLocation(self.program, b"uTexture\0".as_ptr() as *const _);
            gl::Uniform1i(tex_loc, 0);

            // Draw filled quad
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const _);
        }
    }

    fn render_focus_border(&self, cell: &VideoCell, screen_w: i32, screen_h: i32, _margin: i32) {
        unsafe {
            let border_thickness = 3.0;

            // Slightly expand beyond the cell
            let x = cell.x as f32 - border_thickness;
            let y = cell.y as f32 - border_thickness;
            let w = cell.w as f32 + border_thickness * 2.0;
            let h = cell.h as f32 + border_thickness * 2.0;

            let left   = (x / screen_w as f32) * 2.0 - 1.0;
            let right  = ((x + w) / screen_w as f32) * 2.0 - 1.0;
            let bottom = 1.0 - ((y + h) / screen_h as f32) * 2.0;
            let top    = 1.0 - (y / screen_h as f32) * 2.0;

            gl::UseProgram(self.solid_program);

            let sx = (right - left) / 2.0;
            let sy = (top - bottom) / 2.0;
            let tx = (right + left) / 2.0;
            let ty = (top + bottom) / 2.0;

            let mvp: [f32; 16] = [
                sx, 0.0, 0.0, 0.0,
                0.0, sy, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                tx, ty, 0.0, 1.0,
            ];

            let loc = gl::GetUniformLocation(self.solid_program, b"uMVP\0".as_ptr() as *const _);
            gl::UniformMatrix4fv(loc, 1, gl::FALSE, mvp.as_ptr());

            // Bright orange focus indicator
            let color_loc = gl::GetUniformLocation(self.solid_program, b"uColor\0".as_ptr() as *const _);
            gl::Uniform4f(color_loc, 1.0, 0.55, 0.0, 0.9);

            // Draw wireframe quad (border only)
            gl::LineWidth(border_thickness);
            gl::DrawArrays(gl::LINE_LOOP, 0, 4);
            gl::UseProgram(self.program); // restore
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
        }
    }
}

/// Compile a shader from source
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

/// Create a linked shader program from vertex + fragment sources
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
