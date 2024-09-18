#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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

// Implement serialization and deserialization methods for these structs as needed.
