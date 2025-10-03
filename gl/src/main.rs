use std::ffi::CString;

use glfw_sys::*;
use gl::types::*;

const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

fn main() {
    unsafe {
        if glfwInit() == 0 {
            println!("Failed to initialize GLFW");
            return;
        }

        glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 3);
        glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 3);
        glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);

        let window = glfwCreateWindow(
            SCR_WIDTH as i32,
            SCR_HEIGHT as i32,
            b"LearnOpenGL\0".as_ptr() as *const i8,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );

        if window.is_null() {
            println!("Failed to create GLFW window");
            glfwTerminate();
            return;
        }

        glfwMakeContextCurrent(window);
        glfwSetFramebufferSizeCallback(window, Some(framebuffer_size_callback));
        
        if glfwGetCurrentContext().is_null() {
            println!("Failed to get current context");
            return;
        }
        
        gl::load_with(|s| {
            let name = CString::new(s).unwrap();
            match glfwGetProcAddress(name.as_ptr()) {
                Some(p) => p as *const _,
                None => std::ptr::null(),
            }
        });

        while glfwWindowShouldClose(window) == 0 {
            // input
            // -----
            process_input(window);

            // render
            gl::ClearColor(0.7, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            
            // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
            // -------------------------------------------------------------------------------
            glfwSwapBuffers(window);
            glfwPollEvents();
        }

        // glfw: terminate, clearing all previously allocated GLFW resources.
        // ------------------------------------------------------------------
        glfwTerminate();
    }
}


unsafe extern "C" fn framebuffer_size_callback(
    _window: *mut GLFWwindow,
    width: i32,
    height: i32,
) {
    gl::Viewport(0, 0, width, height);
}

fn process_input(window: *mut GLFWwindow) {
    unsafe {
        if glfwGetKey(window, GLFW_KEY_ESCAPE) == GLFW_PRESS {
            glfwSetWindowShouldClose(window, 1);
        }
    }
}