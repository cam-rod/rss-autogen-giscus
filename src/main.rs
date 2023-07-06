//! Autogenerates GitHub Discussions to be used by Giscus.
//!
//! This came out of a preference for a way to use Giscus, without requiring users to authenticate
//! with the app. Since the discussion isn't created until someone comments, we needed a way to
//! automatically create it once a blog post was uploaded.
//!
//! This crate checks for the latest post via the RSS feed, and then extracts the contents needed to
//! to create a post, formatted as follows:
//!
//! - **Title**: URL path of the post (not including base URL)
//! - **Description**: (potentially) First paragraph of the post, followed by a full link
//!
//! This crate works best when run as a GitHub Action, triggered by the completion of the
//! `pages-build-deployment` action for GitHub pages. It depends on the RSS feed being up-to-date at the time
//! of running, so you may need to introduce a delay.

use std::error::Error;
use std::time::Duration;

use cynic::http::ReqwestExt;
use reqwest::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue};

use gql::create_graphql_request;
use post::{latest_post, post_description};

use crate::constants::GITHUB_GQL_API;

mod constants; // TODO replace by moving this to lib.rs, taking these as params
mod gql;
mod gql_structs;
mod post;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env var is required");
    let mut gh_headers = HeaderMap::new();
    gh_headers.insert(AUTHORIZATION, HeaderValue::from_str(&token).unwrap());
    gh_headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/vnd.github+json"),
    );

    let html_client = Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) rss-autogen-giscus/0.1.0 Chrome/113.0.0.0 Safari/537.36")
        .timeout(Duration::from_secs(60))
        .build()
        .expect("Unable to build REST client");
    let gql_client = Client::builder()
        .timeout(Duration::from_secs(60))
        .default_headers(gh_headers)
        .build()
        .expect("Unable to build GraphQL client");

    let post_url = latest_post(&html_client).await.unwrap();
    let post_desc = post_description(&html_client, post_url.as_str())
        .await
        .unwrap();
    let create_op_vars = create_graphql_request(&gql_client, &post_url, post_desc)
        .await
        .unwrap();
    let create_op_resp = gql_client
        .post(GITHUB_GQL_API)
        .run_graphql(create_op_vars)
        .await?;

    if let Some(create_op_data) = create_op_resp.data {
        if let Some(discussion_info) = create_op_data.create_discussion.unwrap().discussion {
            if discussion_info.title == post_url.path() {
                println!(
                    "Successfully created new discussion at {} ({})",
                    String::from(discussion_info.url),
                    discussion_info.title
                )
            }
        }
    } else {
        panic!(
            "Dicussion could not be generated. GraphQL response: {:#?}",
            create_op_resp.errors
        );
    }
    Ok(())
}
