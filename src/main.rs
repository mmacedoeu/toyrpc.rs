#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log as rlog;
extern crate env_logger;
extern crate arrayvec;
extern crate parking_lot;
extern crate ansi_term;
extern crate futures;
extern crate order_stat;
#[macro_use]
extern crate hyper;
extern crate rustc_serialize;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate rand;
extern crate bigint;
extern crate libc;
#[macro_use]
extern crate heapsize;
extern crate jsonrpc_core;
extern crate jsonrpc_http_server;
#[macro_use]
extern crate jsonrpc_macros;
extern crate ethcore_io as io;

mod util;
mod rpc;
mod types;
mod api;
mod traits;
mod impls;

use std::env;

fn get_server_port() -> u16 {
    let port_str = env::var("PORT").unwrap_or(String::new());
    port_str.parse().unwrap_or(1235)
}

fn main() {
    println!("Hello, world!");
}
