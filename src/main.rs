use std::net::{UdpSocket, SocketAddr};
use std::str;

fn main() -> std::io::Result<()> {
    // The IP address and port to bind to for receiving UDP messages
    let local_address = "0.0.0.0:8888";
    let remote_address = "255.255.255.255:8888"; // Broadcast address

    // Create a UDP socket for receiving messages
    let socket = UdpSocket::bind(local_address)?;

    // Set the socket to allow broadcasting
    socket.set_broadcast(true)?;

    // Message to send for discovery
    let discovery_message = "Hello, devices! Are you there?";

    // Send the discovery message
    socket.send_to(discovery_message.as_bytes(), remote_address)?;

    println!("Sent UDP discovery message: {:?}", discovery_message);

    // Buffer to store received data
    let mut buffer = [0; 1024];

    // Receive responses from other devices
    loop {
        let (bytes_received, source_address) = socket.recv_from(&mut buffer)?;

        let received_message = str::from_utf8(&buffer[0..bytes_received]).unwrap();
        println!("Received from {}: {}", source_address, received_message);
    }
}
