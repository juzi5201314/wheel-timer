use std::any::Any;
use std::time::Duration;

use crossbeam_channel::Receiver;

use crate::behave::Behave;
use crate::callback::BoxedCallback;
use crate::handle::AddHandle;
use crate::task::Task;

#[derive(Clone, Debug, Default)]
pub struct MoveTo(pub Vec<(usize, usize)>);

#[derive(Debug)]
pub struct MultiWheel {
    capacity: usize,
    wheels: Vec<Wheel>,
    granularity: Duration,

    #[allow(clippy::type_complexity)]
    add_handle: (
        Receiver<(BoxedCallback, Duration, Box<dyn Any + Send>)>,
        AddHandle,
    ),
}

#[derive(Default, Debug)]
pub struct Wheel {
    slots: Vec<Slot>,
    index: usize,
}

#[derive(Default, Debug)]
pub struct Slot {
    tasks: Vec<Task>,
}

impl MultiWheel {
    /// Create a roulette wheel consisting of multiple time wheels
    ///
    /// `n`: `n` layers of wheels
    /// `capacity`: how many slots per wheel
    /// `granularity`: the granularity of the time wheel, which must be consistent with the interval at which [MultiWheel::tick] is called.
    pub fn new(n: usize, capacity: usize, granularity: Duration) -> Self {
        let mut wheels = Vec::with_capacity(n);
        wheels.resize_with(n, || Wheel::new(capacity));

        let (tx, rx) = crossbeam_channel::unbounded();
        Self {
            capacity,
            wheels,
            granularity,
            add_handle: (rx, AddHandle(tx)),
        }
    }

    #[inline]
    pub fn add_handle(&self) -> AddHandle {
        self.add_handle.1.clone()
    }

    fn add_task(&mut self, cb: BoxedCallback, dur: Duration, ctx: Box<dyn Any + Send>) {
        let tick = (dur.as_nanos() as f64 / self.granularity.as_nanos() as f64).round() as usize;
        let task = Task {
            cb,
            round: 0,
            move_to: Default::default(),
            ctx,
            tick,
        };
        self.add(task)
    }

    fn add(&mut self, mut task: Task) {
        let mut layer = 0;
        let mut ticks = task.tick;
        let slot_pos = loop {
            let current_index = self.wheels.get(layer).expect("empty wheel").index;

            if task.tick == 0 {
                // if the duration of this task is greater than the granularity of the time wheel,
                // it is scheduled to run at the next moment.
                // worst case runs after a `granular`, fastest possible `immediately`

                // nearest position
                break current_index + 1;
            } else {
                let current = (current_index + ticks) % self.capacity;
                let next = (current_index + ticks) / self.capacity;

                if next == 0 {
                    break current;
                } else if self.wheels.get(layer + 1).is_some() {
                    task.move_to.0.push((layer, current));
                    layer += 1;
                    ticks = next;
                } else {
                    task.round = next - 1;
                    task.move_to.0.push((layer, current));
                    break 0;
                }
            }
        };

        self.wheels
            .get_mut(layer)
            .unwrap()
            .slots
            .get_mut(slot_pos)
            .unwrap()
            .tasks
            .push(task);
    }

    fn roll(&mut self, wheel_index: usize) -> Option<()> {
        if self.wheels.get_mut(wheel_index)?.roll() {
            self.roll(wheel_index + 1);
        }

        let mut task_i = 0;
        loop {
            let wheel = self.wheels.get_mut(wheel_index).unwrap();
            let slot = &mut wheel.slots.get_mut(wheel.index).unwrap().tasks;
            if slot.is_empty() {
                break;
            }

            let mut task = if let Some(task) = slot.get_mut(task_i) {
                task
            } else {
                break;
            };

            if task.round > 0 {
                task.round -= 1;
                task_i += 1;
            } else if let Some((layer, slot_index)) = task.move_to.0.pop() {
                let task = slot.swap_remove(task_i);
                let last_wheel = self.wheels.get_mut(layer).unwrap();
                last_wheel
                    .slots
                    .get_mut(slot_index)
                    .unwrap()
                    .tasks
                    .push(task);
            } else {
                let mut task = slot.swap_remove(task_i);
                match task.cb.call(&mut task.ctx) {
                    Behave::Cancel => {}
                    Behave::Change(dur) => {
                        self.add_task(task.cb, dur, task.ctx);
                    }
                    Behave::Repeat => {
                        self.add(Task {
                            cb: task.cb,
                            round: 0,
                            move_to: Default::default(),
                            ctx: task.ctx,
                            tick: task.tick,
                        });
                    }
                }
            }
        }

        Some(())
    }

    /// move wheel to next tick
    #[inline]
    pub fn tick(&mut self) {
        while let Ok((cb, dur, ctx)) = self.add_handle.0.try_recv() {
            self.add_task(cb, dur, ctx)
        }
        self.roll(0);
    }
}

impl Wheel {
    pub fn new(capacity: usize) -> Wheel {
        let mut slots = Vec::with_capacity(capacity);
        slots.resize_with(capacity, Default::default);
        Wheel {
            slots,
            ..Default::default()
        }
    }

    fn roll(&mut self) -> bool {
        if self.index == self.slots.len() - 1 {
            self.index = 0;
            true
        } else {
            self.index += 1;
            false
        }
    }
}
