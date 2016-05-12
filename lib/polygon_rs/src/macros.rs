/// Derives the `Counter` trait for any tuple-like struct declared as `SomeStruct(usize)`.
///
/// Note that the inner value doesn't have to be `usize`, it only needs to be an integer type.
macro_rules! derive_Counter {
    ($type_name: ident) => {
        impl $crate::Counter for $type_name {
            fn initial() -> Self {
                $type_name(0)
            }

            fn next(&mut self) -> Self {
                let next = *self;
                self.0 += 1;
                next
            }
        }
    }
}
