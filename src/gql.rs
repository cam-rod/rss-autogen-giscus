use graphql_client::{GraphQLQuery, QueryBody, Response};
use reqwest::Client;
use serde_json::Value;
use std::error::Error;
use graphql_client::reqwest::post_graphql;
use url::Url;

use crate::constants::{CATEGORY_NAME, GITHUB_GQL_API, REPO_NAME, REPO_OWNER};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/github.schema.graphql",
    query_path = "graphql/category_query.graphql"
)]
struct CategoryQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/github.schema.graphql",
    query_path = "graphql/create_comments_discussion.graphql"
)]
pub struct CreateCommentsDiscussion;

pub struct GQLResponseError {
    errors: Vec<graphql_client::Error>
}

pub async fn create_graphql_request(
    gql_client: &Client,
    url: &Url,
    desc: String,
) -> Result<QueryBody<create_comments_discussion::Variables>, Box<dyn Error>> {

    let repo_id  = get_repo_id(gql_client).await?;
    let cat_id = get_category_id(gql_client).await?;

    Ok(CreateCommentsDiscussion::build_query(create_comments_discussion::Variables {
        repo_id,
        cat_id,
        desc,
        rel_path: url.to_string()
    }))
}

async fn get_repo_id(gql_client: &Client) -> Result<String, Box<dyn Error>> {
    let repo_resp: Value = gql_client
        .get(format!(
            "https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}"
        ))
        .send()
        .await?
        .json()
        .await?;
    Ok(repo_resp["id"].as_str().unwrap().to_string())
}

async fn get_category_id(gql_client: &Client) -> Result<String, Box<dyn Error>> {
    let category_json = CategoryQuery::build_query(category_query::Variables {
        owner: REPO_OWNER.to_string(),
        repo_name: REPO_NAME.to_string(),
    });

    // let categories_resp: Value = gql_client
    //     .post("https://api.github.com/graphql")
    //     .json(&category_json)
    //     .send()
    //     .await?
    //     .json()
    //     .await?;

    let category_resp: Response<category_query::ResponseData> = post_graphql(gql_client, GITHUB_GQL_API, category_json).await?;

    category_resp.errors.map_or(|| {
        for cat_edge in category_resp.data.unwrap().repository.unwrap().discussion_categories.edges.unwrap() {
            if let Some(cat_node) = cat_edge {
                if cat_node.node.unwrap().name == CATEGORY_NAME {
                    return Ok(cat_node.node.unwrap().id.to_string())
                }
            }
        }
        Err(GQLResponseError { errors: vec![graphql_client::Error {
            message: format!("Category {CATEGORY_NAME} was not present in repository {REPO_OWNER}/{REPO_NAME}"),
            path: None,
            extensions: None,
            locations: None,
        }]})
    }, |errors|  Err(GQLResponseError { errors }) );

    // graphql-client has some major weaknesses, esp. since it doesn't recognize the DateTime or ID types...
    // I'll switch to Cynic next

    todo!()
}

pub async fn discussion_exists() -> bool {
    todo!()
}

fn create_discussion_gql(repo: &str, category: &str, url: &Url, description: &String) -> String {
    format!("\
{{
  'mutation': {{
    createDiscussion(input: {{repositoryId: {repo}, categoryId: {category}, title: \"{}\", body: \"{description}\") {{
      discussion {{
        id
        title
        url
      }}
    }}
  }}
}}", url.path())
}
