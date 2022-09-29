use std::thread;
use std::sync::{mpsc, Arc, Mutex};

/// Generic Event Handler
///
/// A generic implementation of event handler that can be used with
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
/// let mut ev_mgr = EventManager::new();
///
/// ev_mgr.subscribe( |e: &Event| {
///     println!("Subscriber 1: {:?}", e);
/// });
///
/// ev_mgr.subscribe( |e: &Event| {
///     println!("Subscriber 2: {:?}", e);
/// });
///
/// ev_mgr.publish(Event::String("Hello World"));
/// ev_mgr.publish(Event::Bytes(&[0xAA, 0xBB, 0xCC]));
/// ev_mgr.publish(Event::Empty);
/// ```
///

pub struct EventManager<T> {
    thread: Option<thread::JoinHandle<()>>,
    channel: Option<mpsc::Sender<T>>,
    subscribers: Arc<Mutex<Vec<Subscriber<T>>>>
}

type Subscriber<T> = Box<dyn Fn(&T) + Send + Sync + 'static>;

impl <T: Sync + Send + 'static>EventManager<T> {
    /// Create a new event manager with handler function
    pub fn new() -> Self {
        // create event channel
        let (tx, rx): (mpsc::Sender<T>, mpsc::Receiver<T>) = mpsc::channel();
        let subs: Vec<Subscriber<T>> = Vec::new();
        let subs = Arc::new(Mutex::new(subs));
        let list = Arc::clone(&subs);
        // start handler trhead
        let thread = thread::spawn( move || {
            println!("Event Manager ready..");
            loop {
                // wait, read and process events
                match rx.recv() {
                    Ok(event) => {
                        #[cfg(Debug)]
                        println!("Handling event..");
                        // lock the list and send event to all handlers
                        match list.lock() {
                            Ok(list) => {
                                for s in list.as_slice().into_iter() {
                                    s(&event);
                                }
                            },
                            Err(e) => eprintln!("{}", e),
                        }
                    }
                    Err(e) => {
                        eprintln!("Event Manager exiting.. {}", e);
                        break;
                    }
                }
            }
        });

        EventManager{ thread: Some(thread), channel: Some(tx), subscribers: subs }
    }

    /// Subscribe for events
    ///
    /// Registger event handler with this event manager
    /// to recieve events
    pub fn subscribe<F>(&mut self, s: F)
        where F: Fn(&T) + Send + Sync + 'static
    {
        self.subscribers.lock().unwrap().push(Box::new(s));
    }

    /// Send event to event manager
    pub fn publish(&self, event: T) {
        self.channel.as_ref().unwrap().send(event).unwrap();
    }

}

/// Graceful shutdown and cleanup
impl <T>Drop for EventManager<T> {
    fn drop(&mut self) {
        // Close the channel
        drop(self.channel.take());
        // wait for handler to exit
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    enum TestEvent {
        TestString(String),
        TestRaw(&'static [u8]),
        TestEmpty
    }
    #[test]
    fn test_eventmgr() {
        let mut evmgr = EventManager::new();

        evmgr.subscribe( |e: &TestEvent| {
            println!("Subscriber 1: {:?}", e);
        });

        evmgr.subscribe( |e: &TestEvent| {
            println!("Subscriber 2: {:?}", e);
        });

        evmgr.subscribe( |e: &TestEvent| {
            println!("Subscriber 3: {:?}", e);
        });

        evmgr.publish(TestEvent::TestString("Hello World".to_string()));
        evmgr.publish(TestEvent::TestRaw(&[1, 2, 3]));
        evmgr.publish(TestEvent::TestEmpty);
    }
}