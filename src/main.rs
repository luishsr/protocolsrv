use std::collections::HashMap;
use std::net::{UdpSocket, TcpStream, TcpListener};
use tokio;
use rand::Rng;
use std::thread::sleep;
use std::time;
use local_ip_address::local_ip;
use std::io::{Read, Write};

fn get_my_local_ip() -> String{
    local_ip().unwrap().to_string()
}

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
    fn register_player(&mut self, address: &String) {
        self.players.insert(address.clone(), Player{ address: address.clone(), played: false});
    }

    fn start_game(&mut self){
        self.magic_number = guess_number();
    }

    fn get_next_player(&mut self) -> String {
        let mut next_player= String::from("0.0.0.0:7878");// Initializes with itself
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

fn announce_presence() -> TcpStream{
    // The IP address and port to bind to for receiving UDP messages
    let local_address = "0.0.0.0:8888";
    let remote_address = "255.255.255.255:8888"; // Broadcast address

    // Create a UDP socket for receiving messages
    let socket = UdpSocket::bind(local_address).expect("REASON");

    // Set the socket to allow broadcasting
    socket.set_broadcast(true).expect("REASON");

    // Message to send for discovery
    let discovery_message = "DISCOVERY";

     // Buffer to store received data
    let mut buffer = [0; 1024];

    // TCP Stream to return
    let send_stream: TcpStream;

    // Receive responses from other devices
    loop {
        // Send the discovery message
        socket.send_to(discovery_message.as_bytes(), remote_address).expect("REASON");

        let (bytes_received, source_address) = socket.recv_from(&mut buffer).expect("REASON");

        let received_message = std::str::from_utf8(&buffer[0..bytes_received]).unwrap();
        println!("Received from {}: {}", source_address, received_message);
        //println!("My local IP is {}", get_my_local_ip());

        // When someone replies
        if source_address.to_string() != get_my_local_ip() + ":8888" &&  received_message == "DISCOVERY"{
            // Invite to play
            send_stream = send_message_to_player(String::from("PLAY"), source_address.ip().to_string(), true);

            // Stop the loop
            break
        }

        sleep(time::Duration::from_secs(2));
    }
    send_stream
}

fn listen_to_players() {
    let mut buffer = [0; 1024]; // Buffer to store incoming data

    // Manage peers
    let mut player_manager = PlayerManager { players: HashMap::new(), magic_number: 0 };

    println!("Server listening on ");

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap(); // Bind to an IP and port.
    println!("Server listening on port 8080...");

    // Discover peers
    announce_presence();

    for mut stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                loop {
                    match stream.read(&mut buffer) {
                        Ok(n) => {
                            if n == 0 {
                                break
                            }

                            // Convert the received data to a string
                            let received_message = String::from_utf8_lossy(&buffer);
                            let origin = stream.peer_addr().unwrap().to_string();
                            println!("Received message: {}", received_message);

                            // Process received messages
                            if received_message == "PLAY" {
                                // Register the peer player
                                player_manager.register_player(&origin);

                                // Register itself as the other player
                                player_manager.register_player(&get_my_local_ip());

                                // Start the game
                                player_manager.start_game();

                                // Ask the peer player to play
                                send_message_to_player(String::from("YOUR_TURN"), origin.clone(), false);
                            } else if received_message == "YOUR_TURN" {
                                // Guess a number and play
                                let win = player_manager.play_turn(guess_number(), origin.clone());

                                // Check if the player guessed the number
                                if win == player_manager.magic_number {
                                    send_message_to_player(String::from("YOU_WIN"), origin.clone(), false);
                                } else {
                                    send_message_to_player(String::from("YOUR_TURN"), player_manager.get_next_player(), false);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error accepting connection: {}", e);
                        }
                    }
                }
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }
}

fn send_message_to_player(message: String, player_address: String, change_port: bool) -> TcpStream{
    let mut stream;

    if change_port{
        // Connect to the specified IP address and port
        stream = TcpStream::connect(format!("{}:{}", player_address, String::from("7878"))).expect("REASON");
    } else {
        stream = TcpStream::connect(player_address).expect("REASON");
    }

    // Send the message
    stream.write_all(message.as_bytes()).unwrap();
    stream
}

#[tokio::main]
async fn main(){
    //Listen to peers to play with
    listen_to_players();
}

