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

use std::fs;
use std::path::{PathBuf, Path};
use helpers::{replace_home, replace_home_for_db};
use app_dirs::{AppInfo, get_app_root, AppDataType};

#[cfg(target_os = "macos")]
const AUTHOR: &'static str = "Toyrpc";
#[cfg(target_os = "macos")]
const PRODUCT: &'static str = "org.eu.mmacedo.toyrpc";
#[cfg(target_os = "macos")]
const PRODUCT_HYPERVISOR: &'static str = "org.eu.mmacedo.toyrpc-updates";
#[cfg(target_os = "windows")]
const AUTHOR: &'static str = "Toyrpc";
#[cfg(target_os = "windows")]
const PRODUCT: &'static str = "Toyrpc";
#[cfg(target_os = "windows")]
const PRODUCT_HYPERVISOR: &'static str = "Toyrpc-updates";
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
const AUTHOR: &'static str = "toyrpc";
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
const PRODUCT: &'static str = "org.eu.mmacedo.toyrpc";
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
const PRODUCT_HYPERVISOR: &'static str = "org.eu.mmacedo.toyrpc-updates";

#[cfg(target_os = "windows")]
pub const CHAINS_PATH: &'static str = "$LOCAL/chains";
#[cfg(not(target_os = "windows"))]
pub const CHAINS_PATH: &'static str = "$BASE/chains";

#[derive(Debug, PartialEq)]
pub struct Directories {
    pub base: String,
}

impl Default for Directories {
    fn default() -> Self {
        let data_dir = default_data_path();
        let local_dir = default_local_path();
        Directories { base: replace_home(&data_dir, "$BASE") }
    }
}

impl Directories {
    pub fn create_dirs(&self,
                       dapps_enabled: bool,
                       signer_enabled: bool,
                       secretstore_enabled: bool)
                       -> Result<(), String> {
        fs::create_dir_all(&self.base).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Get the ipc sockets path
    pub fn ipc_path(&self) -> PathBuf {
        let mut dir = Path::new(&self.base).to_path_buf();
        dir.push("ipc");
        dir
    }

    // TODO: remove in 1.7
    pub fn legacy_keys_path(&self, testnet: bool) -> PathBuf {
        let mut dir = Path::new(&self.base).to_path_buf();
        if testnet {
            dir.push("testnet_keys");
        } else {
            dir.push("keys");
        }
        dir
    }
}

#[derive(Debug, PartialEq)]
pub struct DatabaseDirectories {
    pub path: String,
    pub legacy_path: String,
    pub fork_name: Option<String>,
    pub spec_name: String,
}

impl DatabaseDirectories {
    pub fn spec_root_path(&self) -> PathBuf {
        let mut dir = Path::new(&self.path).to_path_buf();
        dir.push(&self.spec_name);
        dir
    }

    pub fn db_root_path(&self) -> PathBuf {
        let mut dir = self.spec_root_path();
        dir.push("db");
        dir
    }

    pub fn user_defaults_path(&self) -> PathBuf {
        let mut dir = self.spec_root_path();
        dir.push("user_defaults");
        dir
    }

    /// Get the path for the snapshot directory given the genesis hash and fork name.
    pub fn snapshot_path(&self) -> PathBuf {
        let mut dir = self.db_root_path();
        dir.push("snapshot");
        dir
    }

    /// Get the path for the network directory.
    pub fn network_path(&self) -> PathBuf {
        let mut dir = self.spec_root_path();
        dir.push("network");
        dir
    }
}

pub fn default_data_path() -> String {
    let app_info = AppInfo {
        name: PRODUCT,
        author: AUTHOR,
    };
    get_app_root(AppDataType::UserData, &app_info)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "$HOME/.toyrpc".to_owned())
}

pub fn default_local_path() -> String {
    let app_info = AppInfo {
        name: PRODUCT,
        author: AUTHOR,
    };
    get_app_root(AppDataType::UserCache, &app_info)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "$HOME/.parity".to_owned())
}

pub fn default_hypervisor_path() -> String {
    let app_info = AppInfo {
        name: PRODUCT_HYPERVISOR,
        author: AUTHOR,
    };
    get_app_root(AppDataType::UserData, &app_info)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "$HOME/.parity-hypervisor".to_owned())
}

#[cfg(test)]
mod tests {
    use super::Directories;
    use helpers::{replace_home, replace_home_for_db};

    #[test]
    fn test_default_directories() {
        let data_dir = super::default_data_path();
        let local_dir = super::default_local_path();
        let expected = Directories {
            base: replace_home(&data_dir, "$BASE"),
            db: replace_home_for_db(&data_dir,
                                    &local_dir,
                                    if cfg!(target_os = "windows") {
                                        "$LOCAL/chains"
                                    } else {
                                        "$BASE/chains"
                                    }),
            keys: replace_home(&data_dir, "$BASE/keys"),
            signer: replace_home(&data_dir, "$BASE/signer"),
            dapps: replace_home(&data_dir, "$BASE/dapps"),
            secretstore: replace_home(&data_dir, "$BASE/secretstore"),
        };
        assert_eq!(expected, Directories::default());
    }
}
