use crate::client::{Client, WindowInfo};
use anyhow::{bail, Context, Result};
use std::collections::HashMap;
use wayland_client::backend::ObjectId;
use wayland_client::globals::{registry_queue_init, GlobalListContents};
use wayland_client::protocol::wl_registry;
use wayland_client::{event_created_child, Connection, Dispatch, EventQueue, Proxy, QueueHandle};
use wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_handle_v1::{
    Event as HandleEvent, State as HandleState, ZwlrForeignToplevelHandleV1,
};
use wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_manager_v1::{
    Event as ManagerEvent, ZwlrForeignToplevelManagerV1,
};

#[derive(Default, Debug)]
struct State {
    toplevel_handles: HashMap<ObjectId, ZwlrForeignToplevelHandleV1>,
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

    fn borrow<'a>(&'a mut self) -> Result<(&'a mut EventQueue<State>, &'a mut State)> {
        if self.queue.is_none() {
            self.connect()?;
        }

        let state = &mut self.state;
        let queue = self
            .queue
            .as_mut()
            .ok_or_else(|| anyhow::format_err!("This cannot happen"))?;

        queue.roundtrip(state)?;

        Ok((queue, state))
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

    fn window_list(&mut self) -> anyhow::Result<Vec<WindowInfo>> {
        bail!("window_list not implemented for wlroot")
    }

    fn close_windows_by_app_class(&mut self, target_app_class: &str) -> Result<()> {
        let (queue, state) = self.borrow()?;

        for (id, app_class) in &state.windows {
            if app_class == target_app_class {
                let toplevel = state
                    .toplevel_handles
                    .get(id)
                    .ok_or_else(|| anyhow::format_err!("Toplevel should have been set here."))?;
                toplevel.close();
            }
        }

        queue.flush()?; // Ensure it happens right away.

        Ok(())
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for State {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
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
            state.toplevel_handles.insert(toplevel.id(), toplevel);
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
                state.toplevel_handles.remove(&handle.id());
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
