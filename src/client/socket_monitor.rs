use anyhow::{anyhow, Result};
use futures_util::stream::StreamExt;
use log::{debug, info, warn};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use zbus::fdo::DBusProxy;
use zbus::zvariant::{OwnedObjectPath, Value};
use zbus::{Connection, MatchRule, Message, MessageStream};

pub struct SessionMonitor {
    socket_path: String,
    sessions: Mutex<Sessions>,
    connection: OnceLock<Connection>,
    proxy: OnceLock<DBusProxy<'static>>,
}

impl SessionMonitor {
    pub fn new(socket_path: String) -> Self {
        SessionMonitor {
            socket_path,
            sessions: Mutex::new(Sessions::new()),
            connection: OnceLock::new(),
            proxy: OnceLock::new(),
        }
    }

    pub fn get_active_session(&self) -> Option<Session> {
        self.sessions.lock().unwrap().get_active_session()
    }

    pub async fn run(&self) -> Result<()> {
        let session_new_rule = MatchRule::builder()
            .msg_type(zbus::message::Type::Signal)
            .interface("org.freedesktop.login1.Manager")?
            .member("SessionNew")?
            .build();
        let session_removed_rule = MatchRule::builder()
            .msg_type(zbus::message::Type::Signal)
            .interface("org.freedesktop.login1.Manager")?
            .member("SessionRemoved")?
            .build();
        let connection = Connection::system().await?;
        let proxy = DBusProxy::new(&connection).await?;
        let connection = self.connection.get_or_init(|| connection);
        let proxy = self.proxy.get_or_init(|| proxy);

        proxy.add_match_rule(session_new_rule).await?;
        proxy.add_match_rule(session_removed_rule).await?;

        debug!("Monitoring user sessions...");
        if let Err(why) = self.monitor_existing_sessions().await {
            warn!("Cannot monitor existing sessions: {}", why)
        };
        while let Some(msg) = MessageStream::from(connection).next().await {
            match msg {
                Ok(message) => {
                    if let Err(handle_err) = self.handle_message(&message).await {
                        warn!("Could not handle {:?}: {}", message, handle_err)
                    }
                }
                Err(why) => warn!("Message fail: {}", why),
            };
        }
        Ok(())
    }

    async fn handle_message(&self, message: &Message) -> Result<()> {
        let header = message.header();
        let member = match header.member() {
            Some(m) => m,
            None => return Ok(()), // ignore null member
        };
        match member.as_str() {
            "SessionNew" => {
                let (session_id, session_path): (String, OwnedObjectPath) = message.body().deserialize()?;
                self.handle_new_session(&session_id, &session_path).await?;
            }
            "PropertiesChanged" => {
                let header = message.header();
                let path_ref = header.path().ok_or_else(|| anyhow::anyhow!("No path in message"))?;
                let session_path: OwnedObjectPath = path_ref.clone().into();
                let body = message.body();
                let (_name, changed, _invalidated): (String, HashMap<String, Value<'_>>, Vec<String>) =
                    body.deserialize()?;
                self.handle_properties_changed(session_path, changed).await;
            }
            "SessionRemoved" => {
                let (session_id, session_path): (String, OwnedObjectPath) = message.body().deserialize()?;
                let (session, active) = self.sessions.lock().unwrap().remove(&session_path);
                if session.is_some() || active.is_some() {
                    self.remove_properties_changed_match_rule(&session_path).await?;
                    if session.is_some() {
                        info!("Removed session {}", session_id);
                    } else if active.is_some() {
                        warn!("Discarded unknown active session {}", session_id);
                    }
                }
            }
            sig => warn!("Ignored message: {}", sig),
        };
        Ok(())
    }

    async fn handle_new_session(&self, session_id: &String, session_path: &OwnedObjectPath) -> Result<()> {
        let connection = self.connection.get().ok_or(anyhow!("not connected"))?;
        let session_proxy = SessionProxy::builder(&connection).path(session_path)?.build().await?;
        let (seat_id, _seat_path) = session_proxy.seat().await?;
        if seat_id.is_empty() {
            debug!("Ignoring unseated session {}", session_id);
            return Ok(());
        }
        let (uid, _user_path) = session_proxy.user().await?;
        let is_active = session_proxy.active().await?;
        let session = Session {
            id: session_id.clone(),
            user_socket: user_socket_path(&self.socket_path, uid),
        };
        let active_str = if is_active { " active" } else { "" };
        info!("Monitoring{} session {} (uid={}, seat={})", active_str, session_id, uid, seat_id);
        self.add_properties_changed_match_rule(&session_path).await?;
        self.sessions
            .lock()
            .unwrap()
            .insert(session_path.clone(), session, is_active);
        Ok(())
    }

    async fn handle_properties_changed(&self, session_path: OwnedObjectPath, changed: HashMap<String, Value<'_>>) {
        if let Some(Value::Bool(is_active)) = changed.get("Active") {
            let mut sessions = self.sessions.lock().unwrap();
            if *is_active {
                if let Some(session) = sessions.activate(session_path) {
                    debug!("Activated session {}", session.id);
                }
            } else {
                if let Some(session) = sessions.deactivate(session_path) {
                    debug!("Deactivated session {}", session.id);
                }
            }
        }
    }

    async fn monitor_existing_sessions(&self) -> Result<()> {
        let connection = self.connection.get().ok_or(anyhow!("not connected"))?;
        let manager = ManagerProxy::new(&connection).await?;
        let session_list = manager.list_sessions().await?;
        for (session_id, uid, _user, seat_id, session_path) in session_list {
            if seat_id.is_empty() {
                continue;
            }
            info!("Existing session: {} (uid={}, seat={})", session_id, uid, seat_id);
            let session_proxy = SessionProxy::builder(&connection).path(&session_path)?.build().await?;
            let is_active = session_proxy.active().await?;
            let session = Session {
                id: session_id.clone(),
                user_socket: user_socket_path(&self.socket_path, uid),
            };
            self.sessions
                .lock()
                .unwrap()
                .insert(session_path.clone(), session, is_active);
            self.add_properties_changed_match_rule(&session_path).await?
        }
        Ok(())
    }

    async fn add_properties_changed_match_rule(&self, session_path: &OwnedObjectPath) -> Result<()> {
        let proxy = self.proxy.get().ok_or(anyhow!("not connected"))?;
        let rule = session_changed_rule(session_path)?;
        Ok(proxy.add_match_rule(rule).await?)
    }

    async fn remove_properties_changed_match_rule(&self, session_path: &OwnedObjectPath) -> Result<()> {
        let proxy = self.proxy.get().ok_or(anyhow!("not connected"))?;
        let rule = session_changed_rule(session_path)?;
        Ok(proxy.remove_match_rule(rule).await?)
    }
}

fn session_changed_rule(session_path: &OwnedObjectPath) -> Result<MatchRule<'_>> {
    Ok(MatchRule::builder()
        .msg_type(zbus::message::Type::Signal)
        .interface("org.freedesktop.DBus.Properties")?
        .path(session_path)?
        .member("PropertiesChanged")?
        .build())
}

