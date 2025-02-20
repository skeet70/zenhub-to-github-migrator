use reqwest::blocking::Client;
use std::{env, iter};

mod github;
mod zenhub;

fn parse_repo_name(repo_name: &str) -> Result<(&str, &str), anyhow::Error> {
    let mut parts = repo_name.split('/');
    match (parts.next(), parts.next()) {
        (Some(owner), Some(name)) => Ok((owner, name)),
        _ => Err(anyhow::anyhow!("wrong format for the repository name param (we expect something like facebook/graphql)"))
    }
}

fn parse_project_id(project_url: &str) -> Result<i64, anyhow::Error> {
    project_url
        .split("projects/")
        .last()
        .and_then(|v| v.parse::<i64>().ok())
        .ok_or_else(|| anyhow::anyhow!("Provided project URL didn't end with an ID."))
}

fn main() -> Result<(), anyhow::Error> {
    // currently think it needs Issues:RW, Pull Requests:RW, Issue Types:RW, and Projects:RW
    let github_api_token = env::var("GITHUB_TOKEN").expect("Missing GITHUB_TOKEN.");
    let zenhub_api_token = env::var("ZENHUB_TOKEN").expect("Missing ZENHUB_TOKEN.");

    // TODO(murph): inputs. Since mappings are required, may want to require passing a config file
    let (organization, repo_name) = parse_repo_name("IronCoreLabs/tenant-security-proxy")?;
    let issue_number = 699;
    let fields = vec![
        ("Estimate", "Estimate"),
        ("Priority", "Priority"),
        ("Pipeline", "Status"),
        ("Linked Issues", "Text"),
        ("Blocking", "Text"),
        // ("Sprint", "Iteration"), don't pull this over
    ];
    let project_number = parse_project_id("https://github.com/orgs/IronCoreLabs/projects/8")?;
    let workspace_name = "🍻 The Big Board 🌯";

    let github_client = Client::builder()
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

    let zenhub_client = Client::builder()
        .user_agent("zenhub-to-github-migrator/0.1.0")
        .default_headers(
            iter::once((
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", zenhub_api_token))
                    .unwrap(),
            ))
            .collect(),
        )
        .build()?;

    let zenhub_workspace = dbg!(zenhub::get_workspace(zenhub_client, workspace_name)?);
    let github_project = dbg!(github::get_project_id(
        github_client,
        organization,
        project_number
    )?);
    // for each pipeline in zenhub workspace
    //   for each issue in the pipeline
    //     add the issue to the GH project
    //     set the GH fields per the mapping table

    Ok(())
}
