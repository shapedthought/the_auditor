use std::fs::{self, File};

use dialoguer::{Confirm, Select, MultiSelect};
use reqwest::Client;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{modifiers::UTF8_SOLID_INNER_BORDERS, Table};
use vauth::{Profile, VProfile, build_url};
use anyhow::Result;

use crate::{login::{login, get_org_id}, models::{audit::AuditItem, self, group::ItemIds}, helpers::select_selection};

pub fn confirm_action() {
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

pub async fn get_audit_items(username: &String, address: &String) -> Result<()> {
    let mut profile = Profile::get_profile(VProfile::VB365);

    let client = login(username, address, &mut profile).await?;
    let id = get_org_id(address, &profile, &client).await?;

    let response = audit_items(&id, address, &profile, &client).await?;

    if response.is_empty() {
        println!("No audit items found");
        return Ok(());
    }

    let mut table = Table::new();

    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .apply_modifier(UTF8_SOLID_INNER_BORDERS)
        .set_header(vec![
            "Name",
            "ID",
            "Type",
        ]);

    for i in response {
        let mut name = String::new();
        let mut id = String::new();
        if let Some(user) = i.user {
            let short_id: Vec<&str> = user.id.split(":").collect();
            name = user.display_name;
            id = short_id[3].to_owned();
        }
        else if let Some(group) = i.group {
            let short_id: Vec<&str> = group.id.split(":").collect();
            name = group.display_name;
            id = short_id[3].to_owned();
        }

        table.add_row(vec![
            name,
            id,
            i.type_field,
        ]);
    }
    print!("{table}");
    Ok(())
}

pub async fn audit_items(
    id: &String,
    address: &String,
    profile: &Profile,
    client: &Client,
) -> Result<Vec<AuditItem>, anyhow::Error> {
    let audit_string = format!("Organizations/{}/AuditItems", id);
    let audit_url = build_url(address, &audit_string, profile)?;
    let response: Vec<AuditItem> = client.get(&audit_url).send().await?.json().await?;
    Ok(response)
}

pub async fn add_audit_items(username: &String, address: &String) -> Result<()> {
    let mut profile = Profile::get_profile(VProfile::VB365);

    let client = login(username, address, &mut profile).await?;
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
    let url = build_url(address, &url_str, &profile)?;

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

pub async fn remove_item(username: &String, address: &String) -> Result<()> {
    let mut profile = Profile::get_profile(VProfile::VB365);

    let client = login(username, address, &mut profile).await?;
    let id = get_org_id(address, &profile, &client).await?;

    let response = audit_items(&id, address, &profile, &client).await?;

    let choices = ["Users", "Groups"];

    let type_selected = Select::new()
        .with_prompt("Select item to remove")
        .items(&choices)
        .interact()?;


    let selections: Vec<String> = if type_selected == 0 {
        response
            .iter()
            .filter(|x| x.user.is_some())
            .map(|x| x.user.as_ref().unwrap().clone())
            .map(|x| x.display_name)
            .collect::<Vec<String>>()
    } else {
        response
            .iter()
            .filter(|x| x.group.is_some())
            .map(|x| x.group.as_ref().unwrap().clone())
            .map(|x| x.display_name)
            .collect::<Vec<String>>()
    };

    if selections.is_empty() {
        println!("No items found");
        return Ok(());
    }

    let type_string = if type_selected == 0 {
        "user"
    } else {
        "group"
    };

    let ml_prompt = format!("Select {} to remove", type_string);

    let multi_select = MultiSelect::new()
        .with_prompt(ml_prompt)
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
        address,
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

pub async fn get_users_groups(username: &String, address: &String) -> Result<()> {
    let mut profile = Profile::get_profile(VProfile::VB365);

    let client = login(username, address, &mut profile).await?;
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
            let users_url = build_url(address, &user_string, &profile)?;
            let users: models::user::User = client.get(&users_url).send().await?.json().await?;
            let file = File::create("users.json")?;
            serde_json::to_writer_pretty(file, &users)?;
            print!("Users saved to users.json")
        }
        1 => {
            let group_string = format!("Organizations/{}/Groups", org_id);
            let groups_url = build_url(address, &group_string, &profile)?;
            let groups: models::group::Group = client.get(&groups_url).send().await?.json().await?;
            let file = File::create("groups.json")?;
            serde_json::to_writer_pretty(file, &groups)?;
            print!("Groups saved to groups.json")
        }
        _ => println!("Invalid selection"),
    };
    Ok(())
}