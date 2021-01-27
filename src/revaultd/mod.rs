use std::fmt::Debug;
use std::path::Path;
use std::process::Command;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, span, Level};

mod client;
pub mod config;
pub mod model;

use client::Client;
use config::Config;
use model::{Vault, VaultTransactions};

#[derive(Debug, Clone)]
pub enum RevaultDError {
    UnexpectedError(String),
    StartError(String),
    RPCError(String),
    IOError(std::io::ErrorKind),
    NoAnswerError,
}

impl std::fmt::Display for RevaultDError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::StartError(e) => write!(f, "Revauld error while starting: {}", e),
            Self::RPCError(e) => write!(f, "Revauld error rpc call: {}", e),
            Self::UnexpectedError(e) => write!(f, "Revauld unexpected error: {}", e),
            Self::NoAnswerError => write!(f, "Revaultd returned no answer"),
            Self::IOError(kind) => write!(f, "Revaultd io error: {:?}", kind),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RevaultD {
    client: Client,
    config: Config,
}

impl RevaultD {
    pub fn new(config: &Config) -> Result<RevaultD, RevaultDError> {
        let span = span!(Level::INFO, "revaultd");
        let _enter = span.enter();

        let socket_path = config.socket_path().map_err(|e| {
            RevaultDError::UnexpectedError(format!(
                "Failed to find revaultd socket path: {}",
                e.to_string()
            ))
        })?;

        let client = Client::new(socket_path);
        let revaultd = RevaultD {
            client,
            config: config.to_owned(),
        };

        debug!("Connecting to revaultd");

        revaultd.get_info()?;

        info!("Connected to revaultd");

        Ok(revaultd)
    }

    pub fn network(&self) -> bitcoin::Network {
        match self.config.bitcoind_config.network.as_str() {
            "bitcoin" => bitcoin::Network::Bitcoin,
            "regtest" => bitcoin::Network::Regtest,
            "mainnet" => bitcoin::Network::Bitcoin,
            _ => bitcoin::Network::Bitcoin,
        }
    }

    /// Generic call function for RPC calls.
    fn call<T: Serialize + Debug, U: DeserializeOwned + Debug>(
        &self,
        method: &str,
        input: Option<T>,
    ) -> Result<U, RevaultDError> {
        let span = span!(Level::INFO, "request");
        let _guard = span.enter();
        info!(method);
        self.client
            .send_request(method, input)
            .and_then(|res| res.into_result())
            .map_err(|e| {
                error!("method {} failed: {}", method, e);
                match e {
                    client::error::Error::Io(e) => RevaultDError::IOError(e.kind()),
                    client::error::Error::NoErrorOrResult => RevaultDError::NoAnswerError,
                    _ => RevaultDError::RPCError(format!("method {} failed: {}", method, e)),
                }
            })
    }

    pub fn get_info(&self) -> Result<GetInfoResponse, RevaultDError> {
        self.call("getinfo", Option::<Request>::None)
    }

    pub fn list_vaults(&self) -> Result<ListVaultsResponse, RevaultDError> {
        self.call("listvaults", Option::<Request>::None)
    }

    pub fn list_transactions(
        &self,
        outpoints: Option<Vec<String>>,
    ) -> Result<ListTransactionsResponse, RevaultDError> {
        match outpoints {
            Some(list) => self.call(
                "listtransactions",
                Some(vec![ListTransactionsRequest(list)]),
            ),
            None => self.call("listtransactions", Option::<Request>::None),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Request {}

/// getinfo

/// getinfo response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetInfoResponse {
    pub blockheight: u64,
    pub network: String,
    pub sync: f64,
    pub version: String,
}

/// list_vaults

/// listvaults response
#[derive(Debug, Clone, Deserialize)]
pub struct ListVaultsResponse {
    pub vaults: Vec<Vault>,
}

/// list_transactions

/// listtransactions request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListTransactionsRequest(Vec<String>);

/// listtransactions response
#[derive(Debug, Clone, Deserialize)]
pub struct ListTransactionsResponse {
    pub transactions: Vec<VaultTransactions>,
}

// RevaultD can start only if a config path is given.
pub async fn start_daemon(config_path: &Path) -> Result<(), RevaultDError> {
    debug!("starting revaultd daemon");
    let child = Command::new("revaultd")
        .arg("--conf")
        .arg(config_path.to_path_buf().into_os_string().as_os_str())
        .spawn()
        .map_err(|e| {
            RevaultDError::StartError(format!("Failed to launched revaultd: {}", e.to_string()))
        })?;

    debug!("waiting for revaultd daemon status");

    // daemon binary should fork and then terminate.
    let output = child.wait_with_output().map_err(|e| {
        RevaultDError::StartError(format!("Child did not terminate: {}", e.to_string()))
    })?;

    if !output.status.success() {
        return Err(RevaultDError::StartError(format!(
            "Error revaultd terminated with status: {} and stderr: {}",
            output.status.to_string(),
            String::from_utf8_lossy(&output.stderr),
        )));
    }

    info!("revaultd daemon started");

    Ok(())
}
