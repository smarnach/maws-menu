#![deny(clippy::all)]

use std::{
    collections::BTreeMap,
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version, Arg};
use home::home_dir;
use serde::Deserialize;

mod history;
pub mod term;

use history::History;

fn main() -> Result<()> {
    let matches = app_from_crate!()
        .arg(
            Arg::with_name("config_dir")
                .long("config-dir")
                .takes_value(true)
                .value_name("PATH")
                .help("Configuration file directory"),
        )
        .arg(
            Arg::with_name("history_accounts")
                .long("history-accounts")
                .takes_value(true)
                .value_name("NUMBER")
                .default_value("5")
                .validator(validate_usize)
                .help("Number of previously used accounts to save in the history"),
        )
        .arg(
            Arg::with_name("reuse_last_role")
                .long("reuse-last-role")
                .help("Automatically reuse the last role for the chosen account"),
        )
        .get_matches();
    let config_dir = matches
        .value_of_os("config_dir")
        .map(Into::into)
        .unwrap_or_else(|| home_dir().unwrap().join(".config/maws"));
    let history_accounts = matches
        .value_of("history_accounts")
        .unwrap()
        .parse()
        .unwrap();
    let reuse_last_role = matches.is_present("reuse_last_role");

    let select = AccountSelect::new(
        config_dir.join("roles.json"),
        config_dir.join("menu.toml"),
        history_accounts,
    )?;
    let account = select.select_account()?;
    eprintln!("Account: {}", account);
    let role = select.select_role(account, reuse_last_role)?;
    eprintln!("Role: {}", role.role);

    let mut child = std::process::Command::new("maws")
        .args(["-r", &role.arn])
        .spawn()?;
    std::process::exit(child.wait()?.code().unwrap_or(-1));
}

fn validate_usize(s: String) -> std::result::Result<(), String> {
    match s.parse::<usize>() {
        Ok(_) => Ok(()),
        Err(_) => Err("must be a non-negative integral number".into()),
    }
}

struct AccountSelect {
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

impl AccountSelect {
    fn new(
        roles_path: impl AsRef<Path>,
        history_path: impl Into<PathBuf>,
        max_last_accounts: usize,
    ) -> Result<AccountSelect> {
        let roles_file = File::open(&roles_path).with_context(|| format!(
            "Cannot open configuration file at {}, please see README.md\nhttps://github.com/smarnach/maws-menu",
            roles_path.as_ref().display(),
        ))?;
        Ok(Self {
            accounts: serde_json::from_reader(roles_file)?,
            history: History::new(history_path, max_last_accounts)?,
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
        let default_index = self
            .history
            .default_account()
            .and_then(|x| self.accounts.keys().position(|y| &x == y))
            .unwrap_or_default();
        let selected = select(menu_items, default_index)?;
        let account = self.accounts.keys().nth(selected).unwrap();
        Ok(account)
    }

    fn select_role(&self, account: &str, reuse_last_role: bool) -> Result<&Role> {
        let account_roles = self.accounts.get(account).unwrap();
        let default_role = self.history.default_role(account);
        let default_index = default_role
            .as_ref()
            .and_then(|x| account_roles.iter().position(|y| x == &y.role))
            .unwrap_or_default();
        if reuse_last_role {
            if let Some(role) = default_role {
                self.history.update(account, &role)?;
                return Ok(&account_roles[default_index]);
            }
        }
        let menu_items: Vec<_> = account_roles
            .iter()
            .map(|r| (r.role.to_owned(), None))
            .collect();
        let role = &account_roles[select(menu_items, default_index)?];
        self.history.update(account, &role.role)?;
        Ok(role)
    }
}

fn select(items: Vec<(String, Option<char>)>, default_index: usize) -> std::io::Result<usize> {
    match term::Menu::new(items).default(default_index).interact()? {
        Some(selected) => Ok(selected),
        None => std::process::exit(-1),
    }
}
