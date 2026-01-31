use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub matched_user: Option<MatchedUser>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchedUser {
    pub github: Option<String>,
    pub twitter: Option<String>,
    pub profile: Profile,
    pub submit_stats_global: SubmitStats,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub ranking: u32,
    pub reputation: i32,
    pub user_avatar: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitStats {
    pub ac_submission_num: Vec<SubmissionCount>,
}

#[derive(Debug, Deserialize)]
pub struct SubmissionCount {
    pub difficulty: ProfileDifficulty,
    pub count: u32,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub enum ProfileDifficulty {
    Easy,
    Medium,
    Hard,
    All,
}
