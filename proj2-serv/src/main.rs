use std::{
    thread,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream, UdpSocket},
};

fn main() {

    let listener_8080 = TcpListener::bind("0.0.0.0:8080").unwrap();
    let listener_7070 = TcpListener::bind("0.0.0.0:7070").unwrap();

    let thread_8080 = thread::spawn(move || {
        for stream in listener_8080.incoming(){
            match stream {
                Ok(stream) => {
                    thread::spawn(move || {
                        handle_8080(stream);
                    });
                }
                Err(e) => {
                    eprintln!("Error handling client on port 8080: {}", e);
                }
            }
        }
    });

    let thread_7070 = thread::spawn(move || {
        for stream in listener_7070.incoming(){
            match stream {
                Ok(stream) => {
                    thread::spawn(move || {
                        handle_7070(stream);
                    });
                }
                Err(e) => {
                    eprintln!("Error handling client on port 7070: {}", e);
                }
            }
        }
    });

    thread_8080.join().unwrap();
    thread_7070.join().unwrap();
    
}

// Handles connections on port 8080 (TCP)
fn handle_8080(mut stream: TcpStream){
    let mut buf_reader = BufReader::new(stream.try_clone().unwrap());

    loop {
        let mut msg = String::new();
        match buf_reader.read_line(&mut msg) {
            Ok(0) => {
                println!("Connection to client ended");
                break;
            }
            Ok(_) => {

                let msg = msg.trim();
                let msg_parts: Vec<&str> = msg.split_whitespace().collect();

                println!("Received from client: {}", msg);

                if msg == "HELLO TCP" {
                    let response = "READY\n";
                    stream.write_all(response.as_bytes()).expect("Failed to send message to client");
                }

                if msg_parts[0] == "SENDING" {
                    // recieve data for 5 seconds
                    

                    // count bytes and send back value
                    let num_bytes = 300;
                    let response = format!("RECEIVED {}\n", num_bytes);
                    stream.write_all(response.as_bytes()).unwrap();
                }

                if msg == "READY" {
                    // send data for five seconds
                    let response = "SENDING\n";
                    stream.write_all(response.as_bytes()).unwrap();

                    /* send bytes in a loop for five seconds */
                }

                if msg_parts[0] == "RECEIVED" {
                    let response = "CLOSE\n";
                    stream.write_all(response.as_bytes()).unwrap();
                }

            }
            Err(e) => {
                println!("Error reading from client: {}", e);
                break;
            }

        }
        
    }

}

// Handles connections on port 7070 (UDP)
// Creates UDP socket inside for actual data transfer
fn handle_7070(stream: TcpStream){
    let _buf_reader = BufReader::new(&stream);

    let socket = UdpSocket::bind("0.0.0.0").unwrap();
    let _port_num = socket.local_addr().unwrap().port();

}