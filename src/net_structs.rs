use serde::{Deserialize, Serialize};

use crate::net_packet::NetPacket;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct TicCmd {
    pub forwardmove: i8, // Movement forward/backward
    pub sidemove: i8,    // Movement sideways
    pub angleturn: i16,  // Angle change
    pub chatchar: u8,    // Chat character
    pub buttons: u8,     // Button states
    pub consistancy: u8, // Consistency check
    pub buttons2: u8,    // Additional button states
    pub inventory: i32,  // Inventory state
    pub lookfly: u8,     // Look/fly direction
    pub arti: u8,        // Artifact type
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct ConnectData {
    pub gamemode: u8,
    pub gamemission: u8,
    pub lowres_turn: u8,
    pub drone: u8,
    pub max_players: u8,
    pub is_freedoom: u8,
    pub wad_sha1sum: [u8; 20],
    pub deh_sha1sum: [u8; 20],
    pub player_class: u8,
}

// Define other necessary structures and enums
// For example, net_gamesettings_t equivalent:

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GameSettings {
    pub ticdup: u8,
    pub extratics: u8,
    pub deathmatch: u8,
    pub nomonsters: u8,
    pub fast_monsters: u8,
    pub respawn_monsters: u8,
    pub episode: u8,
    pub map: u8,
    pub skill: i8,
    pub gameversion: u8,
    pub lowres_turn: u8,
    pub new_sync: u8,
    pub timelimit: u32,
    pub loadgame: i8,
    pub random: u8,
    pub num_players: u8,
    pub consoleplayer: i8,
    pub player_classes: [u8; 8], // NET_MAXPLAYERS is 8
}

pub const MAXNETNODES: usize = 16;
pub const NET_MAXPLAYERS: usize = 8;
pub const MAXPLAYERNAME: usize = 30;
pub const BACKUPTICS: usize = 128;
pub const NET_MAGIC_NUMBER: u32 = 1454104972;
pub const NET_OLD_MAGIC_NUMBER: u32 = 3436803284;
pub const NET_RELIABLE_PACKET: u16 = 1 << 15;

// TicDiff Flags
pub const NET_TICDIFF_FORWARD: u32 = 1 << 0;
pub const NET_TICDIFF_SIDE: u32 = 1 << 1;
pub const NET_TICDIFF_TURN: u32 = 1 << 2;
pub const NET_TICDIFF_BUTTONS: u32 = 1 << 3;
pub const NET_TICDIFF_CONSISTANCY: u32 = 1 << 4;
pub const NET_TICDIFF_CHATCHAR: u32 = 1 << 5;
pub const NET_TICDIFF_RAVEN: u32 = 1 << 6;
pub const NET_TICDIFF_STRIFE: u32 = 1 << 7;

// Enums

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetProtocol {
    #[default]
    ChocolateDoom0,
    // Add your own protocol here; be sure to add a name for it to the list in net_common.rs too.
    Unknown,
}

impl NetProtocol {
    pub const NUM_PROTOCOLS: usize = 2;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetPacketType {
    Syn,
    Ack, // deprecated
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
    NatHolePunch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetMasterPacketType {
    Add,
    AddResponse,
    Query,
    QueryResponse,
    GetMetadata,
    GetMetadataResponse,
    SignStart,
    SignStartResponse,
    SignEnd,
    SignEndResponse,
    NatHolePunch,
    NatHolePunchAll,
}

// Structs corresponding to net_defs.h

pub struct NetModule {
    // Initialize this module for use as a client
    pub init_client: fn() -> bool,

    // Initialize this module for use as a server
    pub init_server: fn() -> bool,

    // Send a packet
    pub send_packet: fn(addr: &NetAddr, packet: &NetPacket),

    // Check for new packets to receive
    // Returns true if packet received
    pub recv_packet: fn(addr: &mut Option<NetAddr>, packet: &mut Option<NetPacket>) -> bool,

    // Converts an address to a string
    pub addr_to_string: fn(addr: &NetAddr, buffer: &mut String, buffer_len: usize),

    // Free back an address when no longer in use
    pub free_address: fn(addr: &mut NetAddr),

    // Try to resolve a name to an address
    pub resolve_address: fn(addr: &str) -> Option<NetAddr>,
}

#[derive(Debug, Clone)]
pub struct NetAddr {
    pub module: *mut NetModule,
    pub refcount: i32,
    pub handle: *mut std::ffi::c_void,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetContext {
    // Define fields as necessary
}

// net_connect_data_t equivalent
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct NetConnectData {
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

// net_gamesettings_t equivalent
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct NetGameSettings {
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
    pub random: i32, // [Strife only]
    pub num_players: i32,
    pub consoleplayer: i32,
    pub player_classes: [i32; NET_MAXPLAYERS],
}

// net_ticdiff_t equivalent
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct NetTicDiff {
    pub diff: u32,
    pub cmd: TicCmd,
}

// net_full_ticcmd_t equivalent
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct NetFullTicCmd {
    pub latency: i32,
    pub seq: u32,
    pub playeringame: [bool; NET_MAXPLAYERS],
    pub cmds: [NetTicDiff; NET_MAXPLAYERS],
}

// net_querydata_t equivalent
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NetQueryData {
    pub version: String,
    pub server_state: i32,
    pub num_players: i32,
    pub max_players: i32,
    pub gamemode: i32,
    pub gamemission: i32,
    pub description: String,
    pub protocol: NetProtocol,
}

// net_waitdata_t equivalent
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
