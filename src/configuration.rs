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

use std::env;
use rpc::HttpConfiguration;
use ethcore_logger::Config as LogConfig;
use dir::{self, Directories};
use run::RunCmd;
use clap::{Arg, App, Error, ArgMatches};
use util::misc::version;

#[derive(Debug, PartialEq)]
pub enum Cmd {
    Run(RunCmd),
    Version,
}

pub struct Execute {
    pub logger: LogConfig,
    pub cmd: Cmd,
}

#[derive(Debug)]
pub struct Configuration<'a> {
    pub args: ArgMatches<'a>,
}

fn get_rpc_port() -> u16 {
    let port_str = env::var("PORT").unwrap_or(String::new());
    port_str.parse().unwrap_or(1235)
}

impl<'a> Configuration<'a> {
    pub fn parse() -> Result<Self, Error> {

        let matches = App::new("Toy json rpc").version("1.0")
            .author("Marcos Macedo <contato@mmacedo.eu.org>")
            .about("Toy json rpc starter kit")
            .arg(Arg::with_name("can-restart")
                .long("can-restart")
                .help("Executable will auto-restart if exiting with 69.")
                .group("Internal"))
            .arg(Arg::with_name("daemon")
                .long("daemon")
                .value_name("PID_FILE")
                .help("Run in daemon.")
                .group("Internal"))
            .arg(Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true)
                .group("Miscellaneous"))
            .arg(Arg::with_name("logging")
                .short("l")
                .long("logging")
                .value_name("LOGGING")
                .help("Specify the logging level. Must conform to the same
                                 format as RUST_LOG. [default: None]")
                .takes_value(true)
                .group("Miscellaneous"))
            .arg(Arg::with_name("log-file")
                .long("log-file")
                .value_name("FILENAME")
                .help("Specify a filename into which logging should be
                                 appended. [default: None]")
                .group("Miscellaneous"))
            .arg(Arg::with_name("no-color")
                .long("no-color")
                .help("Don't use terminal color codes in output. [default: false]")
                .group("Miscellaneous"))
            .arg(Arg::with_name("base-path")
                .short("d")
                .long("base-path")
                .value_name("PATH")
                .help("Specify the base data storage path.")
                .group("operation"))
            .arg(Arg::with_name("identity")
                .long("identity")
                .value_name("NAME")
                .help("Specify your node's name.")
                .default_value("")
                .group("operation"))
            .arg(Arg::with_name("no-jsonrpc")
                .long("no-jsonrpc")
                .help("Disable the JSON-RPC API server. [default: false]"))
            .arg(Arg::with_name("jsonrpc-port")
                .long("jsonrpc-port")
                .value_name("PORT")
                .help("Specify the port portion of the JSONRPC API server
                                 [default: 1235]."))
            .arg(Arg::with_name("jsonrpc-interface")
                .long("jsonrpc-interface")
                .value_name("IP")
                .help("Specify the hostname portion of the JSONRPC API
                                 server, IP should be an interface's IP address, or
                                 all (all interfaces) or local.")
                .default_value("local"))
            .arg(Arg::with_name("jsonrpc-cors")
                .long("jsonrpc-cors")
                .value_name("URL")
                .help("Specify CORS header for JSON-RPC API responses.")
                .default_value("none"))
            .arg(Arg::with_name("jsonrpc-apis")
                .long("jsonrpc-apis")
                .value_name("APIS")
                .help("Specify the APIs available through the JSONRPC
                                 interface. APIS is a comma-delimited list of API
                                 name. Possible name are web3, eth, net, personal,
                                 parity, parity_set, traces, rpc, parity_accounts.")
                .default_value("web3,eth,net,parity,traces,rpc"))
            .arg(Arg::with_name("jsonrpc-hosts")
                .long("jsonrpc-hosts")
                .value_name("HOSTS")
                .help(r#"List of allowed Host header values. This option will
                                 validate the Host header sent by the browser, it
                                 is additional security against some attack
                                 vectors. Special options: "all", "none""#)
                .default_value("none"))
            .version(version().as_str())
            .get_matches_safe()?;

        let config = Configuration { args: matches };
        Ok(config)
    }

    pub fn into_command(self) -> Result<Execute, String> {
        let dirs = self.directories();
        let logger_config = self.logger_config();
        let http_conf = self.http_config()?;

        let cmd = {
            let daemon = self.args.value_of("daemon");
            let run_cmd = RunCmd {
                dirs: dirs,
                daemon: daemon.map(str::to_string),
                logger_config: logger_config.clone(),
                http_conf: http_conf,
                name: self.args.value_of("identity").map(str::to_string).unwrap(),
            };
            Cmd::Run(run_cmd)
        };

        Ok(Execute {
            logger: logger_config,
            cmd: cmd,
        })
    }

    fn logger_config(&self) -> LogConfig {
        LogConfig {
            mode: self.args.value_of("logging").map(str::to_string),
            color: !self.args.is_present("no-color") && !cfg!(windows),
            file: self.args.value_of("log-file").map(str::to_string),
        }
    }

    fn rpc_apis(&self) -> String {
        self.args.value_of("jsonrpc-apis").map(str::to_string).unwrap()
    }

    fn cors(cors: Option<&str>) -> Option<Vec<String>> {
        cors.map(|c| c.split(',').map(Into::into).collect())
    }

    fn rpc_cors(&self) -> Option<Vec<String>> {
        let cors = self.args.value_of("jsonrpc-cors");
        Self::cors(cors)
    }

    fn hosts(hosts: &str) -> Option<Vec<String>> {
        match hosts {
            "none" => return Some(Vec::new()),
            "all" => return None,
            _ => {}
        }
        let hosts = hosts.split(',').map(Into::into).collect();
        Some(hosts)
    }

    fn rpc_hosts(&self) -> Option<Vec<String>> {
        Self::hosts(self.args.value_of("jsonrpc-hosts").unwrap())
    }

    fn http_config(&self) -> Result<HttpConfiguration, String> {
        let m = &self.args;
        let conf = HttpConfiguration {
            enabled: !self.args.is_present("no-jsonrpc"),
            interface: self.rpc_interface(),
            port: value_t!(m, "jsonrpc-port", u16).unwrap_or(get_rpc_port()),
            apis: self.rpc_apis().parse()?,
            hosts: self.rpc_hosts(),
            cors: self.rpc_cors(),
        };

        Ok(conf)
    }

    fn directories(&self) -> Directories {
        let base = dir::default_data_path();
        let base_path = self.args
            .value_of("PATH")
            .unwrap_or(&base);

        Directories { base: base_path.into() }
    }

    fn interface(interface: &str) -> String {
        match interface {
                "all" => "0.0.0.0",
                "local" => "127.0.0.1",
                x => x,
            }
            .into()
    }

    fn rpc_interface(&self) -> String {
        Self::interface(self.args.value_of("jsonrpc-interface").unwrap())
    }
}
