/// A trait defining a common interface for singleton objects.
///
/// While it is rarely useful to be able to act generically over any singleton type (and it's not
/// possible to create a `Singleton` trait object) the process for implementing a singleton is the
/// same for all types. As such this type is useful in combination with `#[singleton]`
/// which allows for a type to easily be made into a singleton.
///
/// NOTE: `#[singleton]` has not yet been implemented, so for now you'll have to do it manually :^(
pub unsafe trait Singleton: 'static + Sized {
    /// Creates the instance of the singleton.
    fn set_instance(instance: Self);

    /// Retrieves an immutable reference to the singleton instance.
    ///
    /// Only shared references to the instance can be safely retrieved. Allowing retrieval of
    /// mutable references would be unsafe because there's no way for Rust to statically avoid
    /// shared mutability. If no instance exists when this function is called the implementation
    /// must panic.
    fn instance() -> &'static Self;

    /// Destroys the instance of the singleton.
    ///
    /// This function is unsafe because it is not possible to statically know that there are no
    /// existing references to the instance. If there is the references will be dangling and will
    /// lead to a memory corruption.
    unsafe fn destroy_instance();
}
