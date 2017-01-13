/// Macro used for generating bindings to OpenGL procs.
///
/// The OpenGL implementation for a computer actually lives in its graphics card. In order to call
/// the various functions that are part of the OpenGL API we must load pointers to those functions.
/// This macro generates the necessary boilerplate for loading and stashing those pointers, as well
/// as handling failure when those pointers fail to load (i.e. panicking).
///
/// TODO: Add a variant where the same gl proc can be mapped to multiple rust functions to improve
/// type safety specification.
#[macro_export]
macro_rules! gl_proc {
    ( $proc_name:ident:
        $( #[$attr:meta] )* fn $fn_name:ident( $( $arg:ident : $arg_ty:ty ),* ) $( -> $result:ty )* ) => {
        $( #[$attr] )*
        pub unsafe fn $fn_name( $( $arg: $arg_ty, )* ) $( -> $result )* {
            match $fn_name::load() {
                Some(gl_proc) => gl_proc( $( $arg ),* ),
                None => panic!("Failed to load gl proc for {}", stringify!( $proc_name )),
            }
        }

        pub mod $fn_name {
            #[allow(unused_imports)]
            use types::*;

            static mut PROC_PTR: Option<ProcType> = None;

            pub type ProcType = extern "system" fn( $( $arg_ty, )* ) $( -> $result )*;

            pub unsafe fn load() -> Option<ProcType> {
                if let None = PROC_PTR {
                    let null_terminated_name = concat!(stringify!($proc_name), "\0");
                    PROC_PTR =
                        $crate::platform::load_proc(null_terminated_name)
                        .map(|ptr| ::std::mem::transmute(ptr));
                }

                PROC_PTR
            }
        }
    }
}
