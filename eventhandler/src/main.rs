
use eventhandler::*;

fn main() {
    let ev_mgr = EventHandler::new(|event: Event| {
        match event {
            Event::One(s) => println!("Event::One: {}", s),
            Event::Two(d) => println!("Event::Two: {:x?}", d),
            Event::Three => println!("Event::Three"),
        }
    });

    ev_mgr.send(Event::One("Hello World".to_string()));
    ev_mgr.send(Event::Two(&[1, 2, 3]));
    ev_mgr.send(Event::Three);
}
