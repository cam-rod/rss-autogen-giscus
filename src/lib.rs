use cynic::http::ReqwestExt;
use std::env;
use std::error::Error;
use std::time::Duration;

use crate::gql::create_graphql_request;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION};
use reqwest::Client;
use url::Url;

mod gql;
mod post;

pub struct HttpClients {
    pub html: Client,
    pub gql: Client,
    pub website_rss_url: Url,

    pub github_rest_url: Url,
    pub github_gql_url: Url,
    pub discussion_category: String,
    pub repo_owner: String,
    pub repo_name: String,
}

pub fn create_clients() -> HttpClients {
    let token = env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env var is required");
    let mut gh_headers = HeaderMap::new();
    gh_headers.insert(AUTHORIZATION, HeaderValue::from_str(&token).unwrap());
    gh_headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/vnd.github+json"),
    );

    HttpClients {
        html: Client::builder()
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) rss-autogen-giscus/0.1.0 Chrome/113.0.0.0 Safari/537.36")
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Unable to build REST client"),
        gql: Client::builder()
            .timeout(Duration::from_secs(60))
            .default_headers(gh_headers)
            .build()
            .expect("Unable to build GraphQL client"),
        // https://team-role-org-testing.github.io/feed.xml, repo team-role-org-testing/team-role-org-testing.github.io, category Blogs
        website_rss_url: env::var("WEBSITE_RSS_URL").expect("WEBSITE_BASE_URL env var is required").parse().expect("Invalid WEBSITE_BASE_URL provided"),

        github_rest_url: env::var("GITHUB_API_URl").unwrap_or("https://api.github.com".to_string()).parse().unwrap(),
        github_gql_url: env::var("GITHUB_GRAPHQL_URL").unwrap_or("https://api.github.com/graphql".to_string()).parse().unwrap(),
        discussion_category: env::var("DISCUSSION_CATEGORY").expect("DISCUSSION_CATEGORY env var is required"),
        repo_owner: env::var("GITHUB_REPOSITORY_OWNER").expect("Repo owner was not found (GITHUB_REPOSITORY_OWNER)"),
        repo_name: env::var("GITHUB_REPOSITORY").unwrap().split_once('/').expect("Not a valid repo/name string").1.into()
    }
}

pub async fn create_discussion(
    clients: &HttpClients,
    post_url: Url,
    post_desc: String,
) -> Result<(), Box<dyn Error>> {
    let create_disc_vars = create_graphql_request(&clients, &post_url, post_desc)
        .await
        .unwrap();
    let create_disc_resp = clients
        .gql
        .post(&clients.github_gql_url)
        .run_graphql(create_disc_vars)
        .await?;

    if let Some(create_disc_data) = create_disc_resp.data {
        if let Some(discussion_info) = create_disc_data.create_discussion.unwrap().discussion {
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
            "Dicussion could not be generated. GraphQL response: \n{:#?}",
            create_disc_resp.errors
        );
    }
    Ok(())
}