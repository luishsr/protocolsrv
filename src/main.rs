use std::net::{UdpSocket, TcpStream, TcpListener};
use rand::Rng;
use std::thread::sleep;
use std::time;
use local_ip_address::local_ip;
use std::io::{Read, Write};
use tokio;

fn get_my_local_ip() -> String{
    local_ip().unwrap().to_string()
}

pub struct PlayerManager {
    magic_number: i32,
    local_player: String,
    remote_player: String
}

fn guess_number() -> i32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(1..=10)
}

impl PlayerManager {
    fn start_game(&mut self){
        self.magic_number = guess_number();
    }

    fn get_magic_number(&mut self) -> i32{
        self.magic_number
    }

    fn play_turn(&mut self, guess: i32) -> i32 {
        println!(" Guessed # {}", guess.to_string());
        if guess > self.magic_number {
            1
        } else if guess < self.magic_number {
            2
        } else {
            0
        }
    }
}

fn announce_presence() {
    // The IP address and port to bind to for receiving UDP messages
    let local_address = "0.0.0.0:8888";
    let remote_address = "255.255.255.255:8888"; // Broadcast address

    // Create a UDP socket for receiving messages
    let socket = UdpSocket::bind(local_address).unwrap();

    // Set the socket to allow broadcasting
    socket.set_broadcast(true).unwrap();

    // Message to send for discovery
    let discovery_message = "DISCOVERY";

    // Buffer to store received data
    let mut buffer = [0; 1024];

    // Receive responses from other devices
    loop {
        // Send the discovery message
        socket.send_to(discovery_message.as_bytes(), remote_address).unwrap();

        let (bytes_received, source_address) = socket.recv_from(&mut buffer).unwrap();

        let received_message = std::str::from_utf8(&buffer[0..bytes_received]).unwrap();
        println!("Received from {}: {}", source_address, received_message);

        // When someone replies
        if source_address.to_string() != get_my_local_ip() + ":8888" && received_message == "DISCOVERY" {
            // Invite to play
            println!("Inviting {} to play", source_address.ip().to_string());
            send_message_to_player(String::from("PLAY"), source_address.ip().to_string());

            // Stop the loop
            break
        }

        sleep(time::Duration::from_secs(2));
    }
}

async fn listen_to_players() {
    let listener = TcpListener::bind("0.0.0.0:7878").expect("Error when binding to listen on port 7878"); // Bind to an IP and port.
    println!("Server listening on port 7878...");

    loop {
        let (mut socket, _) = listener.accept().expect("Failed to accept connection");

        let origin = socket.peer_addr().unwrap().ip().to_string();
        let mut buffer = [0; 4];

        // Manage peers
        let mut player_manager = PlayerManager { magic_number: 0, local_player: get_my_local_ip(), remote_player: socket.peer_addr().unwrap().ip().to_string() };

        socket.read_exact(&mut buffer).expect("Failed to read data");

        match &buffer {
            b"PLAY" => {

                if origin.clone() != get_my_local_ip(){
                    // Start the game
                    println!("Game started!");
                    println!("Local Player: {} | Remote Player: {}", player_manager.local_player, player_manager.remote_player);
                    player_manager.start_game();

                    // Ask the peer player to play
                    send_message_to_player(String::from("TURN"), origin.clone());

                }

            },
            b"TURN" => {
                println!("Player {} is playng >>>", &origin);
                // Guess a number and play
                let win = player_manager.play_turn(guess_number());

                // Check if the player guessed the number
                if win == player_manager.get_magic_number() {
                    println!("Wow!! PERFECT MATCH!! **** Player {} WIN!", origin.clone());
                    send_message_to_player(String::from("WINN"), origin.clone());
                } else {
                    println!("Better luck next time, player {}!", origin.clone());

                    if origin.clone() == player_manager.local_player {
                        send_message_to_player(String::from("TURN"), player_manager.remote_player.clone());
                    } else {
                        //send_message_to_player(String::from("TURN"), player_manager.local_player.clone());
                        println!("Player {} is playng >>>", player_manager.local_player.clone());
                        let win_local = player_manager.play_turn(guess_number());
                        if win_local == player_manager.magic_number{
                            println!("Wow!! PERFECT MATCH!! **** Player {} WIN!", player_manager.local_player);

                            // Finish the game
                            break
                        } else {
                            send_message_to_player(String::from("TURN"), player_manager.remote_player.clone());
                        }
                    }
                }
            },
            b"WINN" => {
                println!(">>>>> The Magic Number is ** {} ** <<<<<<", player_manager.magic_number);

                // Finish the game
                break
            }
            _ => {}
        }
    }
}

fn send_message_to_player(message: String, player_address: String){
    let mut stream;

    println!("Sending message to {}", player_address);

    stream = TcpStream::connect(format!("{}:{}", player_address, String::from("7878"))).unwrap();

    // Send the message
    stream.write_all(message.as_bytes()).unwrap();

    sleep(time::Duration::from_secs(2));
}

#[tokio::main]
async fn main(){
    // Initializing
    println!("Initializing - Local IP Address: {}", get_my_local_ip());

    // Start listening for incoming connections
    tokio::spawn(async move {
        listen_to_players().await;
    });

    // Discover peers
    announce_presence();
}

