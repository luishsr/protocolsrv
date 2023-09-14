use std::collections::HashMap;
use std::net::{UdpSocket, SocketAddr};
use std::str;
use tokio;

pub struct Peer {
    address: String
}

pub struct PeerManager {
    peers: HashMap<String, Peer>
}

impl PeerManager {
    fn register_peer(&mut self, address: String) {
        self.peers.insert(address.clone(), Peer{ address: address.clone()});
    }
}

fn announce_presence(){
    // The IP address and port to bind to for receiving UDP messages
    let local_address = "0.0.0.0:8888";
    let remote_address = "255.255.255.255:8888"; // Broadcast address

    // Create a UDP socket for receiving messages
    let socket = UdpSocket::bind(local_address)?;

    // Set the socket to allow broadcasting
    socket.set_broadcast(true)?;

    // Message to send for discovery
    let discovery_message = "DISCOVERY";

    // Send the discovery message
    socket.send_to(discovery_message.as_bytes(), remote_address)?;

    println!("Sent UDP discovery message: {:?}", discovery_message);
}

async fn listen_to_peers(){
    // The IP address and port to bind to for receiving UDP messages
    let local_address = "0.0.0.0:8888";
    let remote_address = "255.255.255.255:8888"; // Broadcast address

    // Manage peers
    let mut peer_manager = PeerManager{peers: HashMap::new()};

    // Create a UDP socket for receiving messages
    let socket = UdpSocket::bind(local_address)?;

    // Buffer to store received data
    let mut buffer = [0; 1024];

    // Receive responses from other devices
    loop {
        // Read messages
        let (bytes_received, source_address) = socket.recv_from(&mut buffer)?;

        // Process messages
        let received_message = str::from_utf8(&buffer[0..bytes_received]).unwrap();

        if received_message.eq("DISCOVERY"){
            peer_manager.register_peer(source_address.to_string());
        }

        println!("Received from {}: {}", source_address, received_message);
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {

    // Announce its presence to the network
    announce_presence();

    // Listen to peers
    //listen_to_peers().await
}
