use hyper;
use io::PanicHandler;
use jsonrpc_core;
use jsonrpc_core::MetaIoHandler;
use jsonrpc_core::reactor::{RpcHandler, Remote};
use jsonrpc_http_server;
use jsonrpc_http_server::{ServerBuilder, RpcServerError, HttpMetaExtractor};
use types::{Origin, Metadata};
use util::informant::{Middleware, RpcStats, ClientNotifier};
use api;
use api::apis::ApiSet;
use std::fmt;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

pub use jsonrpc_http_server::Server as HttpServer;

#[derive(Debug, PartialEq)]
pub struct HttpConfiguration {
    pub enabled: bool,
    pub interface: String,
    pub port: u16,
    pub apis: ApiSet,
    pub cors: Option<Vec<String>>,
    pub hosts: Option<Vec<String>>,
}

impl Default for HttpConfiguration {
    fn default() -> Self {
        HttpConfiguration {
            enabled: true,
            interface: "127.0.0.1".into(),
            port: 8545,
            apis: ApiSet::UnsafeContext,
            cors: None,
            hosts: Some(Vec::new()),
        }
    }
}

pub struct Dependencies {
    pub panic_handler: Arc<PanicHandler>,
    pub apis: Arc<api::apis::Dependencies>,
    pub remote: Remote,
    pub stats: Arc<RpcStats>,
}

pub struct RpcExtractor;
impl HttpMetaExtractor<Metadata> for RpcExtractor {
    fn read_metadata(&self, req: &hyper::server::Request<hyper::net::HttpStream>) -> Metadata {
        let origin = req.headers()
            .get::<hyper::header::Origin>()
            .map(|origin| format!("{}://{}", origin.scheme, origin.host))
            .unwrap_or_else(|| "unknown".into());
        let mut metadata = Metadata::default();
        metadata.origin = Origin::Rpc(origin);
        metadata
    }
}

pub fn new_http(conf: HttpConfiguration,
                deps: &Dependencies)
                -> Result<Option<HttpServer>, String> {
    if !conf.enabled {
        return Ok(None);
    }

    let url = format!("{}:{}", conf.interface, conf.port);
    let addr = url.parse().map_err(|_| format!("Invalid JSONRPC listen host/port given: {}", url))?;
    Ok(Some(setup_http_rpc_server(deps, &addr, conf.cors, conf.hosts, conf.apis)?))
}

fn setup_apis(apis: ApiSet, deps: &Dependencies) -> MetaIoHandler<Metadata, Middleware> {
    api::apis::setup_rpc(deps.stats.clone(), deps.apis.clone(), apis)
}

pub fn setup_http_rpc_server(dependencies: &Dependencies,
                             url: &SocketAddr,
                             cors_domains: Option<Vec<String>>,
                             allowed_hosts: Option<Vec<String>>,
                             apis: ApiSet)
                             -> Result<HttpServer, String> {
    let apis = setup_apis(apis, dependencies);
    let handler = RpcHandler::new(Arc::new(apis), dependencies.remote.clone());
    let ph = dependencies.panic_handler.clone();
    let start_result = start_http(url, cors_domains, allowed_hosts, ph, handler, RpcExtractor);
    match start_result {
        Err(RpcServerError::IoError(err)) => {
            match err.kind() {
                io::ErrorKind::AddrInUse => {
                    Err(format!("RPC address {} is already in use, make sure that another \
                                 instance of an Ethereum client is not running or change the \
                                 address using the --jsonrpc-port and --jsonrpc-interface \
                                 options.",
                                url))
                }
                _ => Err(format!("RPC io error: {}", err)),
            }
        }
        Err(e) => Err(format!("RPC error: {:?}", e)),
        Ok(server) => Ok(server),
    }
}

/// Start http server asynchronously and returns result with `Server` handle on success or an error.
pub fn start_http<M, T, S>(addr: &SocketAddr,
                           cors_domains: Option<Vec<String>>,
                           allowed_hosts: Option<Vec<String>>,
                           panic_handler: Arc<PanicHandler>,
                           handler: RpcHandler<M, S>,
                           extractor: T)
                           -> Result<HttpServer, RpcServerError>
    where M: jsonrpc_core::Metadata,
          S: jsonrpc_core::Middleware<M>,
          T: HttpMetaExtractor<M>
{

    let cors_domains = cors_domains.map(|domains| {
        domains.into_iter()
            .map(|v| match v.as_str() {
                "*" => jsonrpc_http_server::AccessControlAllowOrigin::Any,
                "null" => jsonrpc_http_server::AccessControlAllowOrigin::Null,
                v => jsonrpc_http_server::AccessControlAllowOrigin::Value(v.into()),
            })
            .collect()
    });

    ServerBuilder::with_rpc_handler(handler)
        .meta_extractor(Arc::new(extractor))
        .cors(cors_domains.into())
        .allowed_hosts(allowed_hosts.into())
        .panic_handler(move || {
            panic_handler.notify_all("Panic in RPC thread.".to_owned());
        })
        .start_http(addr)
}
