use crate::{
    error::{Error, Result},
    scheduler::{context::Context, lifecycle::idle, state::State, switch::Switch, task::Task},
};

#[derive(Debug)]
pub struct RoundRobin<const N: usize> {
    tasks: [Option<Task>; N],
    current: usize,
    first_run: bool,
}

impl<const N: usize> RoundRobin<N> {
    pub fn new() -> Self {
        let mut scheduler = Self {
            tasks: [const { None }; N],
            current: 0,
            first_run: true,
        };

        let task = Task::new_kernel(idle::idle).expect("Can't spawn the idle task");

        scheduler.tasks[0] = Some(task);

        scheduler
    }

    pub fn prepare(&mut self) -> Switch {
        let len = self.tasks.len();

        if len == 0 {
            return Switch::None;
        }

        if self.first_run {
            let idle = self.tasks[0].as_mut().expect("Idle task must exist");

            idle.state = State::Running;
            self.current = 0;
            self.first_run = false;

            return Switch::Launch(&idle.context as *const Context);
        }

        let current = self.current;

        for offset in 1..len {
            let next = (current + offset) % len;

            // Idle task is the fallback.
            if next == 0 {
                continue;
            }

            let Some(task) = self.tasks[next].as_ref() else {
                continue;
            };

            if task.state != State::Ready {
                continue;
            }

            let (current_task, next_task) = if current < next {
                let (left, right) = self.tasks.split_at_mut(next);
                (left[current].as_mut(), right[0].as_mut())
            } else {
                let (left, right) = self.tasks.split_at_mut(current);
                (right[0].as_mut(), left[next].as_mut())
            };

            let next_task = next_task.expect("Next task must exist");

            self.current = next;

            if let Some(current_task) = current_task {
                current_task.state = State::Ready;
                next_task.state = State::Running;

                let old_ctx = &mut current_task.context as *mut Context;
                let new_ctx = &next_task.context as *const Context;

                return Switch::Swap(old_ctx, new_ctx);
            }

            let new_ctx = &next_task.context as *const Context;

            return Switch::Launch(new_ctx);
        }

        Switch::None
    }

    pub fn spawn(&mut self, task: Task) -> Result<usize> {
        if let Some(index) = self.tasks.iter().position(Option::is_none) {
            self.tasks[index] = Some(task);

            return Ok(index);
        }

        Err(Error::ProcessLimitReached)
    }
}
