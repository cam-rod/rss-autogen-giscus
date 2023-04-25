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

use std::time::Duration;

use feed_rs::parser::parse;
use octocrab::Octocrab;
use reqwest::Client;

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

    let post_url = latest_post(&client).await;

    // TODO:
    //   - Strip to the part of the URL that becomes the
    //   - get the first paragraph? or not
    //   - add link to body of request
}

async fn latest_post(rss_client: &Client) -> String {
    let feed = rss_client
        .get("https://team-role-org-testing.github.io/feed.xml") // https://www.wildfly.org/feed.xml
        .send().await.unwrap()
        .bytes().await.unwrap();

    let posts = parse(&feed[..]).expect("Unable to parse team-role-org-testing feed");

    posts
        .entries.first().expect("No posts found in feed")
        .links.first().expect("No link provided with first post")
        .href.clone()
}
