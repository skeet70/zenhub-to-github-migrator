use graphql_client::GraphQLQuery;

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
