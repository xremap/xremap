use crate::event::Event;
use crate::event_handler::{PRESS, RELEASE, REPEAT};

#[allow(unused)]
pub fn print_events(events: &Vec<Event>) {
    println!("vec![");
    for event in events {
        println!("\t{},", format_event(event));
    }
    println!("]");
}

fn format_event(event: &Event) -> String {
    match event {
        Event::KeyEvent(_, key_event) => {
            let value = match key_event.value() {
                PRESS => "key_press",
                RELEASE => "key_release",
                REPEAT => "key_repeat",
                _ => unreachable!(),
            };
            format!("Event::{value}(Key::{:?})", key_event.key)
        }
        _ => {
            unreachable!()
        }
    }
}
