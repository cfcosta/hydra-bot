use crate::net_structs::*;
use std::net::UdpSocket;

pub fn run() {
    // Implement the client logic here
    // For example, connect to the server and send/receive data
    let server_address = "127.0.0.1:23456";
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Could not bind client socket");

    // Example: Send a connect request to the server
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

    // Serialize and send the connect_data
    let packet = serialize_connect_data(&connect_data);
    socket
        .send_to(&packet, server_address)
        .expect("Failed to send connect data");

    // Main client loop
    loop {
        // Receive data from the server
        let mut buf = [0u8; 1024];
        match socket.recv_from(&mut buf) {
            Ok((size, _src)) => {
                // Process the received data
                // TODO: Implement packet processing
            }
            Err(e) => {
                eprintln!("Failed to receive data: {}", e);
                break;
            }
        }

        // Send tic commands or other data to the server
        // TODO: Implement client logic
    }
}

// Function to serialize ConnectData into bytes
fn serialize_connect_data(data: &ConnectData) -> Vec<u8> {
    // TODO: Implement serialization according to the protocol
    Vec::new()
}
