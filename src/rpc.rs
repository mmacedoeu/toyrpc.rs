use jsonrpc_core;
use jsonrpc_core::MetaIoHandler;
use jsonrpc_http_server::{ServerBuilder, Error as HttpServerError, MetaExtractor,
                          AccessControlAllowOrigin, Host, DomainsValidation};
use types::{Origin, Metadata};
use util::informant::{Middleware, RpcStats, CpuPool};
use api;
use api::apis::ApiSet;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use parity_reactor::TokioRemote;

pub use jsonrpc_http_server::Server as HttpServer;
pub use jsonrpc_http_server::hyper;

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
    pub apis: Arc<api::apis::Dependencies>,
    pub remote: TokioRemote,
    pub stats: Arc<RpcStats>,
	pub pool: Option<CpuPool>,    
}

pub struct RpcExtractor;
impl MetaExtractor<Metadata> for RpcExtractor {
    fn read_metadata(&self, req: &hyper::server::Request) -> Metadata {
        let origin = req.headers()
            .get::<hyper::header::Origin>()
            .map(|origin| format!("{:?}://{:?}", origin.scheme(), origin.host()))
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
    api::apis::setup_apis(deps.stats.clone(), deps.apis.clone(), apis, deps.pool.clone())
}

pub fn setup_http_rpc_server(dependencies: &Dependencies,
                             url: &SocketAddr,
                             cors_domains: Option<Vec<String>>,
                             allowed_hosts: Option<Vec<String>>,
                             apis: ApiSet)
                             -> Result<HttpServer, String> {
    let handler = setup_apis(apis, dependencies);
    let remote = dependencies.remote.clone();
    let cors_domains: Option<Vec<_>> = cors_domains.map(|domains| domains.into_iter().map(AccessControlAllowOrigin::from).collect());
    let allowed_hosts: Option<Vec<_>> =
        allowed_hosts.map(|hosts| hosts.into_iter().map(Host::from).collect());
    let start_result = start_http(url,
                                  cors_domains.into(),
                                  allowed_hosts.into(),
                                  handler,
                                  remote,
                                  RpcExtractor);
    match start_result {
        Err(HttpServerError::Io(err)) => {
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

pub fn start_http<M, S, H, T>(addr: &SocketAddr,
                              cors_domains: DomainsValidation<AccessControlAllowOrigin>,
                              allowed_hosts: DomainsValidation<Host>,
                              handler: H,
                              remote: TokioRemote,
                              extractor: T)
                              -> Result<HttpServer, HttpServerError>
    where M: jsonrpc_core::Metadata,
          S: jsonrpc_core::Middleware<M>,
          H: Into<jsonrpc_core::MetaIoHandler<M, S>>,
          T: MetaExtractor<M>
{
    ServerBuilder::new(handler)
        .event_loop_remote(remote)
        .meta_extractor(extractor)
        .cors(cors_domains.into())
        .allowed_hosts(allowed_hosts.into())
        .start_http(addr)
}
