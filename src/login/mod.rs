use std::{fs, io::Write};

use chrono::{Local, DateTime, Utc, Duration};
use dialoguer::{Select, theme::ColorfulTheme, console::Term};
use reqwest::Client;
use vauth::{Profile, build_auth_headers, VClientBuilder, build_url};
use anyhow::Result;
use crate::models::{config::LoginExtended, org::OrgItem};

pub async fn login(username: &String, address: &String, profile: &mut Profile) -> Result<Client> {
    let token_file = fs::read_to_string("token.json");

    match token_file {
        Ok(token_file) => {
            let token_struct: LoginExtended = serde_json::from_str(&token_file)?;
            let now = Local::now();
            let token_expires_on = DateTime::parse_from_rfc3339(&token_struct.expires_on)?;

            if now > token_expires_on {
                login_full(
                    address,
                    username,
                    profile,
                    "Token expired, logging in again.".to_string(),
                )
                .await
            } else {
                println!("Token is still valid, using it.");
                let auth_headers = build_auth_headers(&token_struct.access_token, profile);

                let client = reqwest::Client::builder()
                    .danger_accept_invalid_certs(true)
                    .default_headers(auth_headers)
                    .build()?;

                Ok(client)
            }
        }
        Err(_) => {
            login_full(
                address,
                username,
                profile,
                "No token file, logging in.".to_string(),
            )
            .await
        }
    }
}

pub async fn login_full(
    address: &String,
    username: &String,
    profile: &mut Profile,
    reason: String,
) -> Result<Client> {
    println!("{}", reason);
    let (client, login_response) = VClientBuilder::new(address, username.to_string())
        .insecure()
        .timeout(60)
        .build(profile)
        .await
        .unwrap();
    let now = Utc::now();
    let expires_on = now + Duration::seconds(login_response.expires_in as i64);
    let expires_on_string = expires_on.to_rfc3339();
    save_token_file(login_response, expires_on_string)?;
    Ok(client)
}

fn save_token_file(
    login_response: vauth::LoginResponse,
    expires_on_string: String,
) -> Result<(), anyhow::Error> {
    let login_expended = LoginExtended {
        access_token: login_response.access_token,
        token_type: login_response.token_type,
        refresh_token: login_response.refresh_token,
        expires_in: login_response.expires_in,
        expires_on: expires_on_string,
    };
    let mut json_file = fs::File::create("token.json")?;
    let token_string = serde_json::to_string_pretty(&login_expended)?;
    json_file.write_all(token_string.as_bytes())?;
    Ok(())
}

pub async fn get_org_id(
    address: &String,
    profile: &Profile,
    client: &Client,
) -> Result<String, anyhow::Error> {
    let org_url = build_url(address, &"Organizations".to_string(), profile)?;
    let response: Vec<OrgItem> = client.get(&org_url).send().await?.json().await?;
    let id: String = if response.len() > 1 {
        let selections: Vec<String> = response.iter().map(|x| x.name.clone()).collect();
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select Organization")
            .default(0)
            .items(&selections[..])
            .interact_on_opt(&Term::stderr())
            .unwrap()
            .unwrap();

        response[selection].id.clone()
    } else {
        response[0].id.clone()
    };
    Ok(id)
}