use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::Instant;

use crate::net_packet::NetPacket;

pub const NET_MAXPLAYERS: usize = 8;
pub const MAXPLAYERNAME: usize = 30;
pub const BACKUPTICS: usize = 128;
pub const NET_MAGIC_NUMBER: u32 = 1454104972;

// TicDiff Flags
pub const NET_TICDIFF_FORWARD: u32 = 1 << 0;
pub const NET_TICDIFF_SIDE: u32 = 1 << 1;
pub const NET_TICDIFF_TURN: u32 = 1 << 2;
pub const NET_TICDIFF_BUTTONS: u32 = 1 << 3;
pub const NET_TICDIFF_CONSISTANCY: u32 = 1 << 4;
pub const NET_TICDIFF_CHATCHAR: u32 = 1 << 5;
pub const NET_TICDIFF_RAVEN: u32 = 1 << 6;
pub const NET_TICDIFF_STRIFE: u32 = 1 << 7;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct TicCmd {
    pub forwardmove: i8,
    pub sidemove: i8,
    pub angleturn: i16,
    pub chatchar: u8,
    pub buttons: u8,
    pub consistancy: u8,
    pub buttons2: u8,
    pub inventory: i32,
    pub lookfly: u8,
    pub arti: u8,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct ConnectData {
    pub gamemode: i32,
    pub gamemission: i32,
    pub lowres_turn: i32,
    pub drone: i32,
    pub max_players: i32,
    pub is_freedoom: i32,
    pub wad_sha1sum: [u8; 20],
    pub deh_sha1sum: [u8; 20],
    pub player_class: i32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct GameSettings {
    pub ticdup: i32,
    pub extratics: i32,
    pub deathmatch: i32,
    pub episode: i32,
    pub nomonsters: i32,
    pub fast_monsters: i32,
    pub respawn_monsters: i32,
    pub map: i32,
    pub skill: i32,
    pub gameversion: i32,
    pub lowres_turn: i32,
    pub new_sync: i32,
    pub timelimit: u32,
    pub loadgame: i32,
    pub random: i32,
    pub num_players: i32,
    pub consoleplayer: i32,
    pub player_classes: [i32; NET_MAXPLAYERS],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetPacketType {
    Syn,
    Ack,
    Rejected,
    KeepAlive,
    WaitingData,
    GameStart,
    GameData,
    GameDataAck,
    Disconnect,
    DisconnectAck,
    ReliableAck,
    GameDataResend,
    ConsoleMessage,
    Query,
    QueryResponse,
    Launch,
}

impl TryFrom<u16> for NetPacketType {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(NetPacketType::Syn),
            1 => Ok(NetPacketType::Ack),
            2 => Ok(NetPacketType::Rejected),
            3 => Ok(NetPacketType::KeepAlive),
            4 => Ok(NetPacketType::WaitingData),
            5 => Ok(NetPacketType::GameStart),
            6 => Ok(NetPacketType::GameData),
            7 => Ok(NetPacketType::GameDataAck),
            8 => Ok(NetPacketType::Disconnect),
            9 => Ok(NetPacketType::DisconnectAck),
            10 => Ok(NetPacketType::ReliableAck),
            11 => Ok(NetPacketType::GameDataResend),
            12 => Ok(NetPacketType::ConsoleMessage),
            13 => Ok(NetPacketType::Query),
            14 => Ok(NetPacketType::QueryResponse),
            15 => Ok(NetPacketType::Launch),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct NetTicDiff {
    pub diff: u32,
    pub cmd: TicCmd,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct NetFullTicCmd {
    pub latency: i32,
    pub seq: u32,
    pub playeringame: [bool; NET_MAXPLAYERS],
    pub cmds: [NetTicDiff; NET_MAXPLAYERS],
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NetQueryData {
    pub version: String,
    pub server_state: i32,
    pub num_players: i32,
    pub max_players: i32,
    pub gamemode: i32,
    pub gamemission: i32,
    pub description: String,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct NetWaitData {
    pub num_players: i32,
    pub num_drones: i32,
    pub ready_players: i32,
    pub max_players: i32,
    pub is_controller: i32,
    pub consoleplayer: i32,
    pub player_names: [[char; MAXPLAYERNAME]; NET_MAXPLAYERS],
    pub player_addrs: [[char; MAXPLAYERNAME]; NET_MAXPLAYERS],
    pub wad_sha1sum: [u8; 20],
    pub deh_sha1sum: [u8; 20],
    pub is_freedoom: i32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClientState {
    #[default]
    Disconnected,
    WaitingLaunch,
    WaitingStart,
    InGame,
    DisconnectedSleep,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetConnection {
    pub state: ConnectionState,
    pub addr: SocketAddr,
}

impl NetConnection {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            state: ConnectionState::Disconnected,
            addr,
        }
    }
}

impl Default for NetConnection {
    fn default() -> Self {
        Self {
            state: ConnectionState::default(),
            addr: "127.0.0.1:8080".parse().unwrap(),
        }
    }
}

#[derive(Clone)]
pub struct NetServerRecv {
    pub active: bool,
    pub resend_time: Instant,
    pub cmd: NetFullTicCmd,
}

impl Default for NetServerRecv {
    fn default() -> Self {
        Self {
            active: false,
            resend_time: Instant::now(),
            cmd: Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct NetServerSend {
    pub active: bool,
    pub seq: u32,
    pub time: Instant,
    pub cmd: NetTicDiff,
}

impl Default for NetServerSend {
    fn default() -> Self {
        Self {
            active: false,
            seq: 0,
            time: Instant::now(),
            cmd: Default::default(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    #[default]
    Disconnected,
    Connecting,
    Connected,
}

#[derive(Debug, Clone)]
pub struct SendQueueEntry {
    pub active: bool,
    pub seq: u32,
    pub time: Instant,
    pub cmd: NetTicDiff,
}
