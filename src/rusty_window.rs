#![feature(core, env, std_misc)]

#[macro_use]
extern crate log;

extern crate winapi;
extern crate "gdi32-sys" as gdi32;
extern crate "opengl32-sys" as opengl32;

extern crate gl;

#[macro_use]
extern crate "rust-windows" as windows;

use std::ptr;
use std::mem;
use std::str;
use std::ffi::*;

use gl::types::*;

use winapi::{HBRUSH, CREATESTRUCTW, WORD, PIXELFORMATDESCRIPTOR,
             PFD_DRAW_TO_WINDOW, PFD_SUPPORT_OPENGL, PFD_DOUBLEBUFFER, PFD_TYPE_RGBA, PFD_MAIN_PLANE};

use windows::main_window_loop;
use windows::instance::Instance;
use windows::resource::*;
use windows::window::{WindowImpl, Window, WndClass, WindowParams};
use windows::window::{OnCreate, OnSize, OnDestroy, OnPaint, OnFocus};
use windows::window;
use windows::gdi::PaintDc;

static IDI_ICON: isize = 0x101;
static MENU_MAIN: isize = 0x201;

struct MainFrame
{
    win: Window
    // shader: GLuint
    // vertex_buffer: GLuint
}

wnd_proc!(MainFrame, win, WM_CREATE, WM_DESTROY, WM_SIZE, WM_SETFOCUS, WM_PAINT);

impl OnCreate for MainFrame {
    fn on_create(&self, _cs: &CREATESTRUCTW) -> bool {
        self.init_opengl();

        true
    }
}

impl OnSize for MainFrame {
}

impl OnDestroy for MainFrame {}

impl OnPaint for MainFrame {
    fn on_paint(&self) {
        unsafe {
            // TODO rebind the buffers or whatever

            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Draw a triangle from the 3 vertices
            gl::DrawArrays(gl::TRIANGLE_FAN, 0, 5);

            gdi32::SwapBuffers(opengl32::wglGetCurrentDC());
        }
    }
}

impl OnFocus for MainFrame {
    fn on_focus(&self, _w: Window) {

    }
}

