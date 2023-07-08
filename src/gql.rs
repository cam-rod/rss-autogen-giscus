mod gql_structs;

use std::error::Error;

use cynic::{http::ReqwestExt, MutationBuilder, Operation, QueryBuilder};
use serde_json::Value;
use url::Url;

use crate::{HttpClients, Post};
use gql_structs::{
    CategoryQuery, CategoryQueryVariables, CreateCommentsDiscussion,
    CreateCommentsDiscussionVariables,
};

pub async fn discussion_exists(clients: &HttpClients, post: &Post) -> bool {
    todo!()
}

// TODO: actually make these commands go through each page
pub async fn create_graphql_request(
    clients: &HttpClients,
    post: &Post,
) -> Result<Operation<CreateCommentsDiscussion, CreateCommentsDiscussionVariables>, Box<dyn Error>>
{
    let repo_id = get_repo_id(clients).await?;
    let cat_id = get_category_id(clients).await?;

    let full_desc = String::from_iter(vec![
        String::from(&post.description),
        "\n\n".to_string(),
        post.url.to_string(),
    ]);

    Ok(CreateCommentsDiscussion::build(
        CreateCommentsDiscussionVariables {
            repo_id,
            cat_id,
            desc: full_desc,
            title: post.url.path().to_string(),
        },
    ))
}

async fn get_repo_id(clients: &HttpClients) -> Result<cynic::Id, Box<dyn Error>> {
    let repo_resp: Value = clients
        .gql
        .get(format!(
            "{}/repos/{}/{}",
            clients.github_rest_url, clients.repo_owner, clients.repo_name
        ))
        .send()
        .await?
        .json()
        .await?;
    Ok(repo_resp["id"].as_str().unwrap().into())
}

async fn get_category_id(clients: &HttpClients) -> Result<cynic::Id, Box<dyn Error>> {
    let category_query = CategoryQuery::build(CategoryQueryVariables {
        owner: &clients.repo_owner,
        repo_name: &clients.repo_name,
    });

    let category_resp = clients
        .gql
        .post(&clients.github_gql_url)
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
            if cat_edge.node.as_ref().unwrap().name == clients.discussion_category {
                return Ok(cat_edge.node.unwrap().id);
            }
        }
        panic!(
            "Category {} was not present in repository {}/{}",
            clients.discussion_category, clients.repo_owner, clients.repo_name
        )
    } else {
        panic!("No discussion categories found!");
    }
}
