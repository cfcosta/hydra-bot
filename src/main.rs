#![allow(unused)]

mod net_client;
mod net_packet;
mod net_structs;

use net_client::NetClient;

fn main() {
    // Initialize the client
    let mut client = NetClient::new("Player1".to_string(), false);
    client.init();

    // Run the client
    loop {
        client.run();
        // Add some delay to prevent busy-waiting
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
