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
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, 0 as *const _);
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (2 * std::mem::size_of::<f32>()) as *const _);
            gl::EnableVertexAttribArray(1);
            gl::BindVertexArray(0);

            Ok(Self { program, solid_program, vao, vbo, ebo })
        }
    }

    pub fn render_grid(
        &self, cells: &[VideoCell], focused_idx: usize, screen_w: i32, screen_h: i32,
    ) {
        unsafe {
            gl::ClearColor(0.08, 0.08, 0.10, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            for (i, cell) in cells.iter().enumerate() {
                // Flip Y: OpenGL has origin at bottom-left, our coords have origin at top-left
                let gl_y = screen_h - cell.y - cell.h;

                // Set viewport to this cell's region
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
                gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const _);

                // Draw focus border
                if i == focused_idx {
                    gl::UseProgram(self.solid_program);
                    let color_loc = gl::GetUniformLocation(self.solid_program, b"uColor\0".as_ptr() as *const _);
                    gl::Uniform4f(color_loc, 1.0, 0.55, 0.0, 0.9);
                    gl::LineWidth(3.0);
                    gl::DrawArrays(gl::LINE_LOOP, 0, 4);
                }
            }

            // Restore full viewport
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
