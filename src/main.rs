extern crate base64;
extern crate websocket;

use serde::Deserialize;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::{thread, vec};
use websocket::sync::Server;
use websocket::OwnedMessage;

// Example command
// echo "fiscat/AEE|SLD|||5|Gardenzilla|Köszönjük, hogy nálunk vásárolt|"*"|www.gardenzilla.hu|Csók a családnak|||""|1|termék|8|5|||P|500"
//   > /dev/ttyUSB0
#[derive(Deserialize)]
struct CashierCommand {
    #[serde(default = "default_footnote")]
    footnote: Vec<String>,
    total_price: i32,
    payment_kind: PaymentKind,
}

#[derive(Deserialize)]
enum PaymentKind {
    Cash,
    Card,
}

impl PaymentKind {
    fn to_code_str(&self) -> &'static str {
        match self {
            PaymentKind::Cash => "P",
            PaymentKind::Card => "N",
        }
    }
}

fn default_footnote() -> Vec<String> {
    let v = vec![
        "Köszönjük, hogy nálunk vásárolt!".to_string(),
        "*".to_string(),
        "www.gardenzilla.hu".to_string(),
        "Eszelős favágó".to_string(),
    ];
    v
}

impl CashierCommand {
    // IMPORTANT!
    // Currently only works with FisCat cash register
    // with software version 0005, via serial communication
    fn to_child_process(self) {
        let footnote_vec = self.footnote;
        // Create footnote command parts
        let mut footnote = format!("{}", footnote_vec.len());
        footnote_vec
            .iter()
            .for_each(|n| footnote.push_str(&format!("|{}", n)));

        let command_string = format!(
            "fiscat/AEE|SLD|||{}|||\"\"|1|Tételek|8|{}|||{}",
            footnote,
            self.total_price,
            self.payment_kind.to_code_str(),
        );

        
        let mut device_file = OpenOptions::new().read(true).write(true).open("/dev/ttyUSB0").expect("Could not open cash register device");

        device_file.write_all(command_string.as_bytes()).expect("Error while writing to USB device");
    }
}

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
                    OwnedMessage::Text(jsonstring) => {
                        match serde_json::from_str::<CashierCommand>(&jsonstring) {
                            Ok(command) => {
                                command
                                    .to_child_process();
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

#[cfg(test)]
mod tests {
    use crate::CashierCommand;

    #[test]
    fn test_deserialize() {
        assert!(serde_json::from_str::<CashierCommand>(
            r#"{"total_price": 12, "payment_kind":"Cash"}"#
        )
        .is_ok());
        assert!(serde_json::from_str::<CashierCommand>(
            r#"{"total_price": 12, "payment_kind":"Card"}"#
        )
        .is_ok());
        assert!(serde_json::from_str::<CashierCommand>(
            r#"{"total_price": 12, "payment_kind":"Cashh"}"#
        )
        .is_err());
        let command =
            serde_json::from_str::<CashierCommand>(r#"{"total_price": 12, "payment_kind":"Cash"}"#)
                .unwrap();
        assert_eq!(command.footnote.len() > 0, true);
    }
}
