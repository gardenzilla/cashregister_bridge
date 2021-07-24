extern crate base64;
extern crate websocket;

use std::process::{Command, Stdio};
use std::thread;
use std::{fs::File, io::prelude::*};
use uuid::Uuid;
use websocket::sync::Server;
use websocket::OwnedMessage;

fn main() {
    // Bind websocket server
    let server = Server::bind("127.0.0.1:2796").unwrap();
    for request in server.filter_map(Result::ok) {
        // Spawn a new thread for each connection.
        thread::spawn(|| {
            if !request
                .protocols()
                .contains(&"cashregisterbridge".to_string())
            {
                request.reject().unwrap();
                return;
            }

            let client = request.use_protocol("cashregisterbridge").accept().unwrap();

            let (mut receiver, mut sender) = client.split().unwrap();

            for message in receiver.incoming_messages() {
                let message = message.unwrap();

                match message {
                    OwnedMessage::Close(_) => {
                        let message = OwnedMessage::Close(None);
                        sender.send_message(&message).unwrap();
                        return;
                    }
                    OwnedMessage::Ping(ping) => {
                        let message = OwnedMessage::Pong(ping);
                        sender.send_message(&message).unwrap();
                    }
                    OwnedMessage::Text(b64string) => {
                        let _str: &str = &b64string[1..b64string.len() - 1];
                        let _str = _str.replace("\n", "");
                        let _str = _str.replace("\r", "");
                        let _str = _str.replace("\r\n", "");
                        let _str = _str.replace("\\n", "");
                        match base64::decode(&_str) {
                            Ok(bytes) => {
                                let file_name = format!("{}.pdf", Uuid::new_v4().to_u128_le());

                                // Create
                                let mut file = File::create(&file_name)
                                    .expect("Error while creating temp document file");

                                // Write main latex content into file
                                file.write_all(&bytes).expect("Error writing bytes to file");

                                // Flush main file
                                file.flush().expect("Error while flushing file");

                                let mut process = Command::new("lp")
                                    .arg(&file_name)
                                    .arg("-d")
                                    .arg("epson")
                                    .arg("-o")
                                    .arg("media=Custom.80x2000mm")
                                    .stdin(Stdio::piped())
                                    .stdout(Stdio::piped())
                                    .spawn()
                                    .unwrap();

                                process.wait().expect("Error waiting for child process");

                                std::fs::remove_file(&file_name)
                                    .expect("Error while removing temp doc file");
                            }
                            Err(err) => println!("Error! {}", err),
                        }
                    }
                    _ => sender.send_message(&message).unwrap(),
                }
            }
        });
    }
}
