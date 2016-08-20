extern crate gunship;

use gunship::*;

fn main() {
    // Create the engine.
    EngineBuilder::new().build();

    // Run the main loop.
    Engine::start();
}
