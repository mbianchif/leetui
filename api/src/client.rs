use reqwest::{
    Client, Method,
    header::{CONTENT_TYPE, COOKIE, HeaderMap, HeaderValue, USER_AGENT},
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

use crate::{
    error::{LeetCodeErr, Result},
    models::{
        DailyChallenge, DailyChallengeOuter, GlobalData, GqlResponse, MatchedUser,
        ProblemsetQuestionList, Question, QuestionListOuter, QuestionOuter,
        SubmissionCheckResponse, SubmissionResponse, TestCasesCheckResponse, TestCasesResponse,
        UserProfile, UserStatus,
    },
};

const BASE_URL: &str = "https://leetcode.com";

/// The way to communicate with the LeetCode api.
pub struct LeetCodeClient {
    client: Client,
    session: String,
    csrf: String,
}

impl LeetCodeClient {
    /// Creates a new `LeetCodeClient` api client.
    ///
    /// # Arguments
    /// * `session` - The session string to use as header.
    /// * `csrf` - The csrf-token to use as header.
    pub fn new(session: String, csrf: String) -> Result<Self> {
        let mut headers = HeaderMap::new();

        headers.insert("x-csrftoken", HeaderValue::from_str(&csrf)?);
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("Referer", HeaderValue::from_static("https://leetcode.com"));
        headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36"));

        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self {
            client,
            session,
            csrf,
        })
    }

    /// Retrieves the status of the user.
    ///
    /// # Returns
    /// The status for this specific user.
    pub async fn get_status(&self) -> Result<UserStatus> {
        let query = include_str!("../queries/get_status.graphql");
        let data: GlobalData = self.request_graphql(query, {}).await?;
        Ok(data.user_status)
    }

    /// Retrieves the user's profile.
    ///
    /// # Arguments
    /// * `username` - The user's username.
    ///
    /// # Returns
    /// The user's details.
    pub async fn get_profile(&self, username: &str) -> Result<MatchedUser> {
        let query = include_str!("../queries/get_profile.graphql");
        let vars = json!({ "username": username });
        let data: UserProfile = self.request_graphql(query, vars).await?;
        let matched_user = data
            .matched_user
            .ok_or_else(|| LeetCodeErr::Api("Failed to find the user".into()))?;

        Ok(matched_user)
    }

    /// Retrieves the daily challenge.
    ///
    /// # Returns
    /// The daily problem.
    pub async fn get_daily_challenge(&self) -> Result<DailyChallenge> {
        let query = include_str!("../queries/get_daily_challenge.graphql");
        let data: DailyChallengeOuter = self.request_graphql(query, {}).await?;
        Ok(data.active_daily_coding_challenge_question)
    }

    /// Retrieves a problem.
    ///
    /// # Arguments
    /// * `title_slug` - The slug identifier for the question.
    ///
    /// # Returns
    /// The problem details or an error if either was
    /// not found or the HTML inside the content is invalid.
    pub async fn get_problem(&self, title_slug: &str) -> Result<Question> {
        let query = include_str!("../queries/get_problem.graphql");
        let vars = json!({ "titleSlug": title_slug });

        let data: QuestionOuter = self.request_graphql(query, vars).await?;
        let question = data
            .question
            .ok_or_else(|| LeetCodeErr::Api(format!("Question {title_slug} not found")))?;

        Ok(question)
    }

    /// Retrieves the question list.
    ///
    /// # Arguments
    /// * `skip` - The offset to start the list out of.
    /// * `limit` - The maximum amount of questions to retrieve at once.
    ///
    /// # Returns
    /// A list of questions.
    pub async fn get_problem_list(
        &self,
        skip: usize,
        limit: usize,
    ) -> Result<ProblemsetQuestionList> {
        let query = include_str!("../queries/get_problem_list.graphql");
        let vars = json!({
            "categorySlug": "",
            "skip": skip,
            "limit": limit,
            "filters": {},
        });

        let data: QuestionListOuter = self.request_graphql(query, vars).await?;
        Ok(data.problemset_question_list)
    }

    /// Runs the testing code for a certain problem.
    ///
    /// # Arguments
    /// * `slug` - The slug for the problem being solved.
    /// * `question_id` - The id of the question.
    /// * `lang` - The programming language used to write the code.
    /// * `code` - The code being submitted.
    /// * `test_cases` - The test cases to run.
    ///
    /// # Returns
    /// The interpret id for this submission.
    pub async fn run_tests(
        &self,
        slug: &str,
        question_id: &str,
        lang: &str,
        code: &str,
        test_cases: &str,
    ) -> Result<String> {
        let url = format!("{BASE_URL}/problems/{slug}/interpret_solution/");
        let body = json!({
            "lang": lang,
            "question_id": question_id,
            "typed_code": code,
            "data_input": test_cases,
        });

        let res: TestCasesResponse = self.raw_request(Method::POST, &url, body).await?;
        Ok(res.interpret_id)
    }

    /// Checks for the interpratation of test cases of a problem.
    ///
    /// # Arguments
    /// * `interpret_id` - The id of the testing session that wants to be checked.
    ///
    /// # Returns
    /// A test cases check result.
    pub async fn check_test_cases(&self, interpret_id: &str) -> Result<TestCasesCheckResponse> {
        let url = format!("{BASE_URL}/submissions/detail/{interpret_id}/check/");
        self.raw_request(Method::GET, &url, Value::Null).await
    }

    /// Submits a code solution for a problem.
    ///
    /// # Arguments
    /// * `slug` - The slug for the problem being solved.
    /// * `question_id` - The id of the question.
    /// * `lang` - The programming language used to write the code.
    /// * `code` - The code being submitted.
    ///
    /// # Returns
    /// The submission id for this submission.
    pub async fn submit_code(
        &self,
        slug: &str,
        question_id: &str,
        lang: &str,
        code: &str,
    ) -> Result<u32> {
        let url = format!("{BASE_URL}/problems/{slug}/submit/");
        let body = json!({
            "lang": lang,
            "question_id": question_id,
            "typed_code": code,
        });

        let res: SubmissionResponse = self.raw_request(Method::POST, &url, body).await?;
        Ok(res.submission_id)
    }

    /// Checks for the submission to a problem.
    ///
    /// # Arguments
    /// * `submission_id` - The id of the submission that wants to be checked.
    ///
    /// # Returns
    /// A submission check result.
    pub async fn check_submission(&self, submission_id: u32) -> Result<SubmissionCheckResponse> {
        let url = format!("{BASE_URL}/submissions/detail/{submission_id}/check/");
        self.raw_request(Method::GET, &url, Value::Null).await
    }

    /// Maes a GraphQL request to the `/graphql` endpoint.
    ///
    /// # Arguments
    /// * `query` - The GraphQL query.
    /// * `variables` - The variables to replace in the query.
    ///
    /// # Returns
    /// A result with the response's data.
    async fn request_graphql<V, T>(&self, query: &str, variables: V) -> Result<T>
    where
        V: Serialize,
        T: DeserializeOwned,
    {
        let url = format!("{BASE_URL}/graphql");
        self.request(&url, query, variables).await
    }

    /// Makes a GraphQL request to the leetcode api.
    ///
    /// # Arguments
    /// * `url` - The url of the request.
    /// * `query` - The GraphQL query.
    /// * `variables` - The variables to replace in the query.
    ///
    /// # Returns
    /// A result with the response's data.
    async fn request<V, T>(&self, url: &str, query: &str, variables: V) -> Result<T>
    where
        V: Serialize,
        T: DeserializeOwned,
    {
        let body = json!({ "query": query, "variables": variables });
        let body: GqlResponse<T> = self.raw_request(Method::POST, url, body).await?;

        if let Some(errors) = body.errors {
            let msg = errors
                .into_iter()
                .map(|e| e.message)
                .collect::<Vec<_>>()
                .join(" - ");

            return Err(LeetCodeErr::Api(msg));
        }

        body.data
            .ok_or_else(|| LeetCodeErr::Api("No data was returned from Leet Code".into()))
    }

    /// Makes the actual http request.
    ///
    /// # Args
    /// * `method` - The method.
    /// * `url` - The url.
    /// * `body` - The body.
    ///
    /// # Returns
    /// The data of the response.
    async fn raw_request<T: DeserializeOwned>(
        &self,
        method: Method,
        url: &str,
        body: Value,
    ) -> Result<T> {
        let Self {
            client,
            session,
            csrf,
        } = self;

        let cookie = format!("LEETCODE_SESSION={session}; csrftoken={csrf}");
        let mut req = client.request(method, url).header(COOKIE, cookie);

        if !body.is_null() {
            req = req.json(&body);
        }

        let res = req.send().await?;
        let status = res.status();
        let text = res.text().await?;

        if !status.is_success() {
            return Err(LeetCodeErr::Api(format!("Status: {status}\nError: {text}")));
        }

        serde_json::from_str(&text)
            .map_err(|e| LeetCodeErr::Api(format!("JSON Decode Error: {e}\nRaw Response: {text}")))
    }
}
