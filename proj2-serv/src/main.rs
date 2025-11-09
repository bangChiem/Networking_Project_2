use std::{
    thread,
    io::{self, BufReader, prelude::*},
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

fn read_line(stream: &mut TcpStream, buf: &mut String) -> io::Result<usize> {
    let mut total_bytes = 0u64;
    let mut buffer = [0; 1];

    loop {
        let bytes_read = stream.read(&mut buffer)?;
        if bytes_read == 0 {
            // EOF
            break;
        }

        let byte = buffer[0];
        total_bytes += 1;

        buf.push(byte as char);

        if byte == b'\n' {
            break;
        }
    }

    Ok(total_bytes as usize)
}

// Handles connections on port 8080 (TCP)
fn handle_8080(mut stream: TcpStream){
    let mut buf_reader = BufReader::new(stream.try_clone().unwrap());

    loop {
        let mut msg = String::new();
        match read_line(&mut stream, &mut msg) {
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
                    stream.flush().unwrap();

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
                    stream.flush().unwrap();

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
fn handle_7070(stream: TcpStream){
    let _buf_reader = BufReader::new(&stream);

    let socket = UdpSocket::bind("0.0.0.0").unwrap();
    let _port_num = socket.local_addr().unwrap().port();

}