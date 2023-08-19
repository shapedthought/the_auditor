use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationData {
    pub enable_notification: bool,
    pub authentication_type: String,
    pub from: String,
    pub to: String,
    pub subject: String,
    pub user_id: String,
    pub request_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationResponse {
    pub message: String,
    pub error_code: String,
}
