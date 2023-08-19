use std::fs;

use dialoguer::{console::Term, theme::ColorfulTheme, Select};
mod setup;
mod audit;
mod login;
mod models;
mod tcplistener;
mod helpers;
mod notifications;
use notifications::{setup_notifications, sent_test_email};
use login::login;
use audit::{get_audit_items, add_audit_items, remove_item, get_users_groups};


use anyhow::Result;
use models::config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    let file_string = fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&file_string)?;

    let username = config.vb365.username.clone();
    let address = config.vb365.address.clone();

    let selections = &[
        "Get Audit Items",
        "Add Audit items",
        "Remove Audit items",
        "Get Users/Groups",
        "Setup Notifications",
        "Send Test Email",
    ];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select action")
        .default(0)
        .items(&selections[..])
        .interact_on_opt(&Term::stderr())
        .unwrap()
        .unwrap();

    match selection {
        0 => get_audit_items(&username, &address).await?,
        1 => add_audit_items(&username, &address).await?,
        2 => remove_item(&username, &address).await?,
        3 => get_users_groups(&username, &address).await?,
        4 => setup_notifications(&username, &address, config).await?,
        5 => sent_test_email(&username, &address).await?,
        _ => println!("Invalid selection"),
    }

    Ok(())
}
