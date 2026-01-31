use std::env;

use api::LeetCodeClient;

#[tokio::main]
async fn main() -> Result<(), &'static str> {
    let Ok(session) = env::var("LEETCODE_SESSION") else {
        return Err("LEETCODE_SESSION is not defined");
    };

    let Ok(csrf) = env::var("CSRF_TOKEN") else {
        return Err("CSRF_TOKEN is not defined");
    };

    let api = LeetCodeClient::new(session, csrf).unwrap();

    let res = api.get_problem("two-sum").await.unwrap();
    println!("{res:?}");

    Ok(())
}
