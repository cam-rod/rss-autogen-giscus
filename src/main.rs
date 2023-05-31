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

use std::ops::Deref;
use std::time::Duration;

use feed_rs::parser::parse;
use octocrab::Octocrab;

use reqwest::{Client};
use scraper::{Html, Selector};
use url::Url;

const BASE_URL: &str = "https://team-role-org-testing.github.io";
const FEED_PATH: &str = "/feed.xml";
const REPO_OWNER: &str = "team-role-org-testing";
const REPO_NAME: &str = "team-role-org-testing.github.io";
const CATEGORY_NAME: &str = "Blogs";

#[tokio::main]
pub async fn main() {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env var is required");
    let octocrab = Octocrab::builder()
        .personal_token(token)
        .build()
        .expect("Unable to build GitHub API client");

    let client = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(60))
        .build()
        .expect("Unable to build HTTP client");

    let post_url = latest_post(&client).await.unwrap();
    let post_desc = post_description(&client, post_url.as_str()).await.unwrap();
    let request = create_graphql_request(&octocrab, &post_url, &post_desc);
    todo!()

    // TODO:
    //   - Strip to the part of the URL that becomes the
    //   - get the first paragraph? or not
    //   - add link to body of request
}

async fn latest_post(rss_client: &Client) -> reqwest::Result<Url> {
    let rss_response = rss_client
        .get(format!("{BASE_URL}{FEED_PATH}")) // https://www.wildfly.org/feed.xml
        .send()
        .await?
        .bytes()
        .await?;
    let parsed_feed =
        parse(rss_response.deref()).expect("Unable to parse team-role-org-testing feed");
    let post = parsed_feed.entries.first().expect("No posts found in feed");

    Ok(Url::parse(
        post.links
            .first()
            .expect("No link provided with first post")
            .href
            .as_str(),
    )
    .unwrap())
}

async fn post_description(client: &Client, post_url: &str) -> reqwest::Result<String> {
    let desc_selector = Selector::parse("meta[name=\"description\"]").unwrap();
    let post = Html::parse_document(&client.get(post_url).send().await?.text().await?);

    let desc_element = post
        .select(&desc_selector)
        .next()
        .expect("Could not find 'meta' element with name 'description'");

    Ok(desc_element
        .value()
        .attr("content")
        .expect("Invalid formatting for 'name' meta tag")
        .to_string())
}

async fn create_graphql_request(octocrab: &Octocrab, url: &Url, description: &String) -> octocrab::Result<String> {
    let repo_id = octocrab.repos(REPO_OWNER, REPO_NAME)
        .get().await?.id;
    let category_id = octocrab.graphql(&format!("\
query {{
  repository(owner:\"{REPO_OWNER}\", name:\"{REPO_NAME}\") {{
    discussionCategories(
}}"))

    let gql = format!("\
mutation {{
    createDiscussion(input: {{repositoryId: {repo_id},
}}");
    todo!()
}