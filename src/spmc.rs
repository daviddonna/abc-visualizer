use std::sync::{Mutex, MutexGuard, Condvar};

/// Deliver fitnesses to the threads that requested them.
///
/// This is essentially a single-producer, multiple-consumer queue,
/// where each consumer is waiting for an item with a specific ID.
/// Each time a new item is produced, the consumers all wake up and
/// examine it, and the producer who was waiting for that item's ID
/// actually consumes the item.
pub struct Queue<T: Clone + Send> {
    container: Mutex<Option<(usize, T)>>,
    produced: Condvar,
    consumed: Condvar,
}

impl<T: Clone + Send> Queue<T> {
    pub fn new() -> Queue<T> {
        Queue {
            container: Mutex::new(None),
            produced: Condvar::new(),
            consumed: Condvar::new(),
        }
    }

    pub fn send(&self, id: usize, item: T) {
        let mut guard = self.container.lock().unwrap();
        while guard.is_some() {
            guard = self.consumed.wait(guard).unwrap();
        }
        *guard = Some((id, item));
        self.produced.notify_all()
    }

    fn _wait_for(&self, mut guard: MutexGuard<Option<(usize, T)>>, id: usize) -> T {
        let found = guard.as_ref().map_or(None, |&(id2, ref item)| {
            if id2 == id {
                Some(item.clone())
            } else {
                None
            }
        });
        match found {
            Some(item) => {
                *guard = None;
                self.consumed.notify_one();
                item
            }
            None => self._wait_for(self.produced.wait(guard).unwrap(), id),
        }
    }

    pub fn wait_for(&self, id: usize) -> T {
        let guard = self.container.lock().unwrap();
        self._wait_for(guard, id)
    }
}
