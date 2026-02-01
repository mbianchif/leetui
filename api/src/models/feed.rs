use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalData {
    pub user_status: UserStatus,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStatus {
    pub username: String,
    pub is_premium: bool,
    pub is_signed_in: bool,
}
