use std::time::Duration;
use wheel_timer::{Behave, MultiWheel};

const PRECISION: Duration = Duration::from_secs(1);

fn main() {
    let mut wheel = MultiWheel::new(10, 10, PRECISION);

    let add_handle = wheel.add_handle();

    add_handle
        .add(
            || {
                println!("hello");
            },
            Duration::from_secs(1),
        )
        .unwrap();
    add_handle
        .add(
            || {
                println!("world");
            },
            Duration::from_secs(2),
        )
        .unwrap();
    add_handle
        .add_with_ctx(
            |i: &mut usize| {
                println!("{}s", i);
                *i += 1;
                Behave::Repeat
            },
            Duration::from_secs(1),
            1,
        )
        .unwrap();

    loop {
        wheel.tick();
        std::thread::sleep(PRECISION);
    }
}
