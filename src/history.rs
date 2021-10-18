use std::{
    cell::RefCell,
    collections::{BTreeMap, HashSet},
    path::PathBuf,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
struct HistoryConfig {
    account: Option<String>, // deprecated
    last_accounts: Vec<String>,
    roles: BTreeMap<String, String>,
}

#[derive(Debug)]
pub struct History {
    config: RefCell<HistoryConfig>,
    max_last_accounts: usize,
    path: PathBuf,
}

impl History {
    pub fn new(path: impl Into<PathBuf>, max_last_accounts: usize) -> Result<Self> {
        let path = path.into();
        let mut config: HistoryConfig =
            toml::from_slice(&std::fs::read(&path).unwrap_or_default())?;
        if let Some(account) = config.account.take() {
            // migrate from old configuration
            config.last_accounts.push(account);
        }
        Ok(Self {
            config: RefCell::new(config),
            max_last_accounts,
            path,
        })
    }

    pub fn account_index(&self, account: &str) -> Option<usize> {
        self.config
            .borrow()
            .last_accounts
            .iter()
            .position(|x| x == account)
    }

    pub fn default_account(&self) -> Option<String> {
        self.config
            .borrow()
            .last_accounts
            .first()
            .map(ToOwned::to_owned)
    }

    pub fn default_role(&self, account: &str) -> Option<String> {
        self.config
            .borrow()
            .roles
            .get(account)
            .map(ToOwned::to_owned)
    }

    pub fn update(&self, account: &str, role: &str) -> Result<()> {
        let mut config = self.config.borrow_mut();
        config.last_accounts.insert(0, account.to_owned());
        let mut seen = HashSet::new();
        config.last_accounts.retain(|x| seen.insert(x.to_owned()));
        config.last_accounts.truncate(self.max_last_accounts);
        config.roles.insert(account.to_owned(), role.to_owned());
        let config_toml = &toml::to_vec(&*config)?;
        Ok(std::fs::write(&self.path, config_toml)?)
    }
}