impl MainFrame {
    fn new(instance: Instance, title: String) -> Option<Window> {
        let icon = Image::load_resource(instance, IDI_ICON, ImageType::IMAGE_ICON, 0, 0);
        let wnd_class = WndClass {
            classname: "MainFrame".to_string(),
            style: 0x0001 | 0x0002 | 0x0020, // CS_HREDRAW | CS_VREDRAW | CS_OWNDC // TODO: figure out how to do this with not hard-coded values
            icon: icon,
            icon_small: None,
            cursor: Image::load_cursor_resource(32514), // hourglass
            background: (5 + 1) as HBRUSH,
            menu: MenuResource::MenuId(MENU_MAIN),
            cls_extra: 0,
            wnd_extra: 0,
        };
        let res = wnd_class.register(instance);
        if !res {
            return None;
        }

        let wproc = Box::new(MainFrame {
            win: Window::null()
        });

        let win_params = WindowParams {
            window_name: title,
            style: window::WS_OVERLAPPEDWINDOW,
            x: 0,
            y: 0,
            width: 400,
            height: 400,
            parent: Window::null(),
            menu: ptr::null_mut(),
            ex_style: 0,
        };

        Window::new(instance,
                    Some(wproc as Box<WindowImpl + 'static>),
                    wnd_class.classname.as_slice(),
                    &win_params)
    }

    fn init_opengl(&self) {
        let pdc = PaintDc::new(self).expect("Paint DC");

        let pfd = PIXELFORMATDESCRIPTOR {
            nSize: mem::size_of::<PIXELFORMATDESCRIPTOR>() as WORD,
            nVersion: 1,
            dwFlags: PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
            iPixelType: PFD_TYPE_RGBA,
            cColorBits: 16,
            cRedBits: 0,
            cRedShift: 0,
            cGreenBits: 0,
            cGreenShift: 0,
            cBlueBits: 0,
            cBlueShift: 0,
            cAlphaBits: 0,
            cAlphaShift: 0,
            cAccumBits: 0,
            cAccumRedBits: 0,
            cAccumGreenBits: 0,
            cAccumBlueBits: 0,
            cAccumAlphaBits: 0,
            cDepthBits: 16,
            cStencilBits: 1,
            cAuxBuffers: 0,
            iLayerType: PFD_MAIN_PLANE,
            bReserved: 0,
            dwLayerMask: 0,
            dwVisibleMask: 0,
            dwDamageMask: 0
        };

        let success = unsafe {
            let pixelformat = gdi32::ChoosePixelFormat(pdc.dc.raw, &pfd);
            gdi32::SetPixelFormat(pdc.dc.raw, pixelformat, &pfd);
            let render_context = opengl32::wglCreateContext(pdc.dc.raw);
            opengl32::wglMakeCurrent(pdc.dc.raw, render_context)
        };

        println!("{}", success);

        gl::load_with(|s| {
            let string = CString::new(s);
            unsafe {
                opengl32::wglGetProcAddress(string.unwrap().as_ptr())
            }
        });

        println!("{:?}", gl::DrawArrays::is_loaded());

        let mut vao = 0;
        let mut vbo = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            let vertex_data: [GLfloat; 15] =
            [ 0.00, 0.00, 0.00,
              0.50, 0.00, 0.00,
              0.50, 0.50, 0.00,
              0.25, 0.75, 0.00,
              0.00, 0.50, 0.00 ];

            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER,
                           (vertex_data.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                           mem::transmute(&vertex_data[0]),
                           gl::STATIC_DRAW);

            let vertex_shader_src = "\
#version 150                 \n\
                             \n\
in vec4 vertexPos;           \n\
                             \n\
void main(void)              \n\
{                            \n\
    gl_Position = vertexPos; \n\
}";

             let fragment_shader_src = "\
#version 150                                \n\
                                            \n\
out vec4 fragmentColor;                     \n\
                                            \n\
void main(void)                             \n\
{                                           \n\
    fragmentColor = vec4(1, 0, 0, 1);       \n\
}";

            let vs = compile_shader(vertex_shader_src, gl::VERTEX_SHADER);
            let fs = compile_shader(fragment_shader_src, gl::FRAGMENT_SHADER);
            let program = link_program(vs, fs);

            gl::UseProgram(program);
            gl::BindFragDataLocation(program, 0,
                                     CString::new(b"fragmentColor").unwrap().as_ptr());

            // Specify the layout of the vertex data
            let pos_attr = gl::GetAttribLocation(program,
                                                 CString::new(b"vertexPos").unwrap().as_ptr());
            gl::VertexAttribPointer(pos_attr as GLuint, 3, gl::FLOAT,
                                    gl::FALSE as GLboolean, 0, ptr::null());
            gl::EnableVertexAttribArray(pos_attr as GLuint);
        }
    }
}

fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    let shader;
    unsafe {
        shader = gl::CreateShader(ty);

        // Attempt to compile the shader
        let c_str = CString::new(src.as_bytes());
        gl::ShaderSource(shader, 1, &c_str.unwrap().as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Get the compile status
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            panic!("{}", str::from_utf8(buf.as_slice()).ok().expect("ShaderInfoLog not valid utf8"));
        }
    }
    shader
}

fn link_program(vs: GLuint, fs: GLuint) -> GLuint { unsafe {
    let program = gl::CreateProgram();
    gl::AttachShader(program, vs);
    gl::AttachShader(program, fs);
    gl::LinkProgram(program);

    // Get the link status
    let mut status = gl::FALSE as GLint;
    gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

    // Fail on error
    if status != (gl::TRUE as GLint) {
        let mut len: GLint = 0;
        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
        let mut buf = Vec::with_capacity(len as usize);
        buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
        gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
        panic!("{}", str::from_utf8(buf.as_slice()).ok().expect("ProgramInfoLog not valid utf8"));
    }
    program
} }

fn main() {
    let instance = Instance::main_instance();
    let main = MainFrame::new(instance, "Rusty Window".to_string());
    let main = main.unwrap();

    main.show(1);
    main.update();

    let exit_code = main_window_loop();
    std::env::set_exit_status(exit_code as i32);
}