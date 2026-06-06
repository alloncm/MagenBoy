use core::{ffi::{c_void, CStr}, ptr::{null, null_mut}};
use alloc::{ffi::CString, rc::Rc};

use gl::types::*;

use magenboy_core::{ppu::gb_ppu::{SCREEN_HEIGHT, SCREEN_WIDTH}, Pixel};

#[derive(Clone)]
pub struct GlGfxDevice {
    renderer: Rc<GlRenderer>
}

impl GlGfxDevice {
    pub fn new(width: u32, height: u32, gl_load_fn: impl FnMut(&'static str) -> *const c_void) -> Self {
        gl::load_with(gl_load_fn);
        Self::update_viewport(width as i32, height as i32);
        
        Self {
            renderer: Rc::new(GlRenderer::new()),
        }
    }

    pub fn update_viewport(width: i32, height: i32) {
        const GB_SCREEN_RATIO: f32 = SCREEN_HEIGHT as f32 / SCREEN_WIDTH as f32;

        let window_ratio = height as f32 / width as f32;
        let ratio = window_ratio / GB_SCREEN_RATIO;

        let (new_width, new_height) = if ratio < 1.0 {
            ((width as f32 * ratio) as i32, height)
        } 
        else if ratio > 1.0 {
            // invert ratio since it is now positive
            (width, (height as f32 * (1.0 / ratio)) as i32)
        }
        else {
            // exatly 1 no need to change the ratio
            (width, height)
        };

        let width_gap = (width - new_width) / 2;
        let height_gap = (height - new_height) / 2;
        unsafe{gl::Viewport(width_gap, height_gap, new_width, new_height)};
    }

    pub fn render(&mut self, buffer:&[Pixel], width: u32, height: u32) {
        self.renderer.render(buffer, width, height);
    }
}

const VERTEX_SHADER_SOURCE: &'static str = include_str!("vertex.glsl");
const FRAGMENT_SHADER_SOURCE: &'static str = include_str!("fragment.glsl");

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Vertex {
    pos_x: GLfloat,
    pos_y: GLfloat,
    tex_x: GLfloat,
    tex_y: GLfloat
}

// texture coordinates are flipped vertically since OpenGL's texture origin is bottom-left
const POS_TEX_VERTICES: [Vertex; 4] = [
    Vertex { pos_x: 1.0, pos_y: 1.0, tex_x: 1.0, tex_y: 0.0 },     // top right
    Vertex { pos_x: 1.0, pos_y: -1.0, tex_x: 1.0, tex_y: 1.0 },    // bottom right
    Vertex { pos_x: -1.0, pos_y: -1.0, tex_x: 0.0, tex_y: 1.0 },   // bottom left
    Vertex { pos_x: -1.0, pos_y: 1.0, tex_x: 0.0, tex_y: 0.0 }     // top left
];

const INDICIES: [u32; 6] = [
    0, 1, 3, // first triangle
    1, 2, 3  // second triangle
];

struct GlRenderer {
    shader_program: GLuint,
    vertex_array_object: GLuint,
    vertex_buffer_object: GLuint,
    element_buffer_object: GLuint,
    texture_object: GLuint,
}

impl GlRenderer {
    pub fn new() -> Self{
        unsafe {
            let shader_program = Self::link_shader_program();

            let mut vertex_buffer_object: GLuint = 0;
            let mut vertex_array_object: GLuint = 0;
            let mut element_buffer_object: GLuint = 0;
            gl::GenVertexArrays(1, &mut vertex_array_object as *mut _);
            gl::GenBuffers(1, &mut vertex_buffer_object as *mut _);
            gl::GenBuffers(1, &mut element_buffer_object as *mut _);
            // bind the Vertex Array Object first, then bind and set vertex buffer(s), and then configure vertex attributes(s).
            gl::BindVertexArray(vertex_array_object);

            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer_object);
            gl::BufferData(gl::ARRAY_BUFFER, core::mem::size_of_val(&POS_TEX_VERTICES) as GLsizeiptr, POS_TEX_VERTICES.as_ptr() as *const _, gl::STATIC_DRAW);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, element_buffer_object);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, core::mem::size_of_val(&INDICIES) as GLsizeiptr, INDICIES.as_ptr() as *const _, gl::STATIC_DRAW);

