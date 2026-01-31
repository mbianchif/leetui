use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyChallengeOuter {
    pub active_daily_coding_challenge_question: DailyChallenge,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyChallenge {
    pub date: String,
    pub user_status: DailyUserStatus,
    pub question: DailyQuestionSummary,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum DailyUserStatus {
    NotStart,
    Finished,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyQuestionSummary {
    pub title: String,
    pub title_slug: String,
    pub difficulty: String,
}
