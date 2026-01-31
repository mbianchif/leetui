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

    let test_cases = "[2,7,11,15]\n9\n[3,2,4]\n6";

    let res = api
        .run_tests("two-sum", "1", "rust", code, test_cases)
        .await
        .unwrap();

    println!("{res:?}");

    let check = loop {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let check = api.check_test_cases(&res).await.unwrap();

        match check.state {
            api::SubmissionState::Success => break check,
            _ => {}
        }
    };

    println!("{check:#?}");
    Ok(())
}
