// Copyright 2015 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement, version 1.0.  This, along with the
// Licenses can be found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

#![feature(exit_status)]

extern crate crust;

use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::thread;

use crust::{ConnectionManager, Endpoint};

fn main() {
    // We receive events (e.g. new connection, message received) from the ConnectionManager via an
    // asynchronous channel.
    let (channel_sender, channel_receiver) = channel();
    let connection_manager = ConnectionManager::new(channel_sender);

    // Start a thread running a loop which will receive and display responses from the peer.
    let _ = thread::Builder::new().name("SimpleSender event handler".to_string()).spawn(move || {
        loop {
            // Receive the next event
            let event = channel_receiver.recv();
            if event.is_err() {
                println!("Stopped receiving.");
                break;
            }

            // Handle the event
            match event.unwrap() {
                crust::Event::NewMessage(endpoint, bytes) => {
                    match String::from_utf8(bytes) {
                        Ok(reply) => println!("Peer on {:?} replied with \"{}\"", endpoint, reply),
                        Err(why) => {
                            println!("Error receiving message: {}", why);
                            continue
                        },
                    }
                },
                crust::Event::NewConnection(endpoint) => {
                    println!("New connection made to {:?}", endpoint);
                },
                _ => (),
            }
        }
    });

    // Try to connect to "simple_receiver" example node which should be listening on TCP port 8888
    // and for UDP broadcasts (beacon) on 9999.
    let receiver_listening_endpoint =
        Endpoint::Tcp(SocketAddr::from_str(&"127.0.0.1:8888").unwrap());
    let peer_endpoint = match connection_manager.bootstrap(Some(vec![receiver_listening_endpoint]),
                                                           Some(9999)) {
        Ok(endpoint) => endpoint,
        Err(why) => {
            println!("ConnectionManager failed to bootstrap off node listening on TCP port 8888 \
                     and UDP broadcast port 9999: {}.", why);
            println!("This example needs the \"simple_receiver\" example to be running first on \
                     this same machine.");
            std::env::set_exit_status(1);
            return;
        }
    };

    // Send all the numbers from 0 to 12 inclusive.  Expect to receive replies containing the
    // Fibonacci number for each value.
    for value in (0u8..13u8) {
        match connection_manager.send(peer_endpoint.clone(), value.to_string().into_bytes()) {
            Ok(_) => (),
            Err(why) => println!("Failed to send {} to {:?}: {}", value, peer_endpoint, why),
        }
    }

    // Allow the peer time to process the requests and reply.
    thread::sleep_ms(2000);
}
