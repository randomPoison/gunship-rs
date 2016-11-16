#[macro_export]
macro_rules! derive_Component {
    ($type_name: ty) => {
        impl $crate::ecs::Component for $type_name {
            type Manager = $crate::component::DefaultManager<$type_name>;
            type Message = $crate::component::DefaultMessage<$type_name>;
        }
    }
}

#[macro_export]
macro_rules! derive_Singleton {
    ($type_name: ident) => {
        static mut INSTANCE: Option<*mut $type_name> = None;

        unsafe impl $crate::singleton::Singleton for $type_name {
            fn set_instance(instance: Self) {
                println!("setting instance");
                if unsafe { INSTANCE.is_some() } {
                    panic!("Cannot create singleton instance");
                }

                let instance = Box::new(instance);
                unsafe {
                    INSTANCE = Some(Box::into_raw(instance));
                }
                println!("done setting instance");
            }

            fn instance() -> &'static Self {
                unsafe {
                    match INSTANCE {
                        Some(instance) => &*instance,
                        None => panic!("No instance found"),
                    }
                }
            }

            unsafe fn destroy_instance() {
                if let Some(instance) = INSTANCE {
                    Box::from_raw(instance);
                    INSTANCE = None;
                }
            }
        }
    }
}

// TODO: Make this threadsafe by useing `std::sync::Once`.
#[macro_export]
macro_rules! warn_once {
    ($message: expr) => {
        static mut HAS_WARNED: bool = false;

        unsafe {
            if !HAS_WARNED {
                HAS_WARNED = true;
                println!($message);
            }
        }
    }
}
