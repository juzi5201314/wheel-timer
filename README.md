# wheel-timer
A timer based on a multi-time wheel structure

This library uses a multi-layered time wheel structure.

When a task is added to the wheel, it will go to the wheel with the coarsest granularity first, 
then to the wheel with a higher granularity, until the specified time is reached.

If the task is too long to fit on all the wheels, the task will be run by adding `round`,
if there are too many too long tasks, it will cause a lot of tasks to accumulate in the coarsest granularity wheel, 
so the number of layers, 
capacity and granularity of the wheel should be adjusted according to actual needs to avoid this kind of thing.

## Examples
look [examples](./examples)
