use graphql_client::{reqwest::post_graphql_blocking as post_graphql, GraphQLQuery};
use reqwest::blocking::Client;

pub const URL: &str = "https://api.github.com/graphql";

type URI = String;
type Date = String;
type DateTime = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemas/github.graphql",
    query_path = "queries/github/get_issue_or_pr.graphql",
    response_derives = "Debug"
)]
pub struct GetIssueOrPr;

pub fn get_issue_or_pr(
    client: Client,
    organization: &str,
    repo_name: &str,
    issue_number: i64,
) -> Result<(), anyhow::Error> {
    use get_issue_or_pr::*;

    let variables = Variables {
        repo: repo_name.to_string(),
        owner: organization.to_string(),
        number: issue_number,
    };
    let response_body = post_graphql::<GetIssueOrPr, _>(&client, URL, variables)?;
    println!("{:?}", response_body);
    let response_data: ResponseData = response_body.data.expect("expected data");
    let title = match response_data
        .repository
        .expect("missing repository")
        .issue_or_pull_request
        .expect("missing any node")
    {
        GetIssueOrPrRepositoryIssueOrPullRequest::Issue(issue) => {
            println!("{}", issue.title);
        }
        GetIssueOrPrRepositoryIssueOrPullRequest::PullRequest(pr) => {
            println!("{}", pr.title)
        }
    };

    Ok(())
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemas/github.graphql",
    query_path = "queries/github/get_project.graphql",
    response_derives = "Debug"
)]
pub struct GetProject;

pub fn get_project_id(
    client: Client,
    organization: &str,
    project_number: i64,
) -> Result<String, anyhow::Error> {
    use get_project::*;

    let variables = Variables {
        project_number,
        organization: organization.to_string(),
    };
    let response_body = post_graphql::<GetProject, _>(&client, URL, variables)?;
    let response_data: ResponseData = response_body.data.expect("Expected GH field data.");
    Ok(response_data
        .organization
        .expect("The organization does not exist.")
        .project_v2
        .expect("The project does not exist.")
        .id)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemas/github.graphql",
    query_path = "queries/github/get_fields.graphql",
    response_derives = "Debug"
)]
pub struct GetFields;

pub fn get_fields(
    client: Client,
    project_id: String,
) -> Result<get_fields::GetFieldsNode, anyhow::Error> {
    use get_fields::*;

    let variables = Variables {
        project_id: project_id.to_string(),
    };
    let response_body = post_graphql::<GetFields, _>(&client, URL, variables)?;
    println!("{:?}", response_body);
    let response_data: ResponseData = response_body.data.expect("Expected GH field data.");
    Ok(dbg!(response_data.node.expect(
        "There were no fields on the provided GH project."
    )))
}
