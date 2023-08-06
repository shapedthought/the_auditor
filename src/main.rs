use dialoguer::{
    console::Term, theme::ColorfulTheme, Confirm, Input, MultiSelect, Password, Select,
};
use regex::Regex;
use reqwest::header::{CONTENT_LENGTH, HeaderMap};
use std::convert::From;
use std::{
    fs::{self, File},
    io::Write,
};
use vauth::{build_auth_headers, build_url, Profile, VClientBuilder, VProfile};
use webbrowser;
mod models;
use anyhow::Result;
use chrono::{DateTime, Duration, Local, Utc};
use models::{
    audit::AuditItem,
    config::{Config, LoginExtended},
    notification::NotificationData,
    org::OrgItem,
};
use reqwest::Client;

use crate::models::{
    group::ItemIds,
    oauth::{AuthRequest, AuthResponse, CompleteRequest, CompleteResponse},
};

async fn login(username: &String, address: &String, profile: &mut Profile) -> Result<Client> {
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

                return Ok(client);
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

async fn login_full(
    address: &String,
    username: &String,
    profile: &mut Profile,
    reason: String,
) -> Result<Client> {
    println!("{}", reason);
    let (client, login_response) = VClientBuilder::new(&address, username.to_string())
        .insecure()
        .timeout(60)
        .build(profile)
        .await
        .unwrap();
    let now = Utc::now();
    let expires_on = now + Duration::seconds(login_response.expires_in.clone() as i64);
    let expires_on_string = expires_on.to_rfc3339();
    save_token_file(login_response, expires_on_string)?;
    return Ok(client);
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
    let mut json_file = fs::File::create(&"token.json".to_string())?;
    let token_string = serde_json::to_string_pretty(&login_expended)?;
    json_file.write_all(token_string.as_bytes())?;
    Ok(())
}

//allow dead code for now
#[allow(dead_code)]
enum PromptOptions {
    String,
    Password,
}

#[allow(dead_code)]
fn prompt_info(prompt: &str, p_opt: PromptOptions) -> Result<String> {
    let input: String;

    match p_opt {
        PromptOptions::String => {
            input = Input::new().with_prompt(prompt).interact_text()?;
        }
        PromptOptions::Password => {
            input = Password::new()
                .with_prompt(prompt)
                .with_confirmation("Confirm password", "Passwords mismatching")
                .interact()?;
        }
    }

    Ok(input)
}

async fn setup_notifications(username: &String, address: &String, config: Config) -> Result<()> {
    let mut profile = Profile::get_profile(VProfile::VB365);
    let client = login(&username, &address, &mut profile).await?;
    println!("Logged in successfully!");

    let complete_response = set_up_auth(&config, address, &profile, &client).await?;

    let nd = NotificationData {
        enable_notification: true,
        authentication_type: "Microsoft365".to_string(),
        use_authentication: true,
        username: config.notification.user_id.clone(),
        use_ssl: true,
        to: config.notification.to,
        from: config.notification.from,
        subject: config.notification.subject,
        user_id: config.notification.user_id,
        request_id: complete_response.request_id,
    };

    let url = build_url(&address, &"AuditEmailSettings".to_string(), &profile)?;

    let response = client.put(&url).json(&nd).send().await?;

    if response.status().is_success() {
        println!("Notification settings updated successfully!");
    } else {
        println!("Notification settings update failed!");
    }

    Ok(())
}

async fn set_up_auth(
    config: &Config,
    address: &String,
    profile: &Profile,
    client: &Client,
) -> Result<CompleteResponse, anyhow::Error> {
    let auth_request = AuthRequest {
        authentication_service_kind: "Microsoft365".to_string(),
        client_id: config.azure.client_id.clone(),
        client_secret: config.azure.client_secret.clone(),
        tenant_id: config.azure.tenant_id.clone(),
        redirect_url: config.azure.redirect_url.clone(),
    };
    println!("{:#?}", auth_request);
    let url = build_url(
        &address,
        &"AuditEmailSettings/PrepareOAuthSignIn".to_string(),
        profile,
    )?;
    let response = client
        .post(&url)
        .json(&auth_request)
        .send()
        .await?
        .json::<AuthResponse>()
        .await?;

    println!("Opening browser to sign in...");
    webbrowser::open(&response.sign_in_url)?;
    let mut url_file = fs::File::create(&"callback.txt".to_string())?;
    url_file.write_all(b"replace this text")?;

    println!("Please sign in and copy the URL you are redirected to into a file called callback.txt you will find in the same directory as this executable");
    press_btn_continue::wait("Press any key to continue...\n").unwrap();
    let url_string = fs::read_to_string("callback.txt")?;
    if url_string.is_empty() {
        return Err(anyhow::anyhow!("URL file is empty"));
    }
    let pattern = r"=([^&]+)";
    let regex = Regex::new(pattern).unwrap();
    let matches: Vec<&str> = regex
        .captures_iter(&url_string)
        .map(|capture| capture.get(1).unwrap().as_str())
        .collect();
    if matches.len() != 3 {
        return Err(anyhow::anyhow!("Invalid URL"));
    }
    let complete_request = CompleteRequest {
        code: matches[0].to_string(),
        state: matches[1].to_string(),
    };
    let url_string = build_url(
        &address,
        &"AuditEmailSettings/CompleteOAuthSignIn".to_string(),
        profile,
    )?;
    let complete_response: CompleteResponse = client
        .post(&url_string)
        .json(&complete_request)
        .send()
        .await?
        .json()
        .await?;
    Ok(complete_response)
}

async fn remove_item(username: &String, address: &String) -> Result<()> {
    let mut profile = Profile::get_profile(VProfile::VB365);

    let client = login(&username, &address, &mut profile).await?;
    let id = get_org_id(address, &profile, &client).await?;

    let response = audit_items(&id, address, &profile, &client).await?;

    let choices = ["Users", "Groups"];

    let type_selected = Select::new()
        .with_prompt("Select item to remove")
        .items(&choices)
        .interact()?;

    let selections: Vec<String>;

    if type_selected == 0 {
        selections = response
            .iter()
            .filter(|x| x.user.is_some())
            .map(|x| x.user.as_ref().unwrap().clone())
            .map(|x| x.display_name.clone())
            .collect::<Vec<String>>();
    } else {
        selections = response
            .iter()
            .filter(|x| x.group.is_some())
            .map(|x| x.group.as_ref().unwrap().clone())
            .map(|x| x.display_name.clone())
            .collect::<Vec<String>>();
    }

    let multi_select = MultiSelect::new()
        .with_prompt("Select users to remove")
        .items(&selections)
        .interact()?;

    let mut selected_ids: Vec<String> = Vec::new();

    for (i, v) in response.iter().enumerate() {
        if multi_select.contains(&i) {
            if let Some(id) = &v.id {
                selected_ids.push(id.to_string());
            }
        }
    }

    if selected_ids.is_empty() {
        println!("No items selected");
        return Ok(());
    }
    println!("This will deleted the selected items");
    confirm_action();

    let url = build_url(
        &address,
        &format!("Organizations/{}/AuditItems/remove", id),
        &profile,
    )?;

    let item_ids = ItemIds {
        item_ids: selected_ids.clone(),
    };

    let response = client.post(url).json(&item_ids).send().await?;

    if response.status().is_success() {
        println!("Items deleted successfully!");
    } else {
        println!("Items deletion failed!");
        let response_text = response.text().await?;
        println!("{}", response_text);
    }

    Ok(())
}

async fn get_audit_items(username: &String, address: &String) -> Result<()> {
    let mut profile = Profile::get_profile(VProfile::VB365);

    let client = login(&username, &address, &mut profile).await?;
    let id = get_org_id(address, &profile, &client).await?;

    let response = audit_items(&id, address, &profile, &client).await?;

    if response.is_empty() {
        println!("No audit items found");
        return Ok(());
    }

    for i in response {
        if let Some(user) = i.user {
            println!("User: {}", user.display_name);
            println!("ID: {}", user.id);
        }
        if let Some(group) = i.group {
            println!("Group: {}", group.display_name);
        }

        println!("Field Type: {}", i.type_field);
        println!("");
    }
    Ok(())
}

async fn audit_items(
    id: &String,
    address: &String,
    profile: &Profile,
    client: &Client,
) -> Result<Vec<AuditItem>, anyhow::Error> {
    let audit_string = format!("Organizations/{}/AuditItems", id);
    let audit_url = build_url(&address, &audit_string, &profile)?;
    let response: Vec<AuditItem> = client.get(&audit_url).send().await?.json().await?;
    Ok(response)
}

async fn get_org_id(
    address: &String,
    profile: &Profile,
    client: &Client,
) -> Result<String, anyhow::Error> {
    let org_url = build_url(&address, &"Organizations".to_string(), profile)?;
    let response: Vec<OrgItem> = client.get(&org_url).send().await?.json().await?;
    let id: String;
    if response.len() > 1 {
        let selections: Vec<String> = response.iter().map(|x| x.name.clone()).collect();
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select Organization")
            .default(0)
            .items(&selections[..])
            .interact_on_opt(&Term::stderr())
            .unwrap()
            .unwrap();

        id = response[selection].id.clone();
    } else {
        id = response[0].id.clone();
    }
    Ok(id)
}

async fn sent_test_email(username: &String, address: &String) -> Result<()> {
    let mut profile = Profile::get_profile(VProfile::VB365);

    let client = login(&username, &address, &mut profile).await?;

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_LENGTH, "0".parse().unwrap());

    let url_string = build_url(
        &address,
        &"AuditEmailSettings/SendTest".to_string(),
        &profile,
    )?;

    println!("{}", url_string);

    let response = client.post(&url_string).headers(headers).send().await?;

    if response.status().is_success() {
        println!("Test email sent successfully!");
    } else {
        println!("Test email failed to send!");
        let response_text = response.text().await?;
        println!("{}", response_text);
    }

    Ok(())
}

