use std::collections::HashMap;
use std::net::{UdpSocket, TcpStream, TcpListener};
use tokio;
use rand::Rng;
use std::io::Write;

pub struct Player {
    address: String,
    played: bool
}

pub struct PlayerManager {
    players: HashMap<String, Player>,
    magic_number: i32
}

fn guess_number() -> i32 {
    let mut rng = rand::thread_rng();
    rng.gen()
}

impl PlayerManager {
    fn register_player(&mut self, address: String) {
        self.players.insert(address.clone(), Player{ address: address.clone(), played: false});
    }

    fn start_game(&mut self){
        self.magic_number = guess_number();
    }

    fn get_next_player(&mut self) -> String {
        let mut next_player= String::from("127.0.0.1:8080");// Initializes with itself
        for (address, player) in &self.players {
           if !player.played {
               next_player = address.clone();
           }
        }
        next_player
    }

    fn play_turn(&mut self, guess: i32, player: String) -> i32 {

        // Update turn
        self.players.get_mut(player.as_str()).expect("REASON").played = true;

        if guess > self.magic_number {
            1
        } else if guess < self.magic_number {
            2
        } else {
            0
        }
    }
}

async fn announce_presence() -> std::io::Result<()>{
    // The IP address and port to bind to for receiving UDP messages
    let local_address = "0.0.0.0:8080";
    let remote_address = "255.255.255.255:8080"; // Broadcast address

    // Create a UDP socket for receiving messages
    let socket = UdpSocket::bind(local_address)?;

    // Set the socket to allow broadcasting
    socket.set_broadcast(true)?;

    // Message to send for discovery
    let discovery_message = "PLAY";

    // Send the discovery message
    socket.send_to(discovery_message.as_bytes(), remote_address)?;

    println!("Sent UDP discovery message: {:?}", discovery_message);

    Ok(())
}

async fn listen_to_players() {
    // Manage peers
    let mut player_manager = PlayerManager{players: HashMap::new(), magic_number: 0 };

    let listener = TcpListener::bind("127.0.0.1:8080"); // Bind to a specific IP address and port

    println!("Server listening on 127.0.0.1:8080");

    // Listen for incoming connections and spawn a new thread to handle each one
    for stream in listener.expect("REASON").incoming() {
        match stream {
            Ok(stream) => {
                let buffer = [0; 1024]; // Buffer to store incoming data
                // Convert the received data to a string
                let received_message = String::from_utf8_lossy(&buffer);
                println!("Received message: {}", received_message);

                // Process received messages
                if received_message == "PLAY" {
                    // Register the peer player
                    player_manager.register_player(stream.peer_addr().expect("REASON").to_string());

                    // Register itself as the other player
                    player_manager.register_player(String::from("127.0.0.1:8080"));

                    // Start the game
                    player_manager.start_game();

                    // Ask the peer player to play
                    send_message_to_player(String::from("YOUR_TURN"), stream.peer_addr().expect("REASON").to_string()).await
                } else if received_message == "YOUR_TURN" {
                    // Guess a number and play
                    let win = player_manager.play_turn(guess_number(), stream.peer_addr().expect("REASON").to_string());

                    // Check if the player guessed the number
                    if win == player_manager.magic_number{
                        send_message_to_player(String::from("YOU_WIN"), stream.peer_addr().expect("REASON").to_string()).await
                    } else {
                        send_message_to_player(String::from("YOUR_TURN"), player_manager.get_next_player()).await;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}

async fn send_message_to_player(message: String, player_address: String){
    // Connect to the specified IP address and port
    let mut stream = TcpStream::connect(player_address).expect("REASON");

    // Send the message
    let _ = stream.write_all(message.as_bytes());

}

#[tokio::main]
async fn main(){
    // Announce its presence to the network
    announce_presence().await;

    // Listen to peers
    listen_to_players().await;
}
