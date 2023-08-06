use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub azure: Azure,
    pub notification: Notification,
    pub vb365: Vb356,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Azure {
    pub tenant_id: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Notification {
    pub username: String,
    pub from: String,
    pub to: String,
    pub subject: String,
    pub user_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vb356 {
    pub username: String,
    pub address: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoginExtended {
    pub access_token: String,
    pub token_type: String,
    pub refresh_token: String,
    pub expires_in: i32,
    pub expires_on: String,
}
