use std::collections::HashMap;

use anyhow::{Context, Result};
use wayland_client::{
    backend::ObjectId,
    event_created_child,
    globals::{registry_queue_init, GlobalListContents},
    protocol::wl_registry,
    Connection, Dispatch, EventQueue, Proxy, QueueHandle,
};

use wayland_protocols_wlr::foreign_toplevel::v1::client::{
    zwlr_foreign_toplevel_handle_v1::{Event as HandleEvent, State as HandleState, ZwlrForeignToplevelHandleV1},
    zwlr_foreign_toplevel_manager_v1::{Event as ManagerEvent, ZwlrForeignToplevelManagerV1},
};

use crate::client::Client;

#[derive(Default, Debug)]
struct State {
    active_window: Option<ObjectId>,
    windows: HashMap<ObjectId, String>,
    titles: HashMap<ObjectId, String>,
}

#[derive(Default)]
pub struct WlRootsClient {
    queue: Option<EventQueue<State>>,
    state: State,
}

impl WlRootsClient {
    pub fn new() -> Self {
        Default::default()
    }

    fn connect(&mut self) -> Result<()> {
        let connection = Connection::connect_to_env()?;
        let (globals, mut queue) = registry_queue_init::<State>(&connection)?;

        globals
            .bind::<ZwlrForeignToplevelManagerV1, _, _>(&queue.handle(), 1..=3, ())
            .context("wlr_foreign_toplevel_management_unstable_v1 protocol is not supported")?;

        queue.roundtrip(&mut self.state)?;

        self.queue = Some(queue);

        Ok(())
    }
}

impl Client for WlRootsClient {
    fn supported(&mut self) -> bool {
        match self.connect() {
            Ok(_) => true,
            Err(err) => {
                eprintln!("{err}");
                false
            }
        }
    }
    fn current_window(&mut self) -> Option<String> {
        let queue = self.queue.as_mut()?;

        if queue.roundtrip(&mut self.state).is_err() {
            // try to reconnect
            if let Err(err) = self.connect() {
                log::error!("{err}");
                return None;
            }

            log::debug!("Reconnected to wayland");
        }

        let id = self.state.active_window.as_ref()?;
        self.state.titles.get(id).cloned()
    }

    fn current_application(&mut self) -> Option<String> {
        let queue = self.queue.as_mut()?;

        if queue.roundtrip(&mut self.state).is_err() {
            // try to reconnect
            if let Err(err) = self.connect() {
                log::error!("{err}");
                return None;
            }

            log::debug!("Reconnected to wayland");
        }

        let id = self.state.active_window.as_ref()?;
        self.state.windows.get(id).cloned()
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for State {
    fn event(
        _state: &mut Self,
        _registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &GlobalListContents,
        _connection: &Connection,
        _: &QueueHandle<State>,
    ) {
        log::trace!("{event:?}");
    }
}

impl Dispatch<ZwlrForeignToplevelManagerV1, ()> for State {
    fn event(
        state: &mut Self,
        _: &ZwlrForeignToplevelManagerV1,
        event: ManagerEvent,
        _: &(),
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
        if let ManagerEvent::Toplevel { toplevel } = event {
            state.windows.insert(toplevel.id(), "<unknown>".into());
            state.titles.insert(toplevel.id(), "<unknown>".into());
        }
    }

    event_created_child!(State, ZwlrForeignToplevelManagerV1, [
        _ => (ZwlrForeignToplevelHandleV1, ())
    ]);
}

impl Dispatch<ZwlrForeignToplevelHandleV1, ()> for State {
    fn event(
        state: &mut Self,
        handle: &ZwlrForeignToplevelHandleV1,
        event: HandleEvent,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            HandleEvent::AppId { app_id } => {
                state.windows.insert(handle.id(), app_id);
            }
            HandleEvent::Title { title } => {
                state.titles.insert(handle.id(), title);
            }
            HandleEvent::Closed => {
                state.windows.remove(&handle.id());
                state.titles.remove(&handle.id());
            }
            HandleEvent::State { state: handle_state } => {
                let activated = HandleState::Activated as u8;
                if handle_state.contains(&activated) {
                    state.active_window = Some(handle.id());
                }
            }
            _ => {}
        }
    }
}
