use crate::action::Action;
use crate::event::RelativeEvent;
use evdev::{InputEvent, RelativeAxisCode};

#[allow(unused)]
pub fn print_actions(actions: &Vec<Action>) {
    println!("vec![");
    for action in actions {
        println!("\t{},", format_action(action));
    }
    println!("]");
}

fn format_action(action: &Action) -> String {
    let str = match action {
        Action::KeyEvent(key_event) => {
            let value = match key_event.value() {
                0 => "KeyValue::Release",
                1 => "KeyValue::Press",
                2 => "KeyValue::Repeat",
                _ => panic!("invalid"),
            };

            format!("Action::KeyEvent(KeyEvent::new(Key::{:?}, {}))", key_event.key, value)
        }
        Action::RelativeEvent(relative_event) => {
            format!("Action::RelativeEvent({})", format_relative_event(relative_event, ""))
        }
        Action::MouseMovementEventCollection(relative_events) => {
            format!(
                "Action::MouseMovementEventCollection(\n{}\t\n\t)",
                format_relative_events(relative_events, &format!("\t\t"))
            )
        }
        Action::InputEvent(input_event) => {
            format!("Action::InputEvent({})", format_input_event(input_event))
        }
        Action::Delay(duration) => {
            format!("Action::Delay(Duration::from_nanos({:?}))", duration.as_nanos())
        }
        Action::PopWindowInfo => {
            format!("Action::PopWindowInfo")
        }
        Action::Exit => {
            format!("Action::Exit")
        }
        _ => {
            todo!();
        }
    };

    str
}

fn format_relative_event<'a>(event: &RelativeEvent, indent: &'a str) -> String {
    format!(
        "{indent}RelativeEvent::new_with(RelativeAxisCode::{:?}.0, {})",
        RelativeAxisCode(event.code),
        event.value
    )
}

fn format_relative_events<'a>(events: &Vec<RelativeEvent>, indent: &'a str) -> String {
    let mut str: String = format!("{indent}vec![");
    for event in events {
        str = format!("{str}\n{},", format_relative_event(event, &format!("{indent}\t")));
    }
    format!("{str}\n{indent}]")
}

fn format_input_event<'a>(event: &InputEvent) -> String {
    match event.destructure() {
        evdev::EventSummary::Synchronization(event, code, value) => {
            format!(
                "InputEvent::new(EventType::{:?}.0, SynchronizationCode::{:?}.0, {})",
                event.event_type(),
                code,
                value
            )
        }
        _ => {
            todo!();
        }
    }
}
