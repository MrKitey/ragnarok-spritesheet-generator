use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};

pub struct BusyJob<T> {
    rx: Receiver<T>,
    pub label: String,
}

impl<T> BusyJob<T> {
    pub fn start(label: impl Into<String>) -> (Self, Sender<T>) {
        let (tx, rx) = mpsc::channel();
        let label = label.into();
        (Self { rx, label }, tx)
    }

    pub fn try_poll(&self) -> Result<T, TryRecvError> {
        self.rx.try_recv()
    }
}
