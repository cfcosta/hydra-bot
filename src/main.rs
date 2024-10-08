#![allow(unused)]

mod bot;
mod d_loop;
mod net_client;
mod net_packet;
mod net_structs;

use std::net::SocketAddr;
use tracing::info;

use self::net_client::NetClient;
use self::net_structs::ConnectData;

fn main() {
    tracing_subscriber::fmt::init();

    info!("Initializing client");
    let mut client = NetClient::new("Player1".to_string(), false);
    client.init();

    info!("Client initialized, attempting to connect");

    let server_addr: SocketAddr = "127.0.0.1:2342".parse().expect("Invalid server address");

    let connect_data = ConnectData {
        gamemode: 0,
        gamemission: 0,
        lowres_turn: 0,
        drone: 0,
        max_players: 4,
        is_freedoom: 0,
        wad_sha1sum: [0; 20],
        deh_sha1sum: [0; 20],
        player_class: 0,
    };

    if client.connect(server_addr, connect_data) {
        info!("Connected to server, starting main loop");

        // Initialize the game loop
        d_loop::d_start_game_loop();

        loop {
            // Run the network client
            client.run();

            // Run the game loop
            d_loop::try_run_tics(&mut client);

            // Update the network state
            d_loop::net_update(&mut client);

            // Add some delay to prevent busy-waiting
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    } else {
        info!("Failed to connect to server");
    }
}
