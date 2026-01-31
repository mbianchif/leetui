use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GqlResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GqlError>>,
}

#[derive(Debug, Deserialize)]
pub struct GqlError {
    pub message: String,
}
