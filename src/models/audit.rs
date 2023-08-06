use serde::Deserialize;
use serde::Serialize;

use super::group::Result as GroupResult;
use super::user::Result as UserResult;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditItem {
    #[serde(rename = "_links")]
    pub links: Option<Links>,
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_field: String,
    pub user: Option<User2>,
    pub group: Option<Group>,
}

impl From<UserResult> for AuditItem {
    fn from(user: UserResult) -> Self {
        AuditItem {
            links: None,
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
            links: None,
            id: None,
            type_field: group.type_field,
            user: None,
            group: Some(Group {
                links: Links2 {},
                display_name: group.display_name,
                id: group.id,
                location_type: group.location_type,
                name: group.name,
                on_premises_sid: None,
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
    #[serde(rename = "_links")]
    pub links: Links2,
    pub display_name: String,
    pub id: String,
    pub location_type: String,
    pub name: String,
    pub on_premises_sid: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Links2 {}