fn select_selection(selections: &[&str], prompt: String) -> usize {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(&prompt)
        .default(0)
        .items(&selections[..])
        .interact_on_opt(&Term::stderr())
        .unwrap()
        .unwrap();
    selection
}

async fn get_users_groups(username: &String, address: &String) -> Result<()> {
    let mut profile = Profile::get_profile(VProfile::VB365);

    let client = login(&username, &address, &mut profile).await?;
    let org_id = get_org_id(address, &profile, &client).await?;

    let selections = &["Users", "Groups"];

    let selection = select_selection(selections, "Select Users or Groups".to_string());

    let type_string = if selection == 0 { "Users" } else { "Groups" };

    println!(
        "This will get the {} for the organization and save them to a file",
        type_string
    );
    confirm_action();

    match selection {
        0 => {
            let user_string = format!("Organizations/{}/Users", org_id);
            let users_url = build_url(&address, &user_string, &profile)?;
            let users: models::user::User = client.get(&users_url).send().await?.json().await?;
            let file = File::create("users.json")?;
            serde_json::to_writer_pretty(file, &users)?;
            print!("Users saved to users.json")
        }
        1 => {
            let group_string = format!("Organizations/{}/Groups", org_id);
            let groups_url = build_url(&address, &group_string, &profile)?;
            let groups: models::group::Group = client.get(&groups_url).send().await?.json().await?;
            let file = File::create("groups.json")?;
            serde_json::to_writer_pretty(file, &groups)?;
            print!("Groups saved to groups.json")
        }
        _ => println!("Invalid selection"),
    };
    Ok(())
}

