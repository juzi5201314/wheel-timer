# wheel-timer2
A timer based on a multi-time wheel structure

[![docs.rs](https://img.shields.io/docsrs/wheel-timer2/latest?style=for-the-badge)](https://docs.rs/wheel-timer2)

This library uses a multi-layered time wheel structure.

When a task is added to the wheel, it will go to the wheel with the coarsest granularity first, 
then to the wheel with a higher granularity, until the specified time is reached.

If the task is too long to fit on all the wheels, the task will be run by adding `round`,
if there are too many too long tasks, it will cause a lot of tasks to accumulate in the coarsest granularity wheel, 
so the number of layers, 
capacity and granularity of the wheel should be adjusted according to actual needs to avoid this kind of thing.

To be precise, this is a structure for managing and running timed tasks, not a complete timer, 
because the user himself needs to push (execute) it regularly.

## Usage
1. First, we create a `MultiWheel`:
```rust 
let mut wheel = MultiWheel::new(3, 10, Duration::from_millis(100));
```
this means creating a 3-layer round with 10 slots in each layer. Each slot differs by 100ms:

2. Then, we add a task that prints hello world after a delay of 300ms:
```rust 
let add_handle = wheel.add_handle();

add_handle
    .add(
        || {
            println!("hello");
        },
        Duration::from_millis(300),
    )
    .unwrap();
```
tips: [AddHandle](https://docs.rs/wheel-timer2/latest/wheel_timer2/struct.AddHandle.html) is a handle used to add tasks to the time wheel while the time wheel is running.

3. Finally, we run the time wheel:
```rust 
let mut i = tokio::time::interval(Duration::from_millis(100));
loop {
    i.tick().await;
    wheel.tick();
}
```
**The tick interval must be consistent with the granularity specified when the time wheel was created.
Otherwise, the task will be executed at the wrong time.**

## Examples
look [examples](./examples)

## Precision
Under normal circumstances, 
the error is mainly the error of each running tick plus the time consumed by the tick.

I recommend using `tokio::time::interval` instead of simply using loop+sleep, 
because you can adjust `MissedTickBehavior` to try to compensate for the precision.

If the time of the task is less than the granularity of the time wheel, it will be scheduled to run after the next tick or one granularity time, 
depending on whether the task time is closer to the granularity time or 0.
