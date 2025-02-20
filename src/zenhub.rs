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
