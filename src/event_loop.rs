use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

type Callback = Box<dyn Fn() -> Result<(), ()>>;

pub struct EventLoop {
    callbacks: HashMap<String, Callback>,
    is_running: AtomicBool,
    delay: Duration,
}

impl EventLoop {
    pub fn new(delay: Duration) -> EventLoop {
        EventLoop {
            callbacks: HashMap::new(),
            is_running: AtomicBool::default(),
            delay,
        }
    }

    pub fn start(&mut self) {
        self.is_running.store(true, Ordering::Relaxed);

        while self.is_running.load(Ordering::SeqCst) == true {
            if self.callbacks.len() > 0 {
                for (_, callback) in self.callbacks.iter() {
                    if let Err(_) = callback() {
                        return;
                    }
                }
            }

            thread::sleep(self.delay);
        }
    }

    pub fn add_callback(&mut self, name: &str, callback: Callback) -> bool {
        if !self.callbacks.contains_key(name) {
            self.callbacks.insert(name.to_string(), callback);
            return true;
        }

        false
    }

    pub fn remove_all_callbacks(&mut self) -> bool {
        if self.callbacks.len() > 0 {
            self.callbacks.clear();
            return true;
        }

        false
    }

    pub fn stop(&mut self) {
        self.is_running.store(false, Ordering::Relaxed);
    }
}
