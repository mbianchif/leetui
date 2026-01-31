use std::env;

use api::LeetCodeClient;
use api::{Language, SubmissionState};

#[tokio::main]
async fn main() -> Result<(), &'static str> {
    let Ok(session) = env::var("LEETCODE_SESSION") else {
        return Err("LEETCODE_SESSION is not defined");
    };

    let Ok(csrf) = env::var("CSRF_TOKEN") else {
        return Err("CSRF_TOKEN is not defined");
    };

    let api = LeetCodeClient::new(session, csrf).unwrap();

    let code = r#"
use std::collections::HashMap;

impl Solution {
    pub fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {
        let n = nums.len();
        let mut seen = HashMap::with_capacity(n);

        for (i, x) in nums.into_iter().enumerate() {
            if let Some(j) = seen.get(&(target - x)) {
                return vec![i as i32, *j];
            }

            seen.insert(x, i as i32);
        }

        Vec::new()
    }
}"#;

    let res = api
        .submit_code("two-sum", "1", Language::Rust, code)
        .await
        .unwrap();

    println!("{res:?}");

    let check = loop {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let check = api.check_submission(res).await.unwrap();

        match check.state {
            SubmissionState::Success => break check,
            _ => {}
        }
    };

    println!("{check:#?}");
    Ok(())
}
