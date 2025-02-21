use graphql_client::{reqwest::post_graphql_blocking as post_graphql, GraphQLQuery};
use reqwest::blocking::Client;

pub const URL: &str = "https://api.zenhub.com/public/graphql";

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemas/zenhub.graphql",
    query_path = "queries/zenhub/get_workspace.graphql",
    response_derives = "Debug, Clone"
)]
pub struct GetWorkspace;

pub fn get_workspace(
    client: Client,
    name: &str,
) -> Result<get_workspace::GetWorkspaceRecentlyViewedWorkspacesNodes, anyhow::Error> {
    use get_workspace::*;
    let response_body = post_graphql::<GetWorkspace, _>(&client, URL, Variables {})?;
    let response_data: ResponseData = response_body
        .data
        .expect("Failed to get Zenhub workspace data.");
    let workspaces = response_data.recently_viewed_workspaces.nodes;
    let desired_workspace = workspaces
        .iter()
        .find(|w| w.name == Some(name.to_string()))
        .expect(&format!(
            "No name matching {name} found in response: {:?}",
            workspaces
                .iter()
                .map(|w| w.name.clone())
                .collect::<Vec<Option<String>>>()
        ));

    Ok(desired_workspace.clone())
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemas/zenhub.graphql",
    query_path = "queries/zenhub/get_pipeline_issues.graphql",
    response_derives = "Debug, Clone"
)]
pub struct GetPipelineIssues;

pub fn get_pipeline_issues(
    client: Client,
    pipeline_id: &str,
    workspace_id: &str,
) -> Result<Vec<get_pipeline_issues::GetPipelineIssuesSearchIssuesByPipelineNodes>, anyhow::Error> {
    use get_pipeline_issues::*;

    let mut pipeline_issues = vec![];
    let mut has_next_page = true;
    let mut end_cursor = None;
    while has_next_page {
        let variables = Variables {
            pipeline_id: pipeline_id.to_string(),
            workspace_id: workspace_id.to_string(),
            end_cursor: end_cursor.clone(),
        };
        let response_body = post_graphql::<GetPipelineIssues, _>(&client, URL, variables)?;
        if response_body.errors.is_some() {
            println!(
                "Error while getting ZH Pipeline issues {:?}",
                response_body.errors.as_ref().unwrap()
            );
        }
        let response_data = response_body
            .data
            .expect("Failed to get Zenhub pipeline issue data.")
            .search_issues_by_pipeline
            .expect("No issue data recieved for pipeline.");
        has_next_page = response_data.page_info.has_next_page;
        end_cursor = response_data.page_info.end_cursor;
        pipeline_issues.append(&mut response_data.nodes.clone());
    }

    Ok(pipeline_issues)
}