async fn add_audit_items(username: &String, address: &String) -> Result<()> {
    let mut profile = Profile::get_profile(VProfile::VB365);

    let client = login(&username, &address, &mut profile).await?;
    let org_id = get_org_id(address, &profile, &client).await?;

    let selections = &["Users", "Groups"];

    let selection = select_selection(selections, "Select Users or Groups".to_string());

    let mut audit_items: Vec<AuditItem> = Vec::new();

    let supported_user_types = vec!["User", "Shared", "Public"];

    let supported_group_types = vec![
        "Office365",
        "Security",
        "Distribution",
        "DynamicDistribution",
    ];

    if selection == 0 {
        let file = serde_json::from_str::<models::user::User>(&fs::read_to_string("users.json")?)?;

        let users_to_add = file.results;
        println!("This will add the following users to the audit items:");
        for user in users_to_add.iter() {
            if supported_user_types.contains(&user.type_field.as_str()) {
                println!(" {}", user.display_name);
            }
        }
        confirm_action();
        for user in users_to_add.iter() {
            if supported_user_types.contains(&user.type_field.as_str()) {
                let audit_item = AuditItem::from(user.clone());
                audit_items.push(audit_item);
            }
        }
    } else if selection == 1 {
        let file =
            serde_json::from_str::<models::group::Group>(&fs::read_to_string("groups.json")?)?;
        let groups_to_add = file.results;
        println!("This will add the following groups to the audit items:");
        for group in groups_to_add.iter() {
            if supported_group_types.contains(&group.type_field.as_str()) {
                println!(" {}", group.display_name);
            }
        }
        confirm_action();
        for group in groups_to_add.iter() {
            if supported_group_types.contains(&group.type_field.as_str()) {
                let audit_item = AuditItem::from(group.clone());
                audit_items.push(audit_item);
            }
        }
    }
    let url_str = format!("Organizations/{org_id}/AuditItems");
    let url = build_url(&address, &url_str, &profile)?;

    let res = client.post(url).json(&audit_items).send().await?;

    let type_string = if selection == 0 { "Users" } else { "Groups" };

    if res.status().is_success() {
        println!("{} added successfully!", type_string);
    } else {
        println!("{} failed to add!",  type_string);
        let response_text = res.text().await?;
        println!("{}", response_text);
    }

    Ok(())
}

fn confirm_action() {
    if Confirm::new()
        .with_prompt("Do you want to continue?")
        .interact()
        .unwrap()
    {
        println!("Continuing...");
    } else {
        println!("Exiting...");
        std::process::exit(0);
    };
}

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
        "Setup/Reauthorize Notifications",
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
        2 => {
            remove_item(&username, &address).await?;
        }
        3 => {
            get_users_groups(&username, &address).await?;
        }
        4 => {
            setup_notifications(&username, &address, config).await?;
        }
        5 => {
            sent_test_email(&username, &address).await?;
        }
        _ => println!("Invalid selection"),
    }

    Ok(())
}
