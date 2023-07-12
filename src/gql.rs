use std::error::Error;

use chrono::{Duration, Utc};
use cynic::{http::ReqwestExt, Operation};
use serde_json::Value;

use crate::{HttpClients, Post};
use gh_gql_schema::{
    CategoryQuery, CategoryQueryVariables, CreateCommentsDiscussion,
    CreateCommentsDiscussionVariables, DiscussionExists, DiscussionExistsVariables,
};

// TODO: actually make these commands go through each page
pub async fn create_graphql_request(
    clients: &HttpClients,
    post: &Post,
) -> Result<Operation<CreateCommentsDiscussion, CreateCommentsDiscussionVariables>, Box<dyn Error>>
{
    use cynic::MutationBuilder;

    let repo_id = get_repo_id(clients).await?;
    let cat_id = get_category_id(clients).await?;

    let mut full_desc = post.url.to_string();
    if let Some(mut post_desc) = post.description.clone() {
        post_desc.push_str("\n\n");
        full_desc.insert_str(0, post_desc.as_str());
    }

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
    use cynic::QueryBuilder;

    let category_query = CategoryQuery::build(CategoryQueryVariables {
        owner: &clients.repo_owner,
        repo_name: &clients.repo_name,
    });

    let category_resp = clients
        .gql
        .post(&clients.github_gql_url)
        .run_graphql(category_query)
        .await?;

    if let Some(categories) = category_resp
        .data
        .and_then(|d| d.repository)
        .map(|repo| repo.discussion_categories.edges)
    {
        match categories
            .iter()
            .flat_map(|c| &c.node)
            .find(|cat| cat.name == clients.discussion_category)
        {
            Some(matching_cat) => Ok(matching_cat.name.clone().into()),
            None => {
                panic!(
                    "Category {} was not present in repository {}/{}",
                    clients.discussion_category, clients.repo_owner, clients.repo_name
                );
            }
        }
    } else {
        panic!(
            "No discussion categories found! GraphQL errors:\n{:#?}",
            category_resp.errors.unwrap()
        );
    }
}

pub async fn discussion_exists(
    clients: &HttpClients,
    post: &Post,
) -> Result<Option<String>, Box<dyn Error>> {
    use cynic::QueryBuilder;

    let current_time = Utc::now();
    let max_lookback = Duration::days(7);

    let discussion_exists_query = DiscussionExists::build(DiscussionExistsVariables {
        owner: &clients.repo_owner,
        repo_name: &clients.repo_name,
    });

    let discussion_exists_resp = clients
        .gql
        .post(&clients.github_gql_url)
        .run_graphql(discussion_exists_query)
        .await?;

    if discussion_exists_resp.errors.is_none() {
        for discussion in discussion_exists_resp
            .data
            .and_then(|data| data.repository)
            .map(|repo| repo.discussions.edges)
            .unwrap()
            .iter()
            .filter_map(|edge| edge.node.as_ref())
        {
            // Don't check for discussions older than 7 days
            if discussion
                .created_at
                .0
                .parse::<chrono::DateTime<Utc>>()
                .unwrap()
                - current_time
                > max_lookback
            {
                return Ok(None);
            } else if Some(&discussion.title) == post.title.as_ref() {
                return Ok(Some(discussion.url.0.clone()));
            }
        }
    }

    panic!(
        "Unable to query existing repos. GraphQL errors: \n{:#?}",
        discussion_exists_resp.errors
    );
}
