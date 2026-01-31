use serde::Deserialize;
use serde_with::{json::JsonString, serde_as};

#[derive(Debug, Deserialize)]
pub struct QuestionOuter {
    pub question: Option<Question>,
}

#[serde_as]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Question {
    pub question_id: String,
    pub title: String,
    pub title_slug: String,
    pub content: String,
    pub difficulty: Difficulty,
    pub code_snippets: Vec<CodeSnippet>,
    pub example_test_cases: String,
    pub sample_test_case: String,
    #[serde_as(as = "JsonString")]
    pub meta_data: MetaData,
}

#[derive(Debug, Deserialize)]
pub struct MetaData {
    pub name: String,
    pub params: Vec<Param>,
    #[serde(rename = "return")]
    pub return_type: ReturnType,
}

#[derive(Debug, Deserialize)]
pub struct Param {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
}

#[derive(Debug, Deserialize)]
pub struct ReturnType {
    #[serde(rename = "type")]
    pub inner: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeSnippet {
    pub lang: String,
    pub lang_slug: String,
    pub code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionListOuter {
    pub problemset_question_list: ProblemsetQuestionList,
}

#[derive(Debug, Deserialize)]
pub struct ProblemsetQuestionList {
    pub total: i32,
    pub questions: Vec<ProblemSummary>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProblemSummary {
    pub ac_rate: f64,
    pub difficulty: Difficulty,
    pub frontend_question_id: String,
    pub is_favor: bool,
    pub paid_only: bool,
    pub status: Option<String>,
    pub title: String,
    pub title_slug: String,
    pub topic_tags: Vec<TopicTag>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub enum ProblemStatus {
    #[serde(rename = "ac")]
    Accepted,
    #[serde(rename = "notac")]
    Attempted,
}

#[derive(Debug, Deserialize)]
pub struct TopicTag {
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum Language {
    Cpp,
    Java,
    Python,
    Python3,
    Csharp,
    Javascript,
    Ruby,
    Swift,
    Golang,
    Scala,
    Kotlin,
    Rust,
    Php,
    Typescript,
    Racket,
    Erlang,
    Elixir,
    Dart,
    #[serde(other)]
    Unknown,
}
