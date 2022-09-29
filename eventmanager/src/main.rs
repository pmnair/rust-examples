
use eventmanager::*;

fn main() {
    let mut ev_mgr = EventManager::new();

    ev_mgr.subscribe( |e: &Event| {
        println!("Subscriber 1: {:?}", e);
    });

    ev_mgr.subscribe( |e: &Event| {
        println!("Subscriber 2: {:?}", e);
    });

    ev_mgr.publish(Event::One("Hello World".to_string()));
    ev_mgr.publish(Event::Two(&[0xAA, 0xBB, 0xCC]));
    ev_mgr.publish(Event::Three);
}
