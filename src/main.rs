#![deny(clippy::all)]

use std::{
    collections::BTreeMap,
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use home::home_dir;
use serde::Deserialize;

mod history;
pub mod term;

use history::History;

fn main() -> Result<()> {
    let config_dir = home_dir().unwrap().join(".config/maws");
    let config = Config::new(config_dir.join("roles.json"), config_dir.join("menu.toml"))?;
    let account = config.select_account()?;
    let role = config.select_role(account)?;
    config.history.write()?;

    let mut child = std::process::Command::new("maws")
        .args(["-b", "-r", &role.arn])
        .spawn()?;
    std::process::exit(child.wait()?.code().unwrap_or(-1));
}

struct Config {
    accounts: AccountsMap,
    history: History,
}

#[derive(Clone, Debug, Deserialize)]
struct Role {
    arn: String,
    id: String,
    role: String,
}

type AccountsMap = BTreeMap<String, Vec<Role>>;

impl Config {
    fn new(roles_path: impl AsRef<Path>, history_path: impl Into<PathBuf>) -> Result<Config> {
        let roles_file = File::open(&roles_path).with_context(|| format!(
            "Cannot open configuration file at {}, please see README.md\nhttps://github.com/smarnach/maws-menu",
            roles_path.as_ref().display(),
        ))?;
        Ok(Self {
            accounts: serde_json::from_reader(roles_file)?,
            history: History::new(history_path)?,
        })
    }

    fn select_account(&self) -> Result<&String> {
        let menu_items = self
            .accounts
            .iter()
            .map(|(account, roles)| {
                (
                    format!(
                        "{:32} {}",
                        account,
                        roles.first().map(|r| r.id.as_str()).unwrap_or_default(),
                    ),
                    self.history
                        .account_index(account)
                        .map(|i| char::from_digit(i as _, 10).unwrap()),
                )
            })
            .collect();
        let selected = select(menu_items, self.history.default_account())?;
        let account = self.accounts.keys().nth(selected).unwrap();
        eprintln!("Account: {}", account);
        self.history.use_account(account, 5);
        Ok(account)
    }

    fn select_role(&self, account: &str) -> Result<&Role> {
        let account_roles = self.accounts.get(account).unwrap();
        let role_names: Vec<_> = account_roles
            .iter()
            .map(|r| (r.role.to_owned(), None))
            .collect();
        let role = &account_roles[select(role_names, self.history.default_role(account))?];
        eprintln!("Role: {}", role.role);
        self.history.use_role(account, &role.role);
        Ok(role)
    }
}

fn select(items: Vec<(String, Option<char>)>, default: Option<String>) -> std::io::Result<usize> {
    let default_index = default
        .and_then(|x| items.iter().position(|y| x == y.0))
        .unwrap_or_default();
    match term::Menu::new(items).default(default_index).interact()? {
        Some(selected) => Ok(selected),
        None => std::process::exit(-1),
    }
}
