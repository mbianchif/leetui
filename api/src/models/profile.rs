use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub matched_user: MatchedUser,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchedUser {
    pub github_url: Option<String>,
    pub twitter_url: Option<String>,
    pub profile: Profile,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub ranking: u32,
    pub reputation: i32,
    pub solution_count: usize,
}
