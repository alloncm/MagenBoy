use std::{ffi::{CStr, CString}, ptr::{null, null_mut}};

use gl::types::*;
use glfw_sys::{glfwGetFramebufferSize, glfwSwapBuffers, GLFWwindow};
use magenboy_core::{ppu::gb_ppu::{SCREEN_HEIGHT, SCREEN_WIDTH}, GfxDevice};


const VERTEX_SHADER_SOURCE: &'static str = include_str!("vertex.glsl");
const FRAGMENT_SHADER_SOURCE: &'static str = include_str!("fragment.glsl");

pub struct GlRenderer {
    window: *mut GLFWwindow,
    shader_program: GLuint,
    vertex_array_object: GLuint,
    vertex_buffer_object: GLuint,
    element_buffer_object: GLuint,
}

const SCREEN_RATIO: f32 = SCREEN_HEIGHT as f32 / SCREEN_WIDTH as f32;

const POS_TEX_VERTICES: [f32; 16] = [
    1.0, 1.0, 1.0, 0.0,     // top right
    1.0, -1.0, 1.0, 1.0,    // bottom right
    -1.0, -1.0, 0.0, 1.0,   // bottom left
    -1.0, 1.0, 0.0, 0.0     // top left
]; 
const INDICIES: [u32; 6] = [
    0, 1, 3, // first triangle
    1, 2, 3  // second triangle
];

impl GlRenderer{
    pub fn new(window: *mut GLFWwindow) -> GlRenderer{
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

            let mut vertex_buffer_object: GLuint = 0;
            let mut vertex_array_object: GLuint = 0;
            let mut element_buffer_object: GLuint = 0;
            gl::GenVertexArrays(1, &mut vertex_array_object as *mut _);
            gl::GenBuffers(1, &mut vertex_buffer_object as *mut _);
            gl::GenBuffers(1, &mut element_buffer_object as *mut _);
            // bind the Vertex Array Object first, then bind and set vertex buffer(s), and then configure vertex attributes(s).
            gl::BindVertexArray(vertex_array_object);

            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer_object);
            gl::BufferData(gl::ARRAY_BUFFER, std::mem::size_of_val(&POS_TEX_VERTICES) as GLsizeiptr, POS_TEX_VERTICES.as_ptr() as *const _, gl::STATIC_DRAW);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, element_buffer_object);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, std::mem::size_of_val(&INDICIES) as GLsizeiptr, INDICIES.as_ptr() as *const _, gl::STATIC_DRAW);

            let stride = (4 * std::mem::size_of::<GLfloat>()) as GLint;
            // pos attribute
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, 0 as *const _);
            gl::EnableVertexAttribArray(0);
            // tex coord attribute
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (2 * size_of::<GLfloat>()) as *const _);
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
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
            gl::TexImage2D(
                gl::TEXTURE_2D, 
                0, 
                gl::RGB as _, 
                SCREEN_WIDTH as _, 
                SCREEN_HEIGHT as _, 
                0, 
                gl::RGB, 
                gl::UNSIGNED_SHORT_5_6_5, 
                null()
            );
            if let Err(e) = check_gl_error(){
                panic!("Error creating texture: {}", e);
            }

            gl::UseProgram(shader_program);

            // Probably not required on all hardware
            let texture_name = CString::new("Texture").unwrap();
            let uniform_location = gl::GetUniformLocation(shader_program, texture_name.as_ptr());
            gl::Uniform1i(uniform_location, 0);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture);

            let (mut width, mut height) = (0, 0);
            glfwGetFramebufferSize(window, &mut width, &mut height);

            // Set initial viewport
            framebuffer_size_callback(window, width, height);

            return GlRenderer {
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
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // All the objects are already bound and the shader is already in use
            // Update the texture
            gl::TexSubImage2D(
                gl::TEXTURE_2D, 
                0, 
                0, 
                0, 
                SCREEN_WIDTH as _, 
                SCREEN_HEIGHT as _, 
                gl::RGB, 
                gl::UNSIGNED_SHORT_5_6_5, 
                buffer.as_ptr() as *const _
            );
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const _);
            
            glfwSwapBuffers(self.window);
        }
    }
}

fn check_gl_error() -> Result<(), &'static str> {
    let error = unsafe{gl::GetError()};
    if error != gl::NO_ERROR {
        return Err(match error {
            gl::INVALID_ENUM => "GL_INVALID_ENUM",
            gl::INVALID_VALUE => "GL_INVALID_VALUE",
            gl::INVALID_OPERATION => "GL_INVALID_OPERATION",
            gl::STACK_OVERFLOW => "GL_STACK_OVERFLOW",
            gl::STACK_UNDERFLOW => "GL_STACK_UNDERFLOW",
            gl::OUT_OF_MEMORY => "GL_OUT_OF_MEMORY",
            gl::INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION",
            _ => "Unknown error",
        });
    }

    return Ok(());
}

impl Drop for GlRenderer {
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

impl GfxDevice for GlRenderer{
    fn swap_buffer(&mut self, buffer:&[u16; SCREEN_HEIGHT * SCREEN_WIDTH]) {
        self.render(buffer);
    }
}

pub unsafe extern "C" fn framebuffer_size_callback(_window: *mut GLFWwindow, width: i32, height: i32) {
    let width_ratio = width as f32 / SCREEN_WIDTH as f32;
    let height_ratio = height as f32 / SCREEN_HEIGHT as f32;
    let window_ratio = height as f32 / width as f32;

    let (new_width, new_height) = if width_ratio > height_ratio {
        let ratio = window_ratio / SCREEN_RATIO;
        ((width as f32 * ratio) as i32, height)
    } 
    else if width_ratio < height_ratio {
        let ratio = SCREEN_RATIO / window_ratio;
        (width, (height as f32 * ratio) as i32)
    }
    else {
        (width, height)
    };

    let width_gap = (width - new_width) / 2;
    let height_gap = (height - new_height) / 2;
    gl::Viewport(width_gap, height_gap, new_width, new_height);
}