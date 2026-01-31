use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SubmissionResponse {
    pub submission_id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    C,
    CSharp,
    Cpp,
    #[serde(rename = "golang")]
    Go,
    Java,
    JavaScript,
    Kotlin,
    Php,
    Python2,
    Python3,
    Ruby,
    Rust,
    Scala,
    Swift,
    TypeScript,
}

#[derive(Debug, Deserialize)]
pub struct SubmissionCheckResponse {
    pub state: SubmissionState,
    pub status_msg: Option<String>, // "Accepted", "Wrong Answer", "Runtime Error", etc.
    pub total_correct: Option<u32>,
    pub total_testcases: Option<u32>,
    pub status_runtime: Option<String>,
    pub status_memory: Option<String>,
    pub runtime_percentile: Option<f64>,
    pub memory_percentile: Option<f64>,

    // Feedback for failures
    pub input_formatted: Option<String>,
    pub expected_output: Option<String>,
    pub code_answer: Option<Vec<String>>,
    pub std_output: Option<String>,
    pub full_compile_error: Option<String>,
    pub full_runtime_error: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SubmissionState {
    Started,
    Pending,
    Success,
}

#[derive(Debug, Deserialize)]
pub enum SubmissionStatusMsg {
    Accepted,
    #[serde(rename = "Wrong Answer")]
    WrongAnswer,
    #[serde(rename = "Compile Error")]
    CompileError,
    #[serde(rename = "Runtime Error")]
    RuntimeError,
    #[serde(rename = "Time Limit Exceeded")]
    TimeLimitExceeded,
    #[serde(rename = "Memory Limit Exceeded")]
    MemoryLimitExceeded,
    #[serde(rename = "Output Limit Exceeded")]
    OutputLimitExceeded,
    #[serde(other)]
    Unknown,
}