fn user_socket_path(path: &String, uid: u32) -> PathBuf {
    PathBuf::from(path.replace("{uid}", &uid.to_string()))
}

#[derive(Clone, Debug)]
pub struct Session {
    pub id: String,
    pub user_socket: PathBuf,
}

struct Sessions {
    sessions: HashMap<OwnedObjectPath, Session>,
    active_sessions: HashMap<OwnedObjectPath, Session>,
}

impl Sessions {
    fn new() -> Sessions {
        Sessions {
            sessions: HashMap::new(),
            active_sessions: HashMap::new(),
        }
    }

    fn get_active_session(&self) -> Option<Session> {
        let active = &self.active_sessions;
        if active.len() > 1 {
            warn!("Unexpected: multiple active sessions: {:?}", active.keys());
            return None;
        }
        active.values().next().cloned()
    }

    fn remove(&mut self, path: &OwnedObjectPath) -> (Option<Session>, Option<Session>) {
        let session = self.sessions.remove(path);
        let active = self.active_sessions.remove(path);
        (session, active)
    }

    fn insert(&mut self, path: OwnedObjectPath, session: Session, is_active: bool) {
        self.sessions.insert(path.clone(), session.clone());
        if is_active {
            self.active_sessions.insert(path, session);
        }
    }

    fn activate(&mut self, path: OwnedObjectPath) -> Option<Session> {
        if let Some(session) = self.sessions.get(&path) {
            self.active_sessions.insert(path, session.clone());
            return Some(session.clone());
        }
        None
    }

    fn deactivate(&mut self, path: OwnedObjectPath) -> Option<Session> {
        self.active_sessions.remove(&path)
    }
}

#[zbus::proxy(
    interface = "org.freedesktop.login1.Manager",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1"
)]
trait Manager {
    fn list_sessions(&self) -> zbus::Result<Vec<(String, u32, String, String, OwnedObjectPath)>>;
}

#[zbus::proxy(
    interface = "org.freedesktop.login1.Session",
    default_service = "org.freedesktop.login1"
)]
trait Session {
    #[zbus(property)]
    fn seat(&self) -> zbus::Result<(String, OwnedObjectPath)>;

    #[zbus(property)]
    fn user(&self) -> zbus::Result<(u32, OwnedObjectPath)>;

    #[zbus(property)]
    fn active(&self) -> zbus::Result<bool>;
}
