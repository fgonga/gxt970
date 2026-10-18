use gxt970::device::Mouse;
use gxt970::general::General;
use gxt970::protocol::{self, ButtonMap};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

pub enum Request {
    Connect,
    Load(u8),
    Apply { profile: u8, buttons: Option<ButtonMap>, settings: Option<Box<General>> },
    Activate(u8),
}

pub struct Snapshot {
    pub profile: u8,
    pub buttons: ButtonMap,
    pub settings: General,
    pub active: Option<u8>,
}

pub enum Reply {
    Connected { profiles: Vec<u8>, active: Option<u8> },
    Loaded(Snapshot),
    Applied(Snapshot),
    Activated(u8),
    Failed(String),
}

pub struct Worker {
    tx: Sender<Request>,
    rx: Receiver<Reply>,
}

impl Worker {
    pub fn spawn(ctx: egui::Context) -> Self {
        let (tx, requests) = channel::<Request>();
        let (replies, rx) = channel::<Reply>();
        thread::spawn(move || {
            let mut mouse: Option<Mouse> = None;
            for request in requests {
                let reply = handle(&mut mouse, request);
                if replies.send(reply).is_err() {
                    break;
                }
                ctx.request_repaint();
            }
        });
        Self { tx, rx }
    }

    pub fn send(&self, request: Request) {
        let _ = self.tx.send(request);
    }

    pub fn poll(&self) -> Option<Reply> {
        self.rx.try_recv().ok()
    }
}

fn snapshot(mouse: &Mouse, profile: u8) -> anyhow::Result<Snapshot> {
    Ok(Snapshot {
        profile,
        buttons: protocol::read_buttons(mouse, profile)?,
        settings: protocol::read_settings(mouse, profile)?,
        active: protocol::active_profile(mouse).ok(),
    })
}

fn handle(slot: &mut Option<Mouse>, request: Request) -> Reply {
    if matches!(request, Request::Connect) {
        *slot = None;
    }
    if slot.is_none() {
        match Mouse::open() {
            Ok(m) => *slot = Some(m),
            Err(e) => return Reply::Failed(e.to_string()),
        }
    }
    let Some(mouse) = slot.as_ref() else {
        return Reply::Failed("rato não ligado".into());
    };
    let result = match request {
        Request::Connect => Ok(Reply::Connected {
            profiles: protocol::available_profiles(mouse),
            active: protocol::active_profile(mouse).ok(),
        }),
        Request::Load(p) => snapshot(mouse, p).map(Reply::Loaded),
        Request::Apply { profile, buttons, settings } => (|| {
            if let Some(map) = buttons {
                protocol::write_buttons(mouse, profile, &map)?;
            }
            if let Some(general) = settings {
                protocol::write_settings(mouse, profile, general.as_ref())?;
            }
            protocol::activate_profile(mouse, profile)?;
            snapshot(mouse, profile)
        })()
        .map(Reply::Applied),
        Request::Activate(p) => protocol::activate_profile(mouse, p).map(|_| Reply::Activated(p)),
    };
    result.unwrap_or_else(|e| {
        *slot = None;
        Reply::Failed(e.to_string())
    })
}
