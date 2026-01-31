use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SubmissionResponse {
    pub submission_id: u32,
}

#[derive(Debug, Deserialize)]
pub struct SubmissionCheckResponse {
    pub state: SubmissionState,
    pub status_msg: Option<StatusMsg>,
    pub status_id: Option<i32>,
    pub question_id: Option<String>,
    pub task_finish_time: Option<i64>,
    pub run_success: bool,

    // Judging metadata
    pub total_correct: Option<u32>,
    pub total_testcases: Option<u32>,
    pub compare_result: Option<String>,

    // Efficiency Stats
    pub status_runtime: Option<String>,
    pub status_memory: Option<String>,
    pub runtime_percentile: Option<f64>,
    pub memory_percentile: Option<f64>,

    // Wrong answer detail
    pub input_formatted: Option<String>,
    pub expected_output: Option<String>,
    pub code_answer: Option<Vec<String>>,

    // Debugging
    pub std_output: Option<String>,
    pub full_compile_error: Option<String>,
    pub full_runtime_error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TestCasesResponse {
    pub interpret_id: String,
}

#[derive(Debug, Deserialize)]
pub struct TestCasesCheckResponse {
    pub state: SubmissionState,
    pub status_msg: Option<StatusMsg>,
    pub run_success: Option<bool>,

    // Batch Results
    pub code_answer: Option<Vec<String>>,
    pub expected_code_answer: Option<Vec<String>>,

    // Performance
    pub status_runtime: Option<String>,
    pub status_memory: Option<String>,

    // Counters
    pub total_correct: Option<u32>,
    pub total_testcases: Option<u32>,

    // Debugging
    pub std_output: Option<String>,
    pub full_compile_error: Option<String>,
    pub full_runtime_error: Option<String>,
    pub last_testcase: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum SubmissionState {
    Pending,
    Started,
    Success,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub enum StatusMsg {
    #[serde(rename = "Accepted")]
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
    #[serde(rename = "Internal Error")]
    InternalError,
    #[serde(other)]
    Unknown,
}
