use std::sync::{Mutex, MutexGuard, Condvar};
use std::collections::VecDeque;

/// Deliver fitnesses to the threads that requested them.
///
/// This is essentially a single-producer, multiple-consumer queue,
/// where each consumer is waiting for an item with a specific ID.
/// Each time a new item is produced, the consumers all wake up and
/// examine it, and the producer who was waiting for that item's ID
/// actually consumes the item.
pub struct Queue<T: Clone + Send> {
    container: Mutex<VecDeque<(usize, T)>>,
    produced: Condvar,
}

impl<T: Clone + Send> Queue<T> {
    pub fn new() -> Queue<T> {
        Queue {
            container: Mutex::new(VecDeque::new()),
            produced: Condvar::new(),
        }
    }

    pub fn send(&self, id: usize, item: T) {
        let mut guard = self.container.lock().unwrap();
        guard.push_back((id, item));
        self.produced.notify_all()
    }

    fn _wait_for(&self, mut guard: MutexGuard<VecDeque<(usize, T)>>, id: usize) -> T {
        match guard.iter().position(|&(id2, _)| id2 == id) {
            Some(index) => {
                let (_, item) = guard.remove(index).unwrap();
                item
            }
            None => {
                self._wait_for(self.produced.wait(guard).unwrap(), id)
            }
        }
    }

    pub fn wait_for(&self, id: usize) -> T {
        let guard = self.container.lock().unwrap();
        self._wait_for(guard, id)
    }
}
