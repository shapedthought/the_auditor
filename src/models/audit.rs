use serde::Deserialize;
use serde::Serialize;

use super::group::Result as GroupResult;
use super::user::Result as UserResult;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<User2>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<Group>,
}

impl From<UserResult> for AuditItem {
    fn from(user: UserResult) -> Self {
        AuditItem {
            id: None,
            type_field: user.type_field,
            user: Some(User2 {
                links: None,
                display_name: user.display_name,
                id: user.id,
                location_type: user.location_type,
                name: user.name,
            }),
            group: None,
        }
    }
}

impl From<GroupResult> for AuditItem {
    fn from(group: GroupResult) -> Self {
        AuditItem {
            id: None,
            type_field: "Group".to_string(),
            user: None,
            group: Some(Group {
                display_name: group.display_name.clone(),
                id: group.id,
                type_field: "Office365".to_string(),
                location_type: "Cloud".to_string(),
                name: group.display_name,
            }),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Links {
    pub organization: Organization,
    pub user: User,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Organization {
    pub href: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub href: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User2 {
    #[serde(rename = "_links")]
    pub links: Option<Links2>,
    pub display_name: String,
    pub id: String,
    pub location_type: String,
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub display_name: String,
    pub id: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub location_type: String,
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Links2 {}
