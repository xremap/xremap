use crate::client::cosmic_protocols::toplevel_info::v1::client::zcosmic_toplevel_handle_v1::{
    self, State::Activated, ZcosmicToplevelHandleV1,
};
use crate::client::cosmic_protocols::toplevel_info::v1::client::zcosmic_toplevel_info_v1::{
    self, ZcosmicToplevelInfoV1,
};
use crate::client::{Client, WindowInfo};
use anyhow::{Context, Result};
use std::collections::HashMap;
use wayland_client::backend::ObjectId;
use wayland_client::globals::{registry_queue_init, GlobalListContents};
use wayland_client::protocol::wl_registry::{self, WlRegistry};
use wayland_client::{event_created_child, Connection, Dispatch, EventQueue, Proxy, QueueHandle};

// wayland-client api:          https://docs.rs/wayland-client/latest/wayland_client/
// toplevel info protocol:      https://wayland.app/protocols/cosmic-toplevel-info-unstable-v1
// toplevel manager protocol:   https://wayland.app/protocols/cosmic-toplevel-management-unstable-v1

// Cosmic protocols are included in a subfolder, and they come from the official repo.
// The official repo isn't released to crates.io, so the protocols have be included as is.
// The version of wayland_client that is already used by xremap corresponds to this commit:
// https://github.com/pop-os/cosmic-protocols/tree/5b939bff8ff7d3e57a36fa3968d8ad2768f0afd2

#[derive(Debug)]
struct CosmicWindow {
    handle: ZcosmicToplevelHandleV1,
    app_class: Option<String>,
    title: Option<String>,
}

struct State {
    windows: HashMap<ObjectId, CosmicWindow>,
    active_window: Option<ObjectId>,
}

#[derive(Default)]
pub struct CosmicClient {
    queue: Option<EventQueue<State>>,
    state: Option<State>,
}

impl CosmicClient {
    pub fn new() -> Self {
        Default::default()
    }

    fn connect(&mut self) -> Result<()> {
        let connection = Connection::connect_to_env()?;
        let (globals, mut queue) = registry_queue_init::<State>(&connection)?;

        globals
            .bind::<ZcosmicToplevelInfoV1, _, _>(&queue.handle(), 1..=1, ())
            .context("zcosmic_toplevel_info_v1 protocol is not supported")?;

        let mut state = State {
            windows: HashMap::new(),
            active_window: None,
        };

        // Flush so listening starts. Otherwise we would have to wait for the Done event
        // to ensure that we're in a consistent state. But it's easier to just accept that the
        // window info can be partial.
        queue.roundtrip(&mut state)?;

        self.queue = Some(queue);
        self.state = Some(state);

        Ok(())
    }

    fn get_focused_window<'a>(&'a mut self) -> Result<Option<&'a CosmicWindow>> {
        let (_, state) = self.borrow()?;
        Ok(state.active_window.as_ref().and_then(|id| state.windows.get(&id)))
    }

    fn borrow<'a>(&'a mut self) -> Result<(&'a mut EventQueue<State>, &'a mut State)> {
        if self.queue.is_none() {
            self.connect()?;
        }

        let queue = self
            .queue
            .as_mut()
            .ok_or_else(|| anyhow::format_err!("This cannot happen"))?;
        let state = self
            .state
            .as_mut()
            .ok_or_else(|| anyhow::format_err!("This cannot happen"))?;

        queue.roundtrip(state)?;

        Ok((queue, state))
    }
}

impl Client for CosmicClient {
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
        match self.get_focused_window() {
            Ok(window) => window.and_then(|window| window.title.clone()),
            Err(e) => {
                eprintln!("Error when fetching window title: {e:?}");
                None
            }
        }
    }

    fn current_application(&mut self) -> Option<String> {
        match self.get_focused_window() {
            Ok(window) => window.and_then(|window| window.app_class.clone()),
            Err(e) => {
                eprintln!("Error when fetching app_id: {e:?}");
                None
            }
        }
    }

    fn window_list(&mut self) -> Result<Vec<WindowInfo>> {
        let (_, state) = self.borrow()?;

        let windows: Vec<WindowInfo> = state
            .windows
            .iter()
            .map(
                |(
                    _,
                    CosmicWindow {
                        handle,
                        app_class,
                        title,
                    },
                )| WindowInfo {
                    winid: Some(format!("{}", handle.id())),
                    app_class: app_class.clone(),
                    title: title.clone(),
                },
            )
            .collect();

        Ok(windows)
    }

    fn close_windows_by_app_class(&mut self, _app_class: &str) -> Result<()> {
        todo!()
    }
}

impl Dispatch<WlRegistry, GlobalListContents> for State {
    fn event(
        _: &mut Self,
        _: &WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
    }
}

impl Dispatch<ZcosmicToplevelInfoV1, ()> for State {
    fn event(
        state: &mut Self,
        _: &ZcosmicToplevelInfoV1,
        event: zcosmic_toplevel_info_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
        use zcosmic_toplevel_info_v1::Event::{Finished, Toplevel};

        match event {
            Toplevel { toplevel } => {
                let info = CosmicWindow {
                    handle: toplevel,
                    app_class: None,
                    title: None,
                };
                state.windows.insert(info.handle.id(), info);
            }
            Finished => {}
        }
    }

    event_created_child!(
        State,
        ZcosmicToplevelInfoV1,
        [
            zcosmic_toplevel_info_v1::EVT_TOPLEVEL_OPCODE => (ZcosmicToplevelHandleV1, ()),
        ]
    );
}

impl Dispatch<ZcosmicToplevelHandleV1, ()> for State {
    fn event(
        state: &mut Self,
        handle: &ZcosmicToplevelHandleV1,
        event: zcosmic_toplevel_handle_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
        use zcosmic_toplevel_handle_v1::Event::{AppId, Closed, State, Title};

        match event {
            Title { title } => {
                state
                    .windows
                    .get_mut(&handle.id())
                    .map(|window| window.title = Some(title));
            }
            AppId { app_id } => {
                state
                    .windows
                    .get_mut(&handle.id())
                    .map(|window| window.app_class = Some(app_id));
            }
            State { state: window_state } => {
                let (chunks, _) = window_state.as_chunks::<4>();
                let activated = chunks
                    .iter()
                    .map(|&chunk| u32::from_ne_bytes(chunk))
                    .any(|state| state == Activated as u32);

                if activated {
                    state.active_window = Some(handle.id())
                }
            }
            Closed => {
                state.windows.remove(&handle.id());
            }
            _ => {}
        }
    }
}
