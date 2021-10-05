use std::{
    collections::BTreeMap,
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Select};
use home::home_dir;
use serde::{Deserialize, Serialize};

fn main() -> Result<()> {
    let config_dir = home_dir().unwrap().join(".config/maws");
    let mut config = Config::new(config_dir.join("roles.json"), config_dir.join("menu.toml"))?;
    let account = config.select_account()?;
    let role = config.select_role(&account)?;
    config.write_defaults()?;

    let mut child = std::process::Command::new("maws")
        .args(["-b", "-r", &role.arn])
        .spawn()?;
    std::process::exit(child.wait()?.code().unwrap_or(-1));
}

struct Config {
    accounts: AccountsMap,
    defaults: Defaults,
    defaults_path: PathBuf,
}

impl Config {
    fn new(roles_path: impl AsRef<Path>, defaults_path: impl Into<PathBuf>) -> Result<Config> {
        let accounts = serde_json::from_reader(File::open(roles_path.as_ref())?)?;
        let defaults_path = defaults_path.into();
        let defaults: Defaults =
            toml::from_slice(&std::fs::read(&defaults_path).unwrap_or_default())?;
        Ok(Self {
            accounts,
            defaults,
            defaults_path,
        })
    }

    fn select_account(&mut self) -> Result<String> {
        let account_names: Vec<_> = self.accounts.keys().collect();
        let account = account_names[select(&account_names, self.defaults.account.as_ref())?];
        self.defaults.account = Some(account.clone());
        Ok(account.clone())
    }

    fn select_role(&mut self, account: &str) -> Result<Role> {
        let account_roles = self.accounts.get(account).unwrap();
        let role_names: Vec<_> = account_roles.iter().map(|r| &r.role).collect();
        let role = &account_roles[select(&role_names, self.defaults.roles.get(account))?];
        self.defaults
            .roles
            .insert(account.to_owned(), role.role.clone());
        Ok(role.clone())
    }

    fn write_defaults(&self) -> Result<()> {
        let defaults_toml = &toml::to_vec(&self.defaults)?;
        Ok(std::fs::write(&self.defaults_path, defaults_toml)?)
    }
}

#[derive(Clone, Debug, Deserialize)]
struct Role {
    arn: String,
    role: String,
}

type AccountsMap = BTreeMap<String, Vec<Role>>;

#[derive(Debug, Default, Deserialize, Serialize)]
struct Defaults {
    account: Option<String>,
    #[serde(default)]
    roles: BTreeMap<String, String>,
}

fn select<T: Eq + ToString>(items: &[T], default: Option<T>) -> std::io::Result<usize> {
    let default_index = default
        .and_then(|x| items.iter().position(|y| &x == y))
        .unwrap_or_default();
    Select::with_theme(&ColorfulTheme::default())
        .items(items)
        .default(default_index)
        .interact()
}
