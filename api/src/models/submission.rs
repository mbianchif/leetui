use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SubmissionResponse {
    pub submission_id: u32,
}

#[derive(Debug, Deserialize)]
pub struct TestResponse {
    pub interpret_id: String,
}

#[derive(Debug, Deserialize)]
pub struct SubmissionCheckResponse {
    pub state: String,
    pub status_msg: Option<String>,
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
