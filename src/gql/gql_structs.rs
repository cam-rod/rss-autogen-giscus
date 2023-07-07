#[cynic::schema("github")]
mod schema {}

// query CategoryQuery

#[derive(cynic::QueryVariables, Debug)]
pub struct CategoryQueryVariables<'a> {
    pub owner: &'a str,
    pub repo_name: &'a str,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "CategoryQueryVariables")]
pub struct CategoryQuery {
    #[arguments(owner: $owner, name: $repo_name)]
    pub repository: Option<Repository>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Repository {
    #[arguments(first: 20)]
    pub discussion_categories: DiscussionCategoryConnection,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct DiscussionCategoryConnection {
    pub edges: Option<Vec<Option<DiscussionCategoryEdge>>>,
    pub page_info: PageInfo,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct PageInfo {
    pub end_cursor: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct DiscussionCategoryEdge {
    pub node: Option<DiscussionCategory>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct DiscussionCategory {
    pub name: String,
    pub id: cynic::Id,
}

// Mutation CreateCommentsDiscussion

#[derive(cynic::QueryVariables, Debug)]
pub struct CreateCommentsDiscussionVariables {
    pub cat_id: cynic::Id,
    pub desc: String,
    pub rel_path: String,
    pub repo_id: cynic::Id,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    graphql_type = "Mutation",
    variables = "CreateCommentsDiscussionVariables"
)]
pub struct CreateCommentsDiscussion {
    #[arguments(input: { body: $desc, categoryId: $cat_id, repositoryId: $repo_id, title: $rel_path })]
    pub create_discussion: Option<CreateDiscussionPayload>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct CreateDiscussionPayload {
    pub discussion: Option<Discussion>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Discussion {
    pub id: cynic::Id,
    pub title: String,
    pub created_at: DateTime,
    pub url: Uri,
}

#[derive(cynic::Scalar, Debug, Clone)]
pub struct DateTime(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
#[cynic(graphql_type = "URI")]
pub struct Uri(pub String);

impl From<Uri> for String {
    fn from(value: Uri) -> Self {
        value.0
    }
}

// query DiscussionExists

#[derive(cynic::QueryVariables, Debug)]
pub struct DiscussionExistsVariables<'a> {
    pub owner: &'a str,
    pub repo_name: &'a str,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "DiscussionExistsVariables")]
pub struct DiscussionExists {
    #[arguments(owner: $owner, name: $repo_name)]
    pub repository: Option<Repository>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Repository {
    #[arguments(orderBy: { direction: "DESC", field: "CREATED_AT" }, first: 50)]
    pub discussions: DiscussionConnection,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct DiscussionConnection {
    pub edges: Option<Vec<Option<DiscussionEdge>>>,
    pub page_info: PageInfo,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct PageInfo {
    pub end_cursor: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct DiscussionEdge {
    pub node: Option<Discussion>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Discussion {
    pub id: cynic::Id,
    pub title: String,
    pub created_at: DateTime,
}

#[derive(cynic::Scalar, Debug, Clone)]
pub struct DateTime(pub String);

