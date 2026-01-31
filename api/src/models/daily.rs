use serde::Deserialize;

use super::TopicTag;

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
    pub link: String,
    pub question: DailyQuestionSummary,
}

#[derive(Debug, Deserialize)]
pub enum DailyUserStatus {
    NotStart,
    Finished,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyQuestionSummary {
    pub question_id: String,
    #[serde(rename = "questionFrontendId")]
    pub frontend_id: String,
    pub title: String,
    pub title_slug: String,
    pub difficulty: String,
    pub ac_rate: f64,
    pub is_paid_only: bool,
    pub topic_tags: Vec<TopicTag>,
}
