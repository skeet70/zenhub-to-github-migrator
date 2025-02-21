use anyhow::{anyhow, Error};
use graphql_client::{reqwest::post_graphql_blocking as post_graphql, GraphQLQuery};
use reqwest::blocking::Client;

pub const URL: &str = "https://api.github.com/graphql";

type URI = String;
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
) -> Result<get_issue_or_pr::GetIssueOrPrRepositoryIssueOrPullRequest, Error> {
    use get_issue_or_pr::*;

    let variables = Variables {
        repo: repo_name.to_string(),
        owner: organization.to_string(),
        number: issue_number,
    };
    let response_body = post_graphql::<GetIssueOrPr, _>(&client, URL, variables)?;
    let response_data: ResponseData = response_body
        .data
        .expect("Expected data in the get issue/pr response from GH.");
    let response_repo = response_data
        .repository
        .ok_or_else(|| anyhow!("missing repository"))?;
    response_repo
        .issue_or_pull_request
        .ok_or_else(|| anyhow!("missing any node"))
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
) -> Result<String, Error> {
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
    response_derives = "Debug, Clone"
)]
pub struct GetFields;

pub fn get_fields(
    client: Client,
    project_id: &str,
) -> Result<Vec<get_fields::GetFieldsNodeOnProjectV2FieldsNodes>, Error> {
    use get_fields::*;

    let variables = Variables {
        project_id: project_id.to_string(),
    };
    let response_body = post_graphql::<GetFields, _>(&client, URL, variables)?;
    let response_data: ResponseData = response_body.data.expect("Expected GH field data.");
    match response_data
        .node
        .expect("Expected GH field nodes back in response.")
    {
        GetFieldsNode::ProjectV2(fields) => Ok(fields
            .fields
            .nodes
            .expect("Found no fields for the given GH project.")
            .into_iter()
            .flatten()
            .collect()),
        _ => Err(anyhow::anyhow!(
            "Recieved non-ProjectV2 fields back from get fields request to GH."
        )),
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemas/github.graphql",
    query_path = "queries/github/add_item.graphql",
    response_derives = "Debug"
)]
pub struct AddItem;

pub fn add_item(client: Client, project_id: &str, issue_id: &str) -> Result<String, Error> {
    use add_item::*;

    let variables = Variables {
        project_id: project_id.to_string(),
        issue_id: issue_id.to_string(),
    };
    let response_body = post_graphql::<AddItem, _>(&client, URL, variables)?;
    let response_data: ResponseData = response_body.data.expect("Expected ID for added GH item.");
    Ok(response_data
        .add_project_v2_item_by_id
        .expect("GH add item response is missing.")
        .item
        .expect("GH item add response is missing the item.")
        .id)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemas/github.graphql",
    query_path = "queries/github/set_field_value.graphql",
    response_derives = "Debug"
)]
pub struct SetFieldValue;

pub fn set_field_value(
    client: Client,
    project_id: &str,
    // must be the ProjectV2Item ID, not the Issue/PR item ID
    item_id: &str,
    field_id: &str,
    value: Option<String>,
) -> Result<String, Error> {
    use set_field_value::*;

    let variables = Variables {
        item_id: item_id.to_string(),
        field_id: field_id.to_string(),
        value,
        project_id: project_id.to_string(),
    };
    let response_body = post_graphql::<SetFieldValue, _>(&client, URL, variables)?;
    let response_data: ResponseData = response_body
        .data
        .expect("Expected ID for set GH field value.");
    Ok(response_data
        .update_project_v2_item_field_value
        .expect("GH set field value response is missing.")
        .project_v2_item
        .expect("GH set field value missing ID.")
        .id)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemas/github.graphql",
    query_path = "queries/github/set_field_option.graphql",
    response_derives = "Debug"
)]
pub struct SetFieldOption;

pub fn set_field_option(
    client: Client,
    project_id: &str,
    // must be the ProjectV2Item ID, not the Issue/PR item ID
    item_id: &str,
    field_id: &str,
    option_id: Option<String>,
) -> Result<String, Error> {
    use set_field_option::*;

    let variables = Variables {
        item_id: item_id.to_string(),
        field_id: field_id.to_string(),
        value: option_id,
        project_id: project_id.to_string(),
    };
    let response_body = post_graphql::<SetFieldOption, _>(&client, URL, variables)?;
    let response_data: ResponseData = response_body
        .data
        .expect("Expected ID for set GH field option.");
    Ok(response_data
        .update_project_v2_item_field_value
        .expect("GH set field option response is missing.")
        .project_v2_item
        .expect("GH set field option missing ID.")
        .id)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemas/github.graphql",
    query_path = "queries/github/set_field_number.graphql",
    response_derives = "Debug"
)]
pub struct SetFieldNumber;

pub fn set_field_number(
    client: Client,
    project_id: &str,
    // must be the ProjectV2Item ID, not the Issue/PR item ID
    item_id: &str,
    field_id: &str,
    value: Option<f64>,
) -> Result<String, Error> {
    use set_field_number::*;

    let variables = Variables {
        item_id: item_id.to_string(),
        field_id: field_id.to_string(),
        value,
        project_id: project_id.to_string(),
    };
    let response_body = post_graphql::<SetFieldNumber, _>(&client, URL, variables)?;
    let response_data: ResponseData = response_body
        .data
        .expect("Expected ID for set GH field number.");
    Ok(response_data
        .update_project_v2_item_field_value
        .expect("GH set field number response is missing.")
        .project_v2_item
        .expect("GH set field number missing ID.")
        .id)
}