            let stride = core::mem::size_of::<Vertex>() as GLint;
            // pos attribute
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, 0 as *const _);
            gl::EnableVertexAttribArray(0);
            // tex coord attribute
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (2 * core::mem::size_of::<GLfloat>()) as *const _);
            gl::EnableVertexAttribArray(1);

            let texture_object = Self::allocate_texture_object();

            gl::UseProgram(shader_program);

            Self::bind_uniform_texture(shader_program);

            return Self {
                shader_program,
                vertex_array_object,
                vertex_buffer_object,
                element_buffer_object,
                texture_object,
            };
        }
    }

    pub fn render(&self, buffer: &[Pixel], width: u32, height: u32) {
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
                width as _, 
                height as _, 
                gl::RGB, 
                gl::UNSIGNED_SHORT_5_6_5, 
                buffer.as_ptr() as *const _
            );

            if let Err(e) = Self::check_gl_error() {
                core::panic!("GL error after TexSubImage2D: {}", e);
            }

            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const _);
        }
    }

    unsafe fn link_shader_program() -> u32 {
        // vertex shader
        let vertex_shader_source = CString::new(VERTEX_SHADER_SOURCE).unwrap();
        let vertex_shader: GLuint = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(vertex_shader, 1, &vertex_shader_source.as_ptr(), null());
        gl::CompileShader(vertex_shader);
        Self::veify_shader_compile_status(vertex_shader);

        // fragment shader
        let fragment_shader: GLuint = gl::CreateShader(gl::FRAGMENT_SHADER);
        let fragment_shader_source = CString::new(FRAGMENT_SHADER_SOURCE).unwrap();
        gl::ShaderSource(fragment_shader, 1, &fragment_shader_source.as_ptr(), null());
        gl::CompileShader(fragment_shader);
        Self::veify_shader_compile_status(fragment_shader);

        // link shaders
        let shader_program: GLuint = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);
        // check for linking errors
        Self::veify_shader_program_link_status(shader_program);

        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);
        shader_program
    }

    unsafe fn veify_shader_compile_status(shader: u32) {
        let mut success:GLint = 0;
        let mut info_log:[GLchar; 512] = [0; 512];

        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success as *mut GLint);
        if success == 0 {
            gl::GetShaderInfoLog(shader, info_log.len() as GLint, null_mut(), info_log.as_mut_ptr());
            let info_log = CStr::from_ptr(info_log.as_ptr());
            core::panic!("shader compilation failed {:?}", info_log);
        }
    }

    unsafe fn veify_shader_program_link_status(program: u32) {
        let mut success:GLint = 0;
        let mut info_log:[GLchar; 512] = [0; 512];

        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success as *mut GLint);
        if success == 0 {
            gl::GetProgramInfoLog(program, info_log.len() as GLint,null_mut(), info_log.as_mut_ptr());
            let info_log = CStr::from_ptr(info_log.as_ptr());
            core::panic!("shader program link failed {:?}", info_log);
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

    unsafe fn allocate_texture_object() -> u32 {
        let mut texture_object: GLuint = 0;
        gl::GenTextures(1, &mut texture_object as *mut _);
        gl::BindTexture(gl::TEXTURE_2D, texture_object);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as GLint);

        // Nearest upscaling, those must be set for the texture to render
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
        if let Err(e) = Self::check_gl_error(){
            panic!("Error creating texture: {}", e);
        }
        return texture_object;
    }

    /// shader program must be active.
    /// Since we have only single texture uniform OpenGL will auto bind it on most hardware
    unsafe fn bind_uniform_texture(shader_program: u32) {
        let texture_name = CString::new("Texture").unwrap();
        let uniform_location = gl::GetUniformLocation(shader_program, texture_name.as_ptr());
        gl::Uniform1i(uniform_location, 0);
        gl::ActiveTexture(gl::TEXTURE0);
    }
}

impl Drop for GlRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vertex_array_object);
            gl::DeleteBuffers(1, &self.vertex_buffer_object);
            gl::DeleteBuffers(1, &self.element_buffer_object);
            gl::DeleteTextures(1, &self.texture_object);
            gl::DeleteProgram(self.shader_program);
        }
    }
}
