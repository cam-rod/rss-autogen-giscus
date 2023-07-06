use std::error::Error;

use cynic::{http::ReqwestExt, MutationBuilder, Operation, QueryBuilder};
use reqwest::Client;
use serde_json::Value;
use url::Url;

use crate::constants::{CATEGORY_NAME, GITHUB_GQL_API, REPO_NAME, REPO_OWNER};
use crate::gql_structs::{
    CategoryQuery, CategoryQueryVariables, CreateCommentsDiscussion,
    CreateCommentsDiscussionVariables,
};

pub async fn create_graphql_request(
    gql_client: &Client,
    url: &Url,
    desc: String,
) -> Result<Operation<CreateCommentsDiscussion, CreateCommentsDiscussionVariables>, Box<dyn Error>>
{
    let repo_id = get_repo_id(gql_client).await?;
    let cat_id = get_category_id(gql_client).await?;

    Ok(CreateCommentsDiscussion::build(
        CreateCommentsDiscussionVariables {
            repo_id,
            cat_id,
            desc,
            rel_path: url.path().to_string(),
        },
    ))
}

async fn get_repo_id(gql_client: &Client) -> Result<cynic::Id, Box<dyn Error>> {
    let repo_resp: Value = gql_client
        .get(format!(
            "https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}"
        ))
        .send()
        .await?
        .json()
        .await?;
    Ok(repo_resp["id"].as_str().unwrap().into())
}

async fn get_category_id(gql_client: &Client) -> Result<cynic::Id, Box<dyn Error>> {
    let category_query = CategoryQuery::build(CategoryQueryVariables {
        owner: REPO_OWNER,
        repo_name: REPO_NAME,
    });

    let category_resp = gql_client
        .post(GITHUB_GQL_API)
        .run_graphql(category_query)
        .await?;

    if category_resp.errors.is_none() {
        for cat_edge in category_resp
            .data
            .unwrap()
            .repository
            .unwrap()
            .discussion_categories
            .edges
            .unwrap()
            .into_iter()
            .flatten()
        {
            if cat_edge.node.as_ref().unwrap().name == CATEGORY_NAME {
                return Ok(cat_edge.node.unwrap().id);
            }
        }
        panic!("Category {CATEGORY_NAME} was not present in repository {REPO_OWNER}/{REPO_NAME}")
    } else {
        panic!("No discussion categories found!");
    }
}

pub async fn discussion_exists() -> bool {
    todo!()
}
