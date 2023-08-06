use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrgItem {
    pub is_teams_online: bool,
    pub is_teams_chats_online: bool,
    pub exchange_online_settings: ExchangeOnlineSettings,
    pub share_point_online_settings: SharePointOnlineSettings,
    pub is_exchange_online: bool,
    pub is_share_point_online: bool,
    #[serde(rename = "type")]
    pub type_field: String,
    pub region: String,
    pub id: String,
    pub name: String,
    pub office_name: String,
    #[serde(rename = "_links")]
    pub links: Links,
    #[serde(rename = "_actions")]
    pub actions: Actions,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeOnlineSettings {
    pub use_application_only_auth: bool,
    pub account: String,
    pub grant_admin_access: bool,
    pub use_mfa: bool,
    pub application_id: String,
    pub application_certificate_thumbprint: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SharePointOnlineSettings {
    pub use_application_only_auth: bool,
    pub office_organization_name: String,
    pub share_point_save_all_web_parts: bool,
    pub account: String,
    pub grant_admin_access: bool,
    pub use_mfa: bool,
    pub application_id: String,
    pub application_certificate_thumbprint: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Links {
    #[serde(rename = "self")]
    pub self_field: SelfField,
    pub jobs: Jobs,
    pub groups: Groups,
    pub users: Users,
    pub sites: Sites,
    pub teams: Teams,
    pub used_repositories: UsedRepositories,
    pub rbac_roles: RbacRoles,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelfField {
    pub href: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Jobs {
    pub href: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Groups {
    pub href: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Users {
    pub href: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sites {
    pub href: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Teams {
    pub href: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsedRepositories {
    pub href: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RbacRoles {
    pub href: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Actions {}
