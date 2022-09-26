use std::thread;
use std::sync::mpsc::{self, Sender, Receiver};

/// Generic Event Manager
///
/// A generic implementation of event manager that can be used with
/// custom event definition.
///
/// ```
/// use eventmanager::EventManager;
/// #[derive(Debug)]
/// enum Event {
///     String(&'static str),
///     Bytes(&'static [u8]),
///     Empty
/// }
///
/// let ev_mgr = EventManager::new( |ev: Event| {
///     match ev {
///         Event::String(s) => {
///             println!("Event: String \"{}\"", s)
///         }
///         Event::Bytes(b) => {
///             println!("Event: bytes {:x?}", b)
///         }
///         Event::Empty => {
///             println!("Event: Empty")
///         }
///     }
/// });
///
/// ev_mgr.send(Event::String("Hello World"));
/// ev_mgr.send(Event::Bytes(&[0xAA, 0xBB, 0xCC]));
/// ev_mgr.send(Event::Empty);
/// ```
///
pub struct EventManager<T> {
    thread: Option<thread::JoinHandle<()>>,
    sender: Option<Sender<T>>
}

impl <T: Sync + Send + 'static>EventManager<T> {
    /// Create a new event manager with handler function
    pub fn new<F>(handler: F) -> Self
        where F: Fn(T) + Send + 'static,
                T: Send + 'static
    {
        // create event channel
        let (tx, rx): (Sender<T>, Receiver<T>) = mpsc::channel();

        // start handler trhead
        let thread = thread::spawn( move || {
            println!("Event Manager ready..");
            loop {
                // wait, read and process events
                match rx.recv() {
                    Ok(event) => {
                        #[cfg(Debug)]
                        println!("Handling event..");
                        handler(event);
                    }
                    Err(e) => {
                        eprintln!("Event Manager exiting.. {}", e);
                        break;
                    }
                }
            }
        });

        EventManager{ thread: Some(thread), sender: Some(tx) }
    }

    /// Send event to event manager
    pub fn send(&self, event: T)
    {
        self.sender.as_ref().unwrap().send(event).unwrap();
    }

}

/// Graceful shutdown and cleanup
impl <T>Drop for EventManager<T> {
    fn drop(&mut self) {
        // Close the channel
        drop(self.sender.take());
        // wait for handler to exit
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    enum TestEvent {
        TestString(String),
        TestRaw(&'static [u8]),
        TestEmpty
    }
    #[test]
    fn test_eventmgr() {
        let ev_mgr = EventManager::new(|event: TestEvent| {
            match event {
                TestEvent::TestString(s) => println!("TestString: {}", s),
                TestEvent::TestRaw(d) => println!("TestRaw: {:x?}", d),
                TestEvent::TestEmpty => println!("TestEmpty"),
            }
        });

        ev_mgr.send(TestEvent::TestString("Hello World".to_string()));
        ev_mgr.send(TestEvent::TestRaw(&[1, 2, 3]));
        ev_mgr.send(TestEvent::TestEmpty);
    }
}