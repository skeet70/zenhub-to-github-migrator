use anyhow::{anyhow, Error};
use reqwest::blocking::Client;
use std::{collections::HashMap, env, iter};

mod github;
mod zenhub;

fn parse_repo_name(repo_name: &str) -> Result<(&str, &str), Error> {
    let mut parts = repo_name.split('/');
    match (parts.next(), parts.next()) {
        (Some(owner), Some(name)) => Ok((owner, name)),
        _ => Err(anyhow!("wrong format for the repository name param (we expect something like facebook/graphql)"))
    }
}

fn parse_project_id(project_url: &str) -> Result<i64, Error> {
    project_url
        .split("projects/")
        .last()
        .and_then(|v| v.parse::<i64>().ok())
        .ok_or_else(|| anyhow!("Provided project URL didn't end with an ID."))
}

fn main() -> Result<(), Error> {
    // currently think it needs Issues:RW, Pull Requests:RW, Issue Types:RW, and Projects:RW
    let github_api_token = env::var("GITHUB_TOKEN").expect("Missing GITHUB_TOKEN.");
    let zenhub_api_token = env::var("ZENHUB_TOKEN").expect("Missing ZENHUB_TOKEN.");

    // TODO(murph): inputs. Since mappings are required, may want to require passing a config file
    let organization = "IronCoreLabs";
    // ZH -> GH
    let field_mapping = HashMap::from([
        // Currently supported fields
        ("Estimate", "Estimate"),
        ("Priority", "Priority"),
        ("Pipeline", "Status"),
        // Not yet supported fields
        // ("Linked Issues", "Text"),
        // ("Blocking", "Text"),
        // ("Sprint", "Iteration"), don't pull this over
    ]);
    // ZH -> GH
    let lane_mapping = HashMap::from([
        ("Ungroomed", "Ungroomed"),
        ("Tech Debt", "Tech Debt"),
        ("Backlog", "Backlog"),
        ("Next Sprint", "Backlog"),
        ("This Sprint", "Backlog"),
        ("In Progress", "In Progress"),
        ("Review", "Review"),
    ]);
    // TODO(murph): priority is hardcoded from "High priority" to P0 because that's all we have (if not null)

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

    let zenhub_workspace = dbg!(zenhub::get_workspace(
        zenhub_client.clone(),
        workspace_name
    )?);
    let github_project_id = dbg!(github::get_project_id(
        github_client.clone(),
        organization,
        project_number
    )?);
    let github_project_fields = dbg!(github::get_fields(
        github_client.clone(),
        &github_project_id
    ))?;

    for pipeline in zenhub_workspace.pipelines_connection.nodes {
        println!("Getting issues for Zenhub pipeline {}", pipeline.name);
        let mut issues = dbg!(zenhub::get_pipeline_issues(
            zenhub_client.clone(),
            &pipeline.id,
            &zenhub_workspace.id
        ))?;
        // lazy way to get issues in the same order in GH projects. Position APIs are possible from both if we need something better
        issues.reverse();
        println!("Adding issues for Zenhub pipeline {}", pipeline.name);
        for zh_issue in issues {
            println!(
                "Getting GitHub issue {}/{}#{}",
                &zh_issue.repository.owner.login, &zh_issue.repository.name, zh_issue.number
            );
            let maybe_gh_issue = github::get_issue_or_pr(
                github_client.clone(),
                &zh_issue.repository.owner.login,
                &zh_issue.repository.name,
                zh_issue.number,
            );
            match maybe_gh_issue {
                Err(e) => {
                    println!("Missing GH issue {}/{}#{}. Usually when this happens the issue or creator of it have been deleted. {e}", zh_issue.repository.owner.login, zh_issue.repository.name, zh_issue.number);
                    continue;
                }
                Ok(gh_issue) => {
                    let gh_item_id = match gh_issue {
                        github::get_issue_or_pr::GetIssueOrPrRepositoryIssueOrPullRequest::Issue(issue) => {
                            issue.id
                        }
                        github::get_issue_or_pr::GetIssueOrPrRepositoryIssueOrPullRequest::PullRequest(
                            pr,
                        ) => pr.id,
                    };
                    println!(
                        "Adding issue {}/{}#{} to project, GH item ID {}.",
                        &zh_issue.repository.owner.login,
                        &zh_issue.repository.name,
                        zh_issue.number,
                        gh_item_id
                    );
                    let gh_project_item_id =
                        github::add_item(github_client.clone(), &github_project_id, &gh_item_id)?;
                    println!(
                        "Item ID {} added to project, project ID {}.",
                        gh_item_id, gh_project_item_id
                    );
                    let estimate = zh_issue.estimate.map(|e| e.value);
                    let estimate_field_id =
                        zh_to_gh_field_id("Estimate", &field_mapping, &github_project_fields)?;
                    let priority_option_id = zh_to_gh_priority(
                        zh_issue
                            .pipeline_issue
                            .expect("ZH issue missing any priority field.")
                            .priority
                            .map(|p| p.name),
                        &field_mapping,
                        &github_project_fields,
                    )?;
                    // TODO(murph): on this and others can probably do this first or return it from the other method to save time.
                    let priority_field_id =
                        zh_to_gh_field_id("Priority", &field_mapping, &github_project_fields)?;
                    // status field based on current pipeline
                    let status_option_id = zh_to_gh_status_id(
                        &pipeline.name,
                        &lane_mapping,
                        &field_mapping,
                        &github_project_fields,
                    )?;
                    let status_field_id =
                        zh_to_gh_field_id("Pipeline", &field_mapping, &github_project_fields)?;
                    println!(
                        "Setting item {} estimate to {:?}",
                        gh_project_item_id, estimate
                    );
                    github::set_field_number(
                        github_client.clone(),
                        &github_project_id,
                        &gh_project_item_id,
                        &estimate_field_id,
                        estimate,
                    )?;
                    println!(
                        "Setting item {} priority to {:?}",
                        gh_project_item_id, priority_option_id
                    );
                    github::set_field_option(
                        github_client.clone(),
                        &github_project_id,
                        &gh_project_item_id,
                        &priority_field_id,
                        priority_option_id,
                    )?;
                    println!(
                        "Setting item {} status to {:?}",
                        gh_project_item_id, status_option_id
                    );
                    github::set_field_option(
                        github_client.clone(),
                        &github_project_id,
                        &gh_project_item_id,
                        &status_field_id,
                        Some(status_option_id),
                    )?;
                    // TODO(murph): move connected issues into sub-issues?
                }
            }
        }
    }

    Ok(())
}

// helper to get the GH field ID for a given ZH field name, via the mappings and the results of `get_fields`
fn zh_to_gh_field_id(
    zh_field_name: &str,
    field_mapping: &HashMap<&str, &str>,
    gh_fields: &Vec<github::get_fields::GetFieldsNodeOnProjectV2FieldsNodes>,
) -> Result<String, Error> {
    let gh_name = field_mapping.get(zh_field_name).ok_or_else(|| {
        anyhow!("Couldn't find ZH name {zh_field_name} in the field mapping configuration.")
    })?;
    gh_fields
        .iter()
        .find(|gh_field| match gh_field {
            github::get_fields::GetFieldsNodeOnProjectV2FieldsNodes::ProjectV2Field(f) => {
                &&f.name == gh_name
            }
            github::get_fields::GetFieldsNodeOnProjectV2FieldsNodes::ProjectV2SingleSelectField(
                ssf,
            ) => &&ssf.name == gh_name,
            _ => false,
        })
        .ok_or_else(|| anyhow!("Couldn't find GH mapped field {gh_name} in the GH project fields."))
        .and_then(|f| match f {
            github::get_fields::GetFieldsNodeOnProjectV2FieldsNodes::ProjectV2Field(f) => {
                Ok(f.id.clone())
            }
            github::get_fields::GetFieldsNodeOnProjectV2FieldsNodes::ProjectV2SingleSelectField(
                ssf,
            ) => Ok(ssf.id.clone()),
            _ => Err(anyhow!("Encountered something other than a field or single select field (iteration is not yet supported)!"))
        })
}

fn zh_to_gh_status_id(
    zh_name: &str,
    lane_mapping: &HashMap<&str, &str>,
    field_mapping: &HashMap<&str, &str>,
    gh_fields: &Vec<github::get_fields::GetFieldsNodeOnProjectV2FieldsNodes>,
) -> Result<String, Error> {
    let gh_status_field = field_mapping.get("Pipeline").ok_or_else(|| {
        anyhow!("Couldn't find ZH name {zh_name} in the field mapping configuration.")
    })?;
    let gh_status_name = lane_mapping.get(zh_name).ok_or_else(|| {
        anyhow!("Couldn't find ZH name {zh_name} in the lane mapping configuration.")
    })?;
    gh_fields
        .iter()
        .find(|gh_field| match gh_field {
            github::get_fields::GetFieldsNodeOnProjectV2FieldsNodes::ProjectV2SingleSelectField(
                ssf,
            ) => &&ssf.name == gh_status_field,
            _ => false,
        })
        .ok_or_else(|| {
            anyhow!(
                "Couldn't find GH mapped status field {gh_status_name} in the GH project fields."
            )
        })
        .and_then(|f| match f {
            github::get_fields::GetFieldsNodeOnProjectV2FieldsNodes::ProjectV2SingleSelectField(
                ssf,
            ) => {
                    let gh_status_option = ssf
                .options
                .iter()
                .find(|ssf_option| &&ssf_option.name == gh_status_name).ok_or_else(|| anyhow!("Couldn't find a GH status option {gh_status_name} to match the ZH pipeline {zh_name}"))?;
            Ok(gh_status_option.id.clone())
        },
            _ => Err(anyhow!(
                "Encountered something other than a single select field for GH's status field!"
            )),
        })
}

// This involves a lot of magic hardcoding for our expected situation
fn zh_to_gh_priority(
    priority: Option<String>,
    field_mapping: &HashMap<&str, &str>,
    gh_fields: &Vec<github::get_fields::GetFieldsNodeOnProjectV2FieldsNodes>,
) -> Result<Option<String>, Error> {
    let gh_priorities = gh_fields
        .iter()
        .find(|f| match f {
            github::get_fields::GetFieldsNodeOnProjectV2FieldsNodes::ProjectV2SingleSelectField(
                ssf,
            ) => {
                &&ssf.name
                    == field_mapping
                        .get("Priority")
                        .expect("Missing GH name for 'Priority' in the field mapping.")
            }
            _ => false,
        })
        .ok_or_else(|| anyhow!("Found no GH 'Priority' field."))
        .and_then(|v| match v {
            github::get_fields::GetFieldsNodeOnProjectV2FieldsNodes::ProjectV2SingleSelectField(
                ssf,
            ) => Ok(ssf.options.clone()),
            _ => Err(anyhow!("Found no GH options for the 'Priority' field.")),
        })?;
    let p = match priority {
        Some(p) if p == "High priority".to_string() => gh_priorities.iter().find_map(|p| {
            if p.name == "P0" {
                Some(p.id.clone())
            } else {
                None
            }
        }),
        _ => None,
    };

    Ok(p)
}
