use core::task::{RawWaker, Waker, RawWakerVTable, Context, Poll};

use alloc::collections::VecDeque;

use super::Task;

pub struct SimpleExecutor {
    queue: VecDeque<Task>,
}

impl SimpleExecutor {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        self.queue.push_back(task)
    }

    pub fn run(&mut self) {
        while let Some(mut task) = self.queue.pop_front() {
            let waker = dummy_waker();
            let mut ctx = Context::from_waker(&waker);
            
            match task.poll(&mut ctx) {
                Poll::Pending => self.queue.push_back(task),
                Poll::Ready(_) => continue,
            }
        }
    }
}

fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(0 as *const (), vtable)
}

fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}
