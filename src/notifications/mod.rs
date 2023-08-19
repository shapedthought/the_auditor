use reqwest::header::{HeaderMap, CONTENT_LENGTH};
use vauth::{Profile, VProfile, build_url};
use anyhow::Result;

use crate::{models::{config::Config, notification::NotificationData}, login, setup::set_up_auth};

pub async fn setup_notifications(username: &String, address: &String, config: Config) -> Result<()> {
    let mut profile = Profile::get_profile(VProfile::VB365);
    let client = login(username, address, &mut profile).await?;
    println!("Logged in successfully!");
    let complete_response = set_up_auth(&config, address, &profile, &client).await?;

    let nd = NotificationData {
        enable_notification: true,
        authentication_type: "Microsoft365".to_string(),
        to: config.notification.to,
        from: config.notification.from,
        subject: config.notification.subject,
        user_id: config.notification.user_id,
        request_id: complete_response.request_id,
    };

    let url = build_url(address, &"AuditEmailSettings".to_string(), &profile)?;

    let response = client.put(&url).json(&nd).send().await?;

    if response.status().is_success() {
        println!("Notification settings updated successfully!");
    } else {
        println!("Notification settings update failed!");
    }

    Ok(())
}

pub async fn sent_test_email(username: &String, address: &String) -> Result<()> {
    let mut profile = Profile::get_profile(VProfile::VB365);

    let client = login(username, address, &mut profile).await?;

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_LENGTH, "0".parse().unwrap());

    let url_string = build_url(
        address,
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