// Copyright 2015 MaidSafe.net limited
//
// This MaidSafe Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the MaidSafe Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement, version 1.0, found in the root
// directory of this project at LICENSE, COPYING and CONTRIBUTOR respectively and also
// available at: http://www.maidsafe.net/licenses
//
// Unless required by applicable law or agreed to in writing, the MaidSafe Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS
// OF ANY KIND, either express or implied.
//
// See the Licences for the specific language governing permissions and limitations relating to
// use of the MaidSafe Software.

// String.as_str() is unstable; waiting RFC revision
// http://doc.rust-lang.org/nightly/std/string/struct.String.html#method.as_str
#![feature(convert)]

extern crate crust;
extern crate rustc_serialize;
extern crate docopt;

use crust::{Endpoint, Port};
use crust::ConnectionManager;
use docopt::Docopt;
use std::sync::mpsc::channel;
use std::env;
use std::io;
use std::net::SocketAddr;
use std::str::FromStr;

// TODO: switching order of CL params, eg --speed x --bootstrap node
//       gives an error parsing node as usize... so order is strict for now
static USAGE: &'static str = "
Usage: crust_node
       crust_node -h
       crust_node -b <peer>
       crust_node -s <speed>
       crust_node -b <peer> -s <speed>

Options:
    -h, --help        Display this help message.
    -b, --bootstrap   Start a crust node and bootstrap off the peer.
                      If no bootstrap node is provided beacon will be used.
    -s, --speed       Optional send data at maximum speed (bytes/second)
";

#[derive(RustcDecodable, Debug)]
struct Args {
  arg_peer : Option<String>,
  arg_speed : Option<usize>,
  flag_help : bool,
  flag_bootstrap : bool,
  flag_speed : bool
}

// simple "NodeInfo", without PKI
#[derive(Clone)]
struct CrustNode {
  pub endpoint : Endpoint,
  pub connected : bool
}

impl CrustNode {
  pub fn new(endpoint : Endpoint) -> CrustNode {
    CrustNode{
      endpoint : endpoint,
      connected : false
    }
  }
  pub fn set_connected(&mut self) {self.connected = true;}
  pub fn set_disconnected(&mut self) {self.connected = false;}
}

struct FlatWorld {
  our_ep : Endpoint,
  crust_nodes : Vec<CrustNode>
}

// simple "routing table" without any structure
impl FlatWorld {
  pub fn new(our_endpoint : Endpoint) -> FlatWorld {
    FlatWorld {
      our_ep : our_endpoint,
      crust_nodes : Vec::with_capacity(40)
    }
  }

  // Will add node if not duplicated.  Returns true when added.
  pub fn add(&mut self, new_node : CrustNode) -> bool {
    if (self.crust_nodes.iter()
                        .filter(|node| node.endpoint == new_node.endpoint)
                        .count() == 0 &&
        self.our_ep != new_node.endpoint) {
      self.crust_nodes.push(new_node);
      return true;
    }
    return false;
  }

  pub fn get_connected_nodes(&self) -> Vec<CrustNode> {
    self.crust_nodes.iter()
                    .filter_map(|node| if node.connected == true {
                      Some(node.clone())
                      } else {None})
                    .collect::<Vec<_>>()
  }

  pub fn get_all_nodes(&self) -> Vec<CrustNode> {
    self.crust_nodes.clone()
  }
}

fn main() {

  let args : Args = Docopt::new(USAGE)
                     .and_then(|d| d.decode())
                     .unwrap_or_else(|e| e.exit());
  // TODO: remove; here for debug
  if !args.flag_help { println!("{:?}", args); };
  if args.flag_help {
    println!("{:?}", args);     // print help message
    return;
  };

  let (cm_tx, cm_rx) = channel();
  let cm = ConnectionManager::new(cm_tx);
  let cm_eps = match cm.start_listening(vec![Port::Tcp(0)]) {
    Ok(eps) => eps,
    Err(e) => panic!("Connection manager failed to start on arbitrary TCP port: {}", e)
  };

  let mut default_bootstrap = !args.flag_bootstrap;
  if !default_bootstrap {
    match args.arg_peer {
      Some(peer) => {
        // String.as_str() is unstable; waiting RFC revision
        // http://doc.rust-lang.org/nightly/std/string/struct.String.html#method.as_str
        let bootstrap_address = match SocketAddr::from_str(peer.as_str()) {
          Ok(addr) => addr,
          Err(e) => panic!("Failed to parse bootstrap peer as valid IPv4 or IPv6 address: {}", peer)
        };
        cm.bootstrap(Some(vec![Endpoint::Tcp(bootstrap_address)]));
      },
      None => { println!("No peer address provided, result to default");
                default_bootstrap = true; }
    }
  }

  //
  if default_bootstrap {

  }

  // first rely on beacons to bootstrap
  // cm.bootstrap(None);
}
