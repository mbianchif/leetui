use reqwest::{
    Client, Method,
    header::{CONTENT_TYPE, COOKIE, HeaderMap, HeaderValue, USER_AGENT},
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

use crate::{
    error::{LeetCodeErr, Result},
    models::{
        GlobalData, GqlResponse, MatchedUser, ProblemsetQuestionList, Question, QuestionListOuter,
        QuestionOuter, UserProfile, UserStatus,
    },
    utils::convert_to_markdown,
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
        Ok(data.matched_user)
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
        let mut question = data
            .question
            .ok_or_else(|| LeetCodeErr::Api(format!("Question {title_slug} not found")))?;

        question.content = convert_to_markdown(&question.content)?;
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
            "filters": ""
        });

        let data: QuestionListOuter = self.request_graphql(query, vars).await?;
        Ok(data.problemset_question_list)
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
        self.raw_request(Method::POST, url, body).await
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

        let req = client
            .request(method, url)
            .header(
                COOKIE,
                format!("LEETCODE_SESSION={session}; csrftoken={csrf}"),
            )
            .json(&body);

        let res = req.send().await?;
        let body: GqlResponse<T> = res.json().await?;

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
}
