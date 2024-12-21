use crate::events::event::Event;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct BatchEventProcessor {
    batch_size: usize,
    event_queue: Arc<Mutex<VecDeque<Event>>>,
}

const MIN_BATCH_SIZE: usize = 100;
const MAX_BATCH_SIZE: usize = 10_000;

impl BatchEventProcessor {
    pub fn new(batch_size: usize) -> Self {
        // clamp batch size between min and max
        BatchEventProcessor {
            batch_size: batch_size.clamp(MIN_BATCH_SIZE, MAX_BATCH_SIZE),
            event_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn push(&self, event: Event) {
        let mut queue = self.event_queue.lock().unwrap();
        queue.push_back(event);
    }

    pub fn next_batch(&self) -> Vec<Event> {
        let mut queue = self.event_queue.lock().unwrap();
        let mut batch = vec![];
        while let Some(event) = queue.pop_front() {
            batch.push(event);
            if batch.len() >= self.batch_size {
                break;
            }
        }
        batch
    }

    pub fn is_empty(&self) -> bool {
        let queue = self.event_queue.lock().unwrap();
        queue.is_empty()
    }
}
