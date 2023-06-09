mod gql;
mod post;

use std::sync::Arc;
use std::{env, error::Error, time::Duration};

use cynic::http::ReqwestExt;
use reqwest::{
    header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION},
    Client,
};
use tokio::join;

pub use post::Post;

use gql::{create_graphql_request, discussion_exists};

pub struct HttpClients {
    pub html: Client,
    pub gql: Client,
    pub website_rss_url: String,

    pub github_rest_url: String,
    pub github_gql_url: String,
    pub discussion_category: String,
    pub repo_owner: String,
    pub repo_name: String,
}

impl HttpClients {
    pub fn init() -> Arc<Self> {
        let token = env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env var is required");
        let mut gh_headers = HeaderMap::new();
        gh_headers.insert(AUTHORIZATION, HeaderValue::from_str(&token).unwrap());
        gh_headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );

        Arc::new(Self {
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
            // https://team-role-org-testing.github.io/feed.xml, category Blogs
            website_rss_url: env::var("WEBSITE_RSS_URL").expect("WEBSITE_BASE_URL env var is required"),

            github_rest_url: env::var("GITHUB_API_URl").unwrap_or("https://api.github.com".to_string()),
            github_gql_url: env::var("GITHUB_GRAPHQL_URL").unwrap_or("https://api.github.com/graphql".to_string()),
            discussion_category: env::var("DISCUSSION_CATEGORY").expect("DISCUSSION_CATEGORY env var is required"),
            repo_owner: env::var("GITHUB_REPOSITORY_OWNER").expect("Repo owner was not found (GITHUB_REPOSITORY_OWNER)"),
            repo_name: env::var("GITHUB_REPOSITORY").unwrap().split_once('/').expect("Not a valid repo/name string").1.into()
        })
    }
}

pub async fn create_discussion(
    clients: Arc<HttpClients>,
    post: Arc<Post>,
) -> Result<(), Box<dyn Error>> {
    let (is_existing_discussion, create_disc_op) = join!(
        discussion_exists(Arc::clone(&clients), Arc::clone(&post)),
        create_graphql_request(Arc::clone(&clients), Arc::clone(&post))
    );

    if is_existing_discussion.as_ref().unwrap().is_some() {
        panic!(
            "Discussion was not created for {} - an existing discussion was found at {}",
            &post.url,
            is_existing_discussion?.unwrap()
        );
    }

    let create_disc_resp = clients
        .gql
        .post(&clients.github_gql_url)
        .run_graphql(create_disc_op)
        .await?;

    if let Some(discussion_info) = create_disc_resp
        .data
        .and_then(|d| d.create_discussion)
        .and_then(|payload| payload.discussion)
    {
        if discussion_info.title == post.url.path() {
            println!(
                "Successfully created new discussion at {} ({})",
                String::from(discussion_info.url),
                discussion_info.title
            )
        }
    } else {
        panic!(
            "Discussion could not be generated. GraphQL errors: \n{:#?}",
            create_disc_resp.errors
        );
    }
    Ok(())
}
