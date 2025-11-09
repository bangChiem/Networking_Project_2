use std::{
    thread,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream, UdpSocket, Shutdown},
    time::Instant,
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

    let socket_7070 = UdpSocket::bind("0.0.0.0:7070").expect("Could not bind to port 7070");
    socket_7070
    .set_nonblocking(true)
    .expect("Failed to set non-blocking");


    let thread_7070 = thread::spawn(move || {
        loop {
            handle_7070(&socket_7070);
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
                    let mut num_bytes: usize = 0;

                    // count bytes
                    loop {
                        let mut buf = [0u8; 1024 * 8]; // 8KB at a time
                        let n = stream.read(&mut buf).unwrap();
                        if n == 0 { break; } // connection closed

                        num_bytes += n;

                        // peek into buffer to check for message
                        if buf[..n].ends_with(b"UPLOAD_DONE\n") {
                            // subtract bytes from message
                            num_bytes -= "UPLOAD_DONE\n".len();
                            break;
                        }

                    }

                    // send back value
                    let response = format!("RECEIVED {}\n", num_bytes);
                    stream.write_all(response.as_bytes()).unwrap();
                }

                if msg == "READY" {
                    // send data for five seconds
                    let response = "SENDING\n";
                    stream.write_all(response.as_bytes()).unwrap();

                    /* send bytes in a loop for five seconds */

                    // Create start and end times
                    let start = Instant::now();

                    // Set up data to send
                    let data = vec![0; 1024]; // 1 KB of 0s

                    // loop until the duration has passed
                    while start.elapsed().as_secs_f64() < 5.0 {
                        stream.write_all(&data).unwrap();
                    }

                    let eof = "UPLOAD_DONE\n";
                    stream.write_all(eof.as_bytes()).unwrap();

                }

                if msg_parts[0] == "RECEIVED" {
                    let response = "CLOSE\n";
                    stream.write_all(response.as_bytes()).unwrap();
                    stream.shutdown(Shutdown::Both).unwrap();
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
fn handle_7070(socket: &UdpSocket) {
    let mut buf = [0u8; 1024];

    match socket.recv_from(&mut buf) {
        Ok((size, src)) => {
            let msg = String::from_utf8_lossy(&buf[..size]);
            println!("Received from {}: {}", src, msg);

            let reply = format!("Server received: {}", msg);
            if let Err(e) = socket.send_to(reply.as_bytes(), &src) {
                eprintln!("Failed to send reply: {}", e);
            } else {
                println!("Replied to {}", src);
            }
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
            // No message yet — just wait a bit and try again
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        Err(e) => eprintln!("Failed to receive data: {}", e),
    }
}