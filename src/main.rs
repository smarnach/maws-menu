use std::{
    cell::RefCell,
    collections::BTreeMap,
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use home::home_dir;
use serde::{Deserialize, Serialize};

pub mod term;

fn main() -> Result<()> {
    let config_dir = home_dir().unwrap().join(".config/maws");
    let config = Config::new(config_dir.join("roles.json"), config_dir.join("menu.toml"))?;
    let account = config.select_account()?;
    let role = config.select_role(account)?;
    config.write_history()?;

    let mut child = std::process::Command::new("maws")
        .args(["-b", "-r", &role.arn])
        .spawn()?;
    std::process::exit(child.wait()?.code().unwrap_or(-1));
}

struct Config {
    accounts: AccountsMap,
    history: RefCell<History>,
    history_path: PathBuf,
}

impl Config {
    fn new(roles_path: impl AsRef<Path>, history_path: impl Into<PathBuf>) -> Result<Config> {
        let roles_file = File::open(&roles_path).with_context(|| format!(
            "Cannot open configuration file at {}, please see README.md\nhttps://github.com/smarnach/maws-menu",
            roles_path.as_ref().display(),
        ))?;
        let history_path = history_path.into();
        let history: History = toml::from_slice(&std::fs::read(&history_path).unwrap_or_default())?;
        Ok(Self {
            accounts: serde_json::from_reader(roles_file)?,
            history: RefCell::new(history),
            history_path,
        })
    }

    fn select_account(&self) -> Result<&String> {
        let account_names = self.accounts.keys().map(|x| (x.to_owned(), None)).collect();
        let selected = select(account_names, self.history.borrow().account.as_ref())?;
        let account = self.accounts.keys().nth(selected).unwrap();
        self.history.borrow_mut().account = Some(account.clone());
        Ok(account)
    }

    fn select_role(&self, account: &str) -> Result<&Role> {
        let account_roles = self.accounts.get(account).unwrap();
        let role_names: Vec<_> = account_roles
            .iter()
            .map(|r| (r.role.to_owned(), None))
            .collect();
        let role = &account_roles[select(role_names, self.history.borrow().roles.get(account))?];
        self.history
            .borrow_mut()
            .roles
            .insert(account.to_owned(), role.role.clone());
        Ok(role)
    }

    fn write_history(&self) -> Result<()> {
        let history_toml = &toml::to_vec(&self.history)?;
        Ok(std::fs::write(&self.history_path, history_toml)?)
    }
}

#[derive(Clone, Debug, Deserialize)]
struct Role {
    arn: String,
    role: String,
}

type AccountsMap = BTreeMap<String, Vec<Role>>;

#[derive(Debug, Default, Deserialize, Serialize)]
struct History {
    account: Option<String>,
    #[serde(default)]
    roles: BTreeMap<String, String>,
}

fn select(items: Vec<(String, Option<char>)>, default: Option<&String>) -> std::io::Result<usize> {
    let default_index = default
        .and_then(|x| items.iter().position(|y| x == &y.0))
        .unwrap_or_default();
    term::Menu::new(items).default(default_index).interact()
}
