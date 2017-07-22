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
extern crate ctrlc;
extern crate fdlimit;
extern crate ethcore_logger;
extern crate number_prefix;
extern crate app_dirs;
extern crate parity_reactor;
extern crate serde_json;
#[macro_use]
extern crate clap;
extern crate target_info;
extern crate futures_cpupool;

mod util;
mod rpc;
mod types;
mod api;
mod traits;
mod impls;
mod informant;
mod helpers;
mod dir;
mod user_defaults;
mod configuration;
mod run;

use std::{process, env};
use std::io::{self as stdio, Write};
use configuration::{Cmd, Execute, Configuration};
use ethcore_logger::setup_log;

enum PostExecutionAction {
    Print(String),
    Restart(Option<String>),
    Quit,
}

const PLEASE_RESTART_EXIT_CODE: i32 = 69;

fn execute(command: Execute, can_restart: bool) -> Result<PostExecutionAction, String> {
    let logger = setup_log(&command.logger).expect("Logger is initialized only once; qed");

    match command.cmd {
        Cmd::Run(run_cmd) => {
            let (restart, spec_name) = run::execute(run_cmd, can_restart, logger)?;
            Ok(if restart {
                PostExecutionAction::Restart(spec_name)
            } else {
                PostExecutionAction::Quit
            })
        }
        Cmd::Version => Ok(PostExecutionAction::Quit),
    }
}

fn start(can_restart: bool) -> Result<PostExecutionAction, String> {
    let conf = Configuration::parse().unwrap_or_else(|e| {
        // Otherwise, write to stderr and exit
        if e.use_stderr() {
            writeln!(&mut stdio::stderr(), "{}", e.message);
            drop(e);
            process::exit(1);
        }

        e.exit()
    });
    let cmd = conf.into_command()?;
    execute(cmd, can_restart)
}

// Returns the exit error code.
fn main_direct(can_restart: bool) -> i32 {
    match start(can_restart) {
        Ok(result) => {
            match result {
                PostExecutionAction::Print(s) => {
                    println!("{}", s);
                    0
                }
                PostExecutionAction::Restart(_) => PLEASE_RESTART_EXIT_CODE,
                PostExecutionAction::Quit => 0,
            }
        }
        Err(err) => {
            writeln!(&mut stdio::stderr(), "{}", err).expect("StdErr available; qed");
            1
        }
    }
}

fn main() {
    // Always print backtrace on panic.
    env::set_var("RUST_BACKTRACE", "1");

    let force_direct = std::env::args().any(|arg| arg == "--force-direct");
    let exe = std::env::current_exe().ok();
    let development = exe.as_ref()
        .and_then(|p| {
            p.parent().and_then(|p| p.parent()).and_then(|p| p.file_name()).map(|n| n == "target")
        })
        .unwrap_or(false);
    let same_name = exe.as_ref()
        .map(|p| {
            p.file_stem().map_or(false, |s| s == "toyrpc") &&
            p.extension().map_or(true, |x| x == "exe")
        })
        .unwrap_or(false);

    let can_restart = std::env::args().any(|arg| arg == "--can-restart");
    process::exit(main_direct(can_restart));

}
