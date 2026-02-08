use std::fmt::{self, Display};

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
    pub example_testcases: String,
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
    pub lang: Language,
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
    pub status: Option<ProblemStatus>,
    pub title: String,
    pub title_slug: String,
    pub topic_tags: Vec<TopicTag>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
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

#[derive(Debug, Deserialize, Default, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "PascalCase")]
pub enum Language {
    C,
    #[serde(rename = "C++")]
    Cpp,
    Java,
    Python,
    Python3,
    #[serde(rename = "C#")]
    Csharp,
    JavaScript,
    Ruby,
    Swift,
    Go,
    Scala,
    Kotlin,
    Rust,
    #[serde(rename = "PHP")]
    Php,
    TypeScript,
    Racket,
    Erlang,
    Elixir,
    Dart,
    #[default]
    #[serde(other)]
    Unknown,
}

impl Language {
    pub fn ext(&self) -> &'static str {
        match self {
            Language::C => "c",
            Language::Cpp => "cpp",
            Language::Java => "java",
            Language::Python | Language::Python3 => "py",
            Language::Csharp => "cs",
            Language::JavaScript => "js",
            Language::Ruby => "rb",
            Language::Swift => "swift",
            Language::Go => "go",
            Language::Scala => "scala",
            Language::Kotlin => "kt",
            Language::Rust => "rs",
            Language::Php => "php",
            Language::TypeScript => "ts",
            Language::Racket => "rkt",
            Language::Erlang => "erl",
            Language::Elixir => "ex",
            Language::Dart => "dart",
            Language::Unknown => "txt",
        }
    }

    pub fn from_ext(ext: &str) -> Option<Self> {
        let lang = match ext {
            "c" => Language::C,
            "cpp" => Language::Cpp,
            "java" => Language::Java,
            "py" => Language::Python3,
            "cs" => Language::Csharp,
            "js" => Language::JavaScript,
            "rb" => Language::Ruby,
            "swift" => Language::Swift,
            "go" => Language::Go,
            "scala" => Language::Scala,
            "kt" => Language::Kotlin,
            "rs" => Language::Rust,
            "php" => Language::Php,
            "ts" => Language::TypeScript,
            "rkt" => Language::Racket,
            "erl" => Language::Erlang,
            "ex" => Language::Elixir,
            "dart" => Language::Dart,
            "txt" => Language::Unknown,
            _ => return None,
        };

        Some(lang)
    }
}

impl Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Language::C => "C",
            Language::Cpp => "C++",
            Language::Java => "Java",
            Language::Python => "Python",
            Language::Python3 => "Python3",
            Language::Csharp => "C#",
            Language::JavaScript => "JavaScript",
            Language::Ruby => "Ruby",
            Language::Swift => "Swift",
            Language::Go => "Go",
            Language::Scala => "Scala",
            Language::Kotlin => "Kotlin",
            Language::Rust => "Rust",
            Language::Php => "PHP",
            Language::TypeScript => "TypeScript",
            Language::Racket => "Racket",
            Language::Erlang => "Erlang",
            Language::Elixir => "Elixir",
            Language::Dart => "Dart",
            Language::Unknown => "Unknown",
        };

        f.write_str(s)
    }
}
