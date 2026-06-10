//! Rede LAN — UDP + pacotes serializados para multiplayer.

use crate::game::events::GameEvent;
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::net::{SocketAddr, UdpSocket};
pub const DEFAULT_PORT: u16 = 17777;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetRole {
    Offline,
    Host,
    Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemotePlayer {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub yaw: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetPacket {
    Hello {
        player_id: u64,
        name: String,
    },
    PlayerUpdate {
        id: u64,
        x: f32,
        y: f32,
        z: f32,
        yaw: f32,
    },
    SyncEvents {
        tick: u64,
        events: Vec<GameEvent>,
    },
}

#[derive(Debug)]
pub struct MultiplayerHub {
    pub role: NetRole,
    socket: Option<UdpSocket>,
    pub local_id: u64,
    pub remote_addr: Option<SocketAddr>,
    pub remotes: Vec<RemotePlayer>,
    broadcast_timer: f32,
    pub connected: bool,
}

impl Default for MultiplayerHub {
    fn default() -> Self {
        Self {
            role: NetRole::Offline,
            socket: None,
            local_id: 1,
            remote_addr: None,
            remotes: Vec::new(),
            broadcast_timer: 0.0,
            connected: false,
        }
    }
}

impl MultiplayerHub {
    pub fn from_env() -> Self {
        if std::env::var("MULTIPLAYER_HOST").is_ok() {
            Self::start_host(DEFAULT_PORT)
        } else if let Ok(addr) = std::env::var("MULTIPLAYER_CLIENT") {
            Self::start_client(&addr)
        } else {
            Self::default()
        }
    }

    pub fn start_host(port: u16) -> Self {
        let mut hub = Self::default();
        hub.role = NetRole::Host;
        hub.local_id = 1;
        match UdpSocket::bind(format!("0.0.0.0:{port}")) {
            Ok(sock) => {
                sock.set_nonblocking(true).ok();
                hub.socket = Some(sock);
                hub.connected = true;
                log::info!("Multiplayer HOST na porta {port}");
            }
            Err(e) => log::warn!("Falha ao abrir host UDP: {e}"),
        }
        hub
    }

    pub fn start_client(host: &str) -> Self {
        let mut hub = Self::default();
        hub.role = NetRole::Client;
        hub.local_id = 2;
        let addr = format!("{host}:{DEFAULT_PORT}");
        match UdpSocket::bind("0.0.0.0:0") {
            Ok(sock) => {
                sock.set_nonblocking(true).ok();
                if let Ok(remote) = addr.parse() {
                    hub.remote_addr = Some(remote);
                    let hello = NetPacket::Hello {
                        player_id: hub.local_id,
                        name: "Jogador".into(),
                    };
                    if let Ok(data) = serde_json::to_vec(&hello) {
                        let _ = sock.send_to(&data, remote);
                    }
                }
                hub.socket = Some(sock);
                log::info!("Multiplayer CLIENT -> {addr}");
            }
            Err(e) => log::warn!("Falha ao abrir client UDP: {e}"),
        }
        hub
    }

    pub fn update(&mut self, dt: f32, tick: u64, local: &RemotePlayer, events: &[GameEvent]) {
        self.poll_incoming();
        self.broadcast_timer -= dt;
        if self.broadcast_timer > 0.0 {
            return;
        }
        self.broadcast_timer = 0.1;

        let Some(sock) = &self.socket else { return };

        let update = NetPacket::PlayerUpdate {
            id: local.id,
            x: local.x,
            y: local.y,
            z: local.z,
            yaw: local.yaw,
        };
        if let Ok(data) = serde_json::to_vec(&update) {
            if self.role == NetRole::Host {
                if let Some(addr) = self.remote_addr {
                    let _ = sock.send_to(&data, addr);
                }
            } else if let Some(addr) = self.remote_addr {
                let _ = sock.send_to(&data, addr);
            }
        }

        if self.role == NetRole::Host {
            let sync = NetPacket::SyncEvents {
                tick,
                events: events.iter().rev().take(32).cloned().collect(),
            };
            if let (Ok(data), Some(addr)) = (serde_json::to_vec(&sync), self.remote_addr) {
                let _ = sock.send_to(&data, addr);
            }
        }
    }

    fn poll_incoming(&mut self) {
        let mut buf = [0u8; 65507];
        loop {
            let packet = {
                let Some(sock) = self.socket.as_ref() else { return };
                match sock.recv_from(&mut buf) {
                    Ok((n, from)) => serde_json::from_slice::<NetPacket>(&buf[..n])
                        .ok()
                        .map(|pkt| (pkt, from)),
                    Err(e) if e.kind() == ErrorKind::WouldBlock => None,
                    Err(_) => None,
                }
            };
            match packet {
                Some((pkt, from)) => self.handle_packet(pkt, from),
                None => break,
            }
        }
    }

    fn handle_packet(&mut self, pkt: NetPacket, from: SocketAddr) {
        match pkt {
            NetPacket::Hello { player_id, .. } => {
                self.remote_addr = Some(from);
                self.connected = true;
                log::info!("Jogador {player_id} conectou de {from}");
            }
            NetPacket::PlayerUpdate { id, x, y, z, yaw } => {
                self.remote_addr = Some(from);
                self.connected = true;
                if let Some(r) = self.remotes.iter_mut().find(|r| r.id == id) {
                    r.x = x;
                    r.y = y;
                    r.z = z;
                    r.yaw = yaw;
                } else {
                    self.remotes.push(RemotePlayer { id, x, y, z, yaw });
                }
            }
            NetPacket::SyncEvents { .. } => {
                if self.role == NetRole::Client {
                    self.connected = true;
                }
            }
        }
    }

    pub fn drain_remote_events(&mut self) -> Vec<GameEvent> {
        Vec::new()
    }

    pub fn role_label(&self) -> &'static str {
        match self.role {
            NetRole::Offline => "solo",
            NetRole::Host => "host",
            NetRole::Client => "cliente",
        }
    }
}
