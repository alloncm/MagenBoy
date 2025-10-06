use std::{ffi::{CStr, CString}, ptr::{null, null_mut}};

use gl::types::*;
use glfw_sys::{glfwSwapBuffers, GLFWwindow};
use magenboy_core::{ppu::gb_ppu::{SCREEN_HEIGHT, SCREEN_WIDTH}, GfxDevice};


const VERTEX_SHADER_SOURCE: &'static str = include_str!("vertex.glsl");
const FRAGMENT_SHADER_SOURCE: &'static str = include_str!("fragment.glsl");

pub struct Renderer {
    window: *mut GLFWwindow,
    shader_program: GLuint,
    vertex_array_object: GLuint,
    vertex_buffer_object: GLuint,
    element_buffer_object: GLuint,
}

impl Renderer{
    pub fn new(window: *mut GLFWwindow) -> Renderer{
        unsafe {
            // build and compile our shader program
            // ------------------------------------
            // vertex shader
            let vertex_shader_source = CString::new(VERTEX_SHADER_SOURCE).unwrap();
            let vertex_shader: GLuint = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(vertex_shader, 1, &vertex_shader_source.as_ptr(), null());
            gl::CompileShader(vertex_shader);
            // check for shader compile errors
            let mut success:GLint = 0;
            let mut info_log:[GLchar; 512] = [0; 512];
            gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success as *mut GLint);
            if success == 0 {
                gl::GetShaderInfoLog(vertex_shader, info_log.len() as GLint, null_mut(), info_log.as_mut_ptr());
                let info_log = CStr::from_ptr(info_log.as_ptr());
                println!("ERROR::SHADER::VERTEX::COMPILATION_FAILED\n {:?}", info_log);
            }
            // fragment shader
            let fragment_shader: GLuint = gl::CreateShader(gl::FRAGMENT_SHADER);
            let fragment_shader_source = CString::new(FRAGMENT_SHADER_SOURCE).unwrap();
            gl::ShaderSource(fragment_shader, 1, &fragment_shader_source.as_ptr(), null());
            gl::CompileShader(fragment_shader);
            // check for shader compile errors
            gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success as *mut GLint);
            if success == 0 {
                gl::GetShaderInfoLog(fragment_shader, 512, null_mut(), info_log.as_mut_ptr());
                let info_log = CStr::from_ptr(info_log.as_ptr());
                println!("ERROR::SHADER::FRAGMENT::COMPILATION_FAILED\n {:?}", info_log);
            }
            // link shaders
            let shader_program: GLuint = gl::CreateProgram();
            gl::AttachShader(shader_program, vertex_shader);
            gl::AttachShader(shader_program, fragment_shader);
            gl::LinkProgram(shader_program);
            // check for linking errors
            gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success as *mut GLint);
            if success == 0 {
                gl::GetProgramInfoLog(shader_program, info_log.len() as GLint,null_mut(), info_log.as_mut_ptr());
                println!("ERROR::SHADER::PROGRAM::LINKING_FAILED\n {:?}", info_log);
            }
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);

            // set up vertex data (and buffer(s)) and configure vertex attributes
            // ------------------------------------------------------------------
            let pos_tex_vertices: [GLfloat; 16] = [
                1.0, 1.0, 1.0, 1.0,     // top right
                1.0, -1.0, 1.0, 0.0,    // bottom right
                -1.0, -1.0, 0.0, 0.0,   // bottom left
                -1.0, 1.0, 0.0, 1.0     // top left
            ]; 
            let indicies: [GLuint; 6] = [
                0, 1, 3, // first triangle
                1, 2, 3  // second triangle
            ];

            let mut vertex_buffer_object: GLuint = 0;
            let mut vertex_array_object: GLuint = 0;
            let mut element_buffer_object: GLuint = 0;
            gl::GenVertexArrays(1, &mut vertex_array_object as *mut _);
            gl::GenBuffers(1, &mut vertex_buffer_object as *mut _);
            gl::GenBuffers(1, &mut element_buffer_object as *mut _);
            // bind the Vertex Array Object first, then bind and set vertex buffer(s), and then configure vertex attributes(s).
            gl::BindVertexArray(vertex_array_object);

            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer_object);
            gl::BufferData(gl::ARRAY_BUFFER, std::mem::size_of_val(&pos_tex_vertices) as GLsizeiptr, pos_tex_vertices.as_ptr() as *const _, gl::STATIC_DRAW);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, element_buffer_object);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, std::mem::size_of_val(&indicies) as GLsizeiptr, indicies.as_ptr() as *const _, gl::STATIC_DRAW);

            let stride = (4 * std::mem::size_of::<GLfloat>()) as GLint;
            // pos attribute
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, 0 as *const _);
            gl::EnableVertexAttribArray(0);
            // tex coord attribute
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, (2 * size_of::<GLfloat>()) as *const _);
            gl::EnableVertexAttribArray(1);

            // note that this is allowed, the call to glVertexAttribPointer registered VBO as the vertex attribute's bound vertex buffer object so afterwards we can safely unbind
            gl::BindBuffer(gl::ARRAY_BUFFER, 0); 

            let mut texture: GLuint = 0;
            gl::GenTextures(1, &mut texture as *mut _);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as GLint);
            // Nearest upscaling instead of linear
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as _, SCREEN_WIDTH as _, SCREEN_HEIGHT as _, 0, gl::RGB565, gl::UNSIGNED_SHORT_5_6_5, null());

            // You can unbind the VAO afterwards so other VAO calls won't accidentally modify this VAO, but this rarely happens. Modifying other
            // VAOs requires a call to glBindVertexArray anyways so we generally don't unbind VAOs (nor VBOs) when it's not directly necessary.
            gl::BindVertexArray(0); 

            return Renderer {
                window,
                shader_program,
                vertex_array_object,
                vertex_buffer_object,
                element_buffer_object,
            };
        }
    }

    pub fn render(&self, buffer: &[u16; SCREEN_HEIGHT * SCREEN_WIDTH]) {
        unsafe {
            gl::UseProgram(self.shader_program);
            gl::BindVertexArray(self.vertex_array_object); // seeing as we only have a single VAO there's no need to bind it every time, but we'll do so to keep things a bit more organized
            gl::TexSubImage2D(gl::TEXTURE_2D, 0, 0, 0, SCREEN_WIDTH as _, SCREEN_HEIGHT as _, gl::RGB565, gl::UNSIGNED_SHORT_5_6_5, buffer.as_ptr() as *const _);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const _);
            
            glfwSwapBuffers(self.window);
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            // optional: de-allocate all resources once they've outlived their purpose:
            // ------------------------------------------------------------------------
            gl::DeleteVertexArrays(1, &self.vertex_array_object);
            gl::DeleteBuffers(1, &self.vertex_buffer_object);
            gl::DeleteBuffers(1, &self.element_buffer_object);
            gl::DeleteProgram(self.shader_program);
        }
    }
}

impl GfxDevice for Renderer{
    fn swap_buffer(&mut self, buffer:&[u16; SCREEN_HEIGHT * SCREEN_WIDTH]) {
        self.render(buffer);
    }
}
