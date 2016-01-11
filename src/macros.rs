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
                if unsafe { INSTANCE.is_some() } {
                    panic!("Cannot create singleton instance");
                }

                let instance = Box::new(instance);
                unsafe {
                    INSTANCE = Some(Box::into_raw(instance));
                }
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
