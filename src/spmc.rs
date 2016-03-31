use std::sync::{Mutex, MutexGuard, Condvar};
use std::collections::BTreeMap;

/// Deliver fitnesses to the threads that requested them.
///
/// This is essentially a single-producer, multiple-consumer queue,
/// where each consumer is waiting for an item with a specific ID.
/// Each time a new item is produced, the consumers all wake up and
/// examine it, and the producer who was waiting for that item's ID
/// actually consumes the item.
pub struct Queue<T> {
    container: Mutex<BTreeMap<usize, T>>,
    produced: Condvar,
}

impl<T> Queue<T> {
    pub fn new() -> Queue<T> {
        Queue {
            container: Mutex::new(BTreeMap::new()),
            produced: Condvar::new(),
        }
    }

    pub fn send(&self, id: usize, item: T) {
        // The guard is only ever held inside this code,
        // none of which is known to be able to panic and poison the lock.
        let mut guard = self.container.lock().unwrap();
        guard.insert(id, item);
        self.produced.notify_all()
    }

    fn _wait_for(&self, mut guard: MutexGuard<BTreeMap<usize, T>>, id: &usize) -> T {
        match guard.remove(id) {
            Some(item) => item,
            None => self._wait_for(self.produced.wait(guard).unwrap(), id)
        }
    }

    pub fn wait_for(&self, id: usize) -> T {
        let guard = self.container.lock().unwrap();
        self._wait_for(guard, &id)
    }
}
