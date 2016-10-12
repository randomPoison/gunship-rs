extern crate fiber;

use fiber::Fiber;

#[test]
fn closure() {
    let fiber = Fiber::new(
        1024,
        |prev| {
            println!("Running fiber");
            let next = unsafe { prev.resume() };
            unsafe { next.resume(); }
        },
    );
    let fiber_id = fiber.id();

    let prev = unsafe { fiber.resume() };
    assert_eq!(fiber_id, prev.id());

    let other_prev = unsafe { prev.resume() };
    assert_eq!(fiber_id, other_prev.id());
}
