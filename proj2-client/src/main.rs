use iced::{
    alignment, 
    widget::{button, column, container, image, row, text, text_input, toggler, Stack},
    Background, Border, Color, Element, Length, Size, Task, Theme
};

use std::{
    net::{TcpStream, UdpSocket, Shutdown},
    io::{BufReader, prelude::*},
    time::{Duration, Instant}
};

fn main() -> iced::Result {
    iced::application("Speed Testing", NetworkConfigApp::update, NetworkConfigApp::view)
        .window_size(Size::new(800.0, 600.0))
        .theme(|_| Theme::Dark)
        .run_with(|| NetworkConfigApp::new())
}

async fn tcp_test(ip: String, port: String) -> String {

    println!("Testing on TCP");

    // Connect to server on TCP
    let mut stream = TcpStream::connect(format!("{}:{}", ip, port)).expect("Failed to connect to server");

    // Send initial message
    let response = "HELLO TCP\n";
    stream.write_all(response.as_bytes()).expect("Failed to send");

    // Set up buffer
    let mut buf_reader = BufReader::new(stream.try_clone().unwrap());

    // Decide number of seconds to send data
    let num_seconds = 5;

    // Loop while connection is open
    loop {

        let mut msg = String::new();
        match buf_reader.read_line(&mut msg) {

            Ok(0) => {
                break;
            }
            Ok(_) => {

                let msg = msg.trim();
                let msg_parts: Vec<&str> = msg.split_whitespace().collect();

                println!("Received from server: {}", msg);

                if msg == "READY" {
                    // Send data for _ seconds
                    let response = format!("SENDING {}\n", num_seconds);
                    stream.write_all(response.as_bytes()).expect("Failed to send");

                    // Create start and end times
                    let start = Instant::now();
                    let duration = Duration::from_secs(num_seconds);

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

                    // Get number of bytes from server
                    let num_bytes : u64 = msg_parts[1].parse().expect("Failed to get int from string");
                    println!("Server received {} bytes", num_bytes);

                    // Calculate Mbps
                    let bits= (num_bytes as f64) * 8.0;
                    let mega_bits = bits * 0.000001;
                    let mega_bps = mega_bits / (num_seconds as f64);
                    println!("Upload speed: {:.2} Mbps", mega_bps);

                    // Send ready for download message
                    let response = "READY\n";
                    stream.write_all(response.as_bytes()).expect("Failed to send");
                }

                if msg_parts[0] == "SENDING" {
                    // Read bytes
                    // Calculate and update download speed

                    let response = "RECEIVED\n";
                    stream.write_all(response.as_bytes()).unwrap();
                }

                if msg == "CLOSE" {
                    // End connection with server
                    break;
                }

            }
            Err(e) => {
                println!("Error reading from server: {}", e);
                break;
            }

        }
        

    }

    "TCP test complete".to_string()

}

fn _udp_test(_ip: String, _port: String){
    println!("Testing on UDP");
}

#[derive(Debug, Clone)]
pub enum Message {
    IpAddressChanged(String),
    PortChanged(String),
    ProtocolToggled(bool),
    Connect,
    TCPFinished(String),
}

#[derive(Debug, Clone, Copy)]
pub enum Protocol {
    UDP,
    TCP,
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::UDP => write!(f, "UDP"),
            Protocol::TCP => write!(f, "TCP"),
        }
    }
}

#[derive(Debug)]
pub struct NetworkConfigApp {
    ip_address: String,
    port: String,
    is_tcp: bool,
    background_image: iced::widget::image::Handle,
}

impl NetworkConfigApp {
    fn new() -> (Self, Task<Message>) {
        let background_image = image::Handle::from_path("TCNJ_Speed.jpg");
        
        (
            Self {
                ip_address: String::from("127.0.0.1"),
                port: String::from("8080"),
                is_tcp: true,
                background_image,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::IpAddressChanged(value) => {
                self.ip_address = value;
            }
            Message::PortChanged(value) => {
                self.port = value;
            }
            Message::ProtocolToggled(is_tcp) => {
                self.is_tcp = is_tcp;
            }
            Message::Connect => {
                let protocol = if self.is_tcp { Protocol::TCP } else { Protocol::UDP };
                let ip = self.ip_address.clone();
                let port = self.port.clone();
                println!(
                    "Connecting to {}:{} using {}",
                    ip, port, protocol
                );
                return Task::perform(
                    tcp_test(ip, port),
                    Message::TCPFinished
                );
            }
            Message::TCPFinished(result) => {
                println!("TCP testing complete: {}", result);
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let protocol = if self.is_tcp { Protocol::TCP } else { Protocol::UDP };
        
        // Background image
        let background = image(self.background_image.clone())
            .width(Length::Fill)
            .height(Length::Fill);

        // Main content container with semi-transparent background
        let content = container(
            column![
                // Title
                text("Speed Testing")
                    .size(32)
                    .color(Color::WHITE),

                // Increased spacing to push inputs lower
                text("").size(100),

                // IP Address input
                column![
                    text("IP Address:")
                        .size(16)
                        .color(Color::WHITE),
                    text_input("Enter IP address", &self.ip_address)
                        .on_input(Message::IpAddressChanged)
                        .padding(10)
                        .size(16)
                        .width(Length::Fixed(300.0))
                ]
                .spacing(5),

                // Port input
                column![
                    text("Port:")
                        .size(16)
                        .color(Color::WHITE),
                    text_input("Enter port", &self.port)
                        .on_input(Message::PortChanged)
                        .padding(10)
                        .size(16)
                        .width(Length::Fixed(300.0))
                ]
                .spacing(5),

                // Protocol toggle
                row![
                    text("Protocol:")
                        .size(16)
                        .color(Color::WHITE),
                    text("UDP")
                        .size(16)
                        .color(Color::WHITE),
                    toggler(self.is_tcp)
                        .on_toggle(Message::ProtocolToggled)
                        .size(25),
                    text("TCP")
                        .size(16)
                        .color(Color::WHITE),
                ]
                .spacing(10)
                .align_y(alignment::Vertical::Center),

                // Current selection display
                text(format!(
                    "Selected: {}:{} ({})",
                    self.ip_address, self.port, protocol
                ))
                .size(14)
                .color(Color::from_rgb(0.8, 0.8, 1.0)),

                // Connect button
                button(
                    text("Connect")
                        .size(18)
                )
                .on_press(Message::Connect)
                .padding(15)
                .width(Length::Fixed(150.0))
            ]
            .spacing(20)
            .align_x(alignment::Horizontal::Center)
        )
        .width(Length::Fixed(400.0))
        .height(Length::Shrink)
        .padding(30)
        .style(|_theme| {
            container::Style {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.7))),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 10.0.into(),
                },
                ..Default::default()
            }
        });

        // Stack background and content
        container(
            Stack::new()
                .push(background)
                .push(
                    container(content)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .center_x(Length::Fill)
                        .center_y(Length::Fill)
                )
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
