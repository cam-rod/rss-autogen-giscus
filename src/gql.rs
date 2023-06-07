use graphql_client::GraphQLQuery;
use reqwest::header::HeaderMap;
use reqwest::Client;
use serde_json::Value;
use std::error::Error;
use url::Url;

use crate::constants::{CATEGORY_NAME, REPO_NAME, REPO_OWNER};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/github.schema.graphql",
    query_path = "src/category_query.graphql"
)]
struct CategoryQuery;

pub async fn discussion_exists() -> bool {
    todo!()
}

pub async fn create_graphql_request(
    client: &Client,
    gh_headers: &HeaderMap,
    url: &Url,
    description: &String,
) -> Result<String, Box<dyn Error>> {
    let repo_resp: Value = client
        .get(format!(
            "https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}"
        ))
        .headers(gh_headers.clone())
        .send()
        .await?
        .json()
        .await?;
    let repo_id = repo_resp["id"].as_str().unwrap();

    let category_json = CategoryQuery::build_query(category_query::Variables {
        owner: REPO_OWNER.to_string(),
        repo_name: REPO_NAME.to_string(),
    });

    let categories_resp: Value = client
        .post("https://api.github.com/graphql")
        .headers(gh_headers.clone())
        .json(&category_json)
        .send()
        .await?
        .json()
        .await?;

    for category in categories_resp["data"]["repository"]["edges"]
        .as_array()
        .unwrap()
    {
        if let Some(id) = category["node"]["name"].as_str() {
            if id == CATEGORY_NAME {
                return Ok(create_discussion_gql(
                    &repo_id.to_string(),
                    id,
                    url,
                    description,
                ));
            }
        } else {
            panic!("Category {CATEGORY_NAME} was not present in repository {REPO_OWNER}/{REPO_NAME}");
        }
    }
    panic!("No discussion categories exist!")
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
