extern crate gunship;

use gunship::engine::EngineBuilder;

fn main() {
    let mut builder = EngineBuilder::new();
    builder.max_workers(1);
    builder.build(|| {});
}
