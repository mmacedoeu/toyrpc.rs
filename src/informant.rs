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

extern crate ansi_term;
use self::ansi_term::Colour::{Green, Cyan, Blue};

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, AtomicBool};
use std::time::{Instant, Duration};
use io::TimerToken;
use parking_lot::{RwLock, Mutex};
use number_prefix::{binary_prefix, Standalone, Prefixed};
use util::informant::RpcStats;

pub struct Informant {
    last_tick: RwLock<Instant>,
    with_color: bool,
    rpc_stats: Option<Arc<RpcStats>>,
    last_import: Mutex<Instant>,
    skipped: AtomicUsize,
    skipped_txs: AtomicUsize,
    in_shutdown: AtomicBool,
}

/// Format byte counts to standard denominations.
pub fn format_bytes(b: usize) -> String {
    match binary_prefix(b as f64) {
        Standalone(bytes) => format!("{} bytes", bytes),
        Prefixed(prefix, n) => format!("{:.0} {}B", n, prefix),
    }
}

/// Something that can be converted to milliseconds.
pub trait MillisecondDuration {
    /// Get the value in milliseconds.
    fn as_milliseconds(&self) -> u64;
}

impl MillisecondDuration for Duration {
    fn as_milliseconds(&self) -> u64 {
        self.as_secs() * 1000 + self.subsec_nanos() as u64 / 1_000_000
    }
}

impl Informant {
    /// Make a new instance potentially `with_color` output.
    pub fn new(rpc_stats: Option<Arc<RpcStats>>, with_color: bool) -> Self {
        Informant {
            last_tick: RwLock::new(Instant::now()),
            with_color: with_color,
            rpc_stats: rpc_stats,
            last_import: Mutex::new(Instant::now()),
            skipped: AtomicUsize::new(0),
            skipped_txs: AtomicUsize::new(0),
            in_shutdown: AtomicBool::new(false),
        }
    }

    /// Signal that we're shutting down; no more output necessary.
    pub fn shutdown(&self) {
        self.in_shutdown.store(true, ::std::sync::atomic::Ordering::SeqCst);
    }
}

const INFO_TIMER: TimerToken = 0;
