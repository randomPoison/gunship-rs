extern crate fiber;

use fiber::Fiber;

#[test]
fn basic_usage() {
    fn fiber_proc(suspended: Fiber) -> ! {
        println!("Suspended fiber: {:?}", suspended);
        unsafe { suspended.resume(); }

        panic!("Uh-oh, shouldn't have resumed this fiber again");
    }

    let fiber = Fiber::new(1024, fiber_proc);
    let fiber_id = fiber.id();

    let prev = unsafe { fiber.resume() };
    assert_eq!(fiber_id, prev.id());
}
