pub struct Collector;

impl Collector {
    pub fn new() -> Result<Box<Collector>, ()> {
        Ok(Box::new(Collector))
    }

    pub fn flush_to_file(&mut self, _file_name: &str) {
    }
}

impl Drop for Collector {
    /// Doesn't do anything, but ensure that anything that has a Collector as a member still
    /// requires drop in case that has any implications for compilation.
    fn drop(&mut self) {
    }
}

pub struct Stopwatch;

impl Stopwatch {
    pub fn new(_name: &'static str) -> Stopwatch {
        Stopwatch
    }
}

impl Drop for Stopwatch {
    /// Doesn't do anything, but ensure that anything that has a Collector as a member still
    /// requires drop in case that has any implications for compilation.
    fn drop(&mut self) {
    }
}
