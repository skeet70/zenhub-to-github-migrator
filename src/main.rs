use anyhow::*;
use graphql_client::reqwest::post_graphql_blocking as post_graphql;
use reqwest::blocking::Client;
use std::{env, iter};

mod github;
mod zenhub;

fn parse_repo_name(repo_name: &str) -> Result<(&str, &str), anyhow::Error> {
    let mut parts = repo_name.split('/');
    match (parts.next(), parts.next()) {
        (Some(owner), Some(name)) => Ok((owner, name)),
        _ => Err(format_err!("wrong format for the repository name param (we expect something like facebook/graphql)"))
    }
}

fn main() -> Result<(), anyhow::Error> {
    // currently think it needs Issues:RW, Pull Requests:RW, Issue Types:RW, and Projects:RW
    let github_api_token = env::var("GITHUB_TOKEN").expect("Missing GITHUB_TOKEN.");
    let zenhub_api_token = env::var("ZENHUB_TOKEN").expect("Missing ZENHUB_TOKEN.");

    // TODO(inputs)
    let repo_name = "IronCoreLabs/tenant-security-proxy";
    let issue_number = 699;

    let github_reqwest_client = Client::builder()
        .user_agent("zenhub-to-github-migrator/0.1.0")
        .default_headers(
            iter::once((
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", github_api_token))
                    .unwrap(),
            ))
            .collect(),
        )
        .build()?;

    let (owner, name) = parse_repo_name(&repo_name)?;
    let variables = github::get_issue_or_pr::Variables {
        repo: name.to_string(),
        owner: owner.to_string(),
        number: issue_number,
    };
    let response_body =
        post_graphql::<github::GetIssueOrPr, _>(&github_reqwest_client, github::URL, variables)?;
    println!("{:?}", response_body);
    let response_data: github::get_issue_or_pr::ResponseData =
        response_body.data.expect("expected data");
    let title = match response_data
        .repository
        .expect("missing repository")
        .issue_or_pull_request
        .expect("missing any node")
    {
        github::get_issue_or_pr::GetIssueOrPrRepositoryIssueOrPullRequest::Issue(issue) => {
            println!("{}", issue.title);
        }
        github::get_issue_or_pr::GetIssueOrPrRepositoryIssueOrPullRequest::PullRequest(pr) => {
            println!("{}", pr.title)
        }
    };
    Ok(())
}
