use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub(crate) struct TaskControl {
    active: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
}

impl TaskControl {
    pub(crate) fn new() -> Self {
        Self {
            active: Arc::new(AtomicBool::new(false)),
            cancelled: Arc::new(AtomicBool::new(false)),
            paused: Arc::new(AtomicBool::new(false)),
        }
    }

    pub(crate) fn start(&self) -> bool {
        if self.active.load(Ordering::SeqCst) {
            return false;
        }
        self.active.store(true, Ordering::SeqCst);
        self.cancelled.store(false, Ordering::SeqCst);
        self.paused.store(false, Ordering::SeqCst);
        true
    }

    pub(crate) fn finish(&self) {
        self.active.store(false, Ordering::SeqCst);
        self.cancelled.store(false, Ordering::SeqCst);
        self.paused.store(false, Ordering::SeqCst);
    }

    pub(crate) fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub(crate) fn pause(&self) {
        self.paused.store(true, Ordering::SeqCst);
    }

    pub(crate) fn resume(&self) {
        self.paused.store(false, Ordering::SeqCst);
    }

    pub(crate) fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    pub(crate) fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }

    pub(crate) fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }
}
