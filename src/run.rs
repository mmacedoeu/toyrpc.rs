// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use std::sync::Arc;
use std::net::TcpListener;
use ctrlc;
use fdlimit::raise_fd_limit;
use ethcore_logger::RotatingLogger;
use parking_lot::{Mutex, Condvar};
use ansi_term::Colour;
use io::{MayPanic, PanicHandler};
use ethcore_logger::Config as LogConfig;
use util::informant::{self, CpuPool, Builder};
use informant::{Informant};
use rpc::HttpConfiguration;
use dir::Directories;
use user_defaults::UserDefaults;
use api::apis;
use rpc;
use parity_reactor::EventLoop;
use util::misc::version;

#[derive(Debug, PartialEq)]
pub struct RunCmd {
    pub dirs: Directories,
    /// Some if execution should be daemonized. Contains pid_file path.
    pub daemon: Option<String>,
    pub logger_config: LogConfig,
    pub http_conf: HttpConfiguration,
    pub name: String,
}

pub fn execute(cmd: RunCmd,
               can_restart: bool,
               logger: Arc<RotatingLogger>)
               -> Result<(bool, Option<String>), String> {

    // set up panic handler
    let panic_handler = PanicHandler::new_in_arc();

    // increase max number of open files
    raise_fd_limit();

    // run in daemon mode
    if let Some(pid_file) = cmd.daemon {
        daemonize(pid_file.into())?;
    }

    info!("Starting {}", Colour::White.bold().paint(version()));

    // spin up event loop
    let event_loop = EventLoop::spawn();

    // set up dependencies for rpc servers
    let rpc_stats = Arc::new(informant::RpcStats::default());
    let deps_for_rpc_apis = Arc::new(apis::Dependencies { logger: logger.clone() });

    let dependencies = rpc::Dependencies {
        apis: deps_for_rpc_apis.clone(),
        remote: event_loop.raw_remote(),
        stats: rpc_stats.clone(),
        pool: Some(Builder::new().create()),
    };

    // start rpc servers
    let http_server = rpc::new_http(cmd.http_conf, &dependencies)?;

    // the informant
    let informant = Arc::new(Informant::new(Some(rpc_stats.clone()), cmd.logger_config.color));

    // Handle exit
    let restart = wait_for_exit(panic_handler, can_restart);

    // drop this stuff as soon as exit detected.
    drop((http_server, event_loop));

    info!("Finishing work, please wait...");

    // to make sure timer does not spawn requests while shutdown is in progress
    informant.shutdown();
    // just Arc is dropping here, to allow other reference release in its default time
    drop(informant);

    Ok(restart)
}

#[cfg(not(windows))]
fn daemonize(pid_file: String) -> Result<(), String> {
    extern crate daemonize;

    daemonize::Daemonize::new()
        .pid_file(pid_file)
        .chown_pid_file(true)
        .start()
        .map(|_| ())
        .map_err(|e| format!("Couldn't daemonize; {}", e))
}

#[cfg(windows)]
fn daemonize(_pid_file: String) -> Result<(), String> {
    Err("daemon is no supported on windows".into())
}

fn wait_for_exit(panic_handler: Arc<PanicHandler>, can_restart: bool) -> (bool, Option<String>) {
    let exit = Arc::new((Mutex::new((false, None)), Condvar::new()));

    // Handle possible exits
    let e = exit.clone();
    ctrlc::set_handler(move || {
        e.1.notify_all();
    });

    // Handle panics
    let e = exit.clone();
    panic_handler.on_panic(move |_reason| {
        e.1.notify_all();
    });

    // Wait for signal
    let mut l = exit.0.lock();
    let _ = exit.1.wait(&mut l);
    l.clone()
}
