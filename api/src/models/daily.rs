use serde::Deserialize;

use super::ProblemSummary;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyChallengeOuter {
    pub active_daily_coding_challenge_question: DailyChallenge,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyChallenge {
    pub date: String,
    pub question: ProblemSummary,
}
