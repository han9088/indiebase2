use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::auth::extractors::DashboardAuth;
use crate::error::ApiError;
use crate::projects::service::{create_project, list_projects_for_user};
use crate::state::AppState;

/// Create a new project owned by the current Dashboard user.
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({"name":"my-app"}))]
pub struct CreateProjectRequest {
    /// Human-readable project name (trimmed; must be non-empty).
    #[schema(example = "my-app", min_length = 1)]
    pub name: String,
}

/// One-time plaintext API key pair returned only at project creation.
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "publishable": "ib_pub_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
    "secret": "ib_sec_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
}))]
pub struct ProjectKeys {
    /// Publishable key (`ib_pub_…`). Safe to embed in clients; binds to this project.
    #[schema(example = "ib_pub_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")]
    pub publishable: String,
    /// Secret key (`ib_sec_…`). Server-only; shown **once** here — store immediately.
    #[schema(example = "ib_sec_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")]
    pub secret: String,
}

/// Result of creating a project, including one-time key material.
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "id": "01jcqz4sxf7k2m8n3p5r6t9vwx",
    "name": "my-app",
    "keys": {
        "publishable": "ib_pub_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
        "secret": "ib_sec_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
    }
}))]
pub struct CreateProjectResponse {
    /// Project ULID; tenant schema is `proj_{id}`.
    #[schema(
        example = "01jcqz4sxf7k2m8n3p5r6t9vwx",
        min_length = 26,
        max_length = 26
    )]
    pub id: String,
    /// Echo of the created project name.
    #[schema(example = "my-app")]
    pub name: String,
    /// Publishable + Secret keys (plaintext only in this response).
    pub keys: ProjectKeys,
}

/// Project row as seen by the current member.
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "id": "01jcqz4sxf7k2m8n3p5r6t9vwx",
    "name": "my-app",
    "role": "owner"
}))]
pub struct ProjectSummary {
    /// Project ULID.
    #[schema(example = "01jcqz4sxf7k2m8n3p5r6t9vwx")]
    pub id: String,
    /// Project display name.
    #[schema(example = "my-app")]
    pub name: String,
    /// Caller's role on this project: `owner` | `admin` | `member`.
    #[schema(example = "owner")]
    pub role: String,
}

/// Projects the current Dashboard user belongs to.
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "projects": [{
        "id": "01jcqz4sxf7k2m8n3p5r6t9vwx",
        "name": "my-app",
        "role": "owner"
    }]
}))]
pub struct ListProjectsResponse {
    /// Membership-filtered project list (excludes soft-deleted projects/memberships).
    pub projects: Vec<ProjectSummary>,
}

#[utoipa::path(
    post,
    path = "/api/projects",
    operation_id = "projects_create",
    summary = "Create project",
    description = "Creates a project, adds the caller as `owner`, provisions PostgreSQL schema \
        `proj_{ulid}`, mints default Publishable + Secret API keys, and registers the schema with \
        PostgREST.\n\n\
        **Important:** plaintext keys appear only in this response — persist them client-side; \
        later list endpoints never return full secrets.",
    request_body = CreateProjectRequest,
    responses(
        (status = 200, description = "Project created; keys returned once", body = CreateProjectResponse),
        (status = 400, description = "Empty or invalid `name`"),
        (status = 401, description = "Missing or invalid Dashboard Session")
    ),
    security(("bearer_auth" = [])),
    tag = "projects"
)]
pub async fn create(
    State(state): State<AppState>,
    auth: DashboardAuth,
    Json(body): Json<CreateProjectRequest>,
) -> Result<Json<CreateProjectResponse>, ApiError> {
    let created = create_project(&state.pool, &auth.session.user_id, &body.name).await?;

    Ok(Json(CreateProjectResponse {
        id: created.id,
        name: created.name,
        keys: ProjectKeys {
            publishable: created.publishable_key,
            secret: created.secret_key,
        },
    }))
}

#[utoipa::path(
    get,
    path = "/api/projects",
    operation_id = "projects_list",
    summary = "List my projects",
    description = "Returns projects where the current Dashboard user is an active member \
        (`project_members.deleted_at` and `projects.deleted_at` are null). Each item includes the \
        caller's `role` on that project.",
    responses(
        (status = 200, description = "Membership-filtered project list", body = ListProjectsResponse),
        (status = 401, description = "Missing or invalid Dashboard Session")
    ),
    security(("bearer_auth" = [])),
    tag = "projects"
)]
pub async fn list(
    State(state): State<AppState>,
    auth: DashboardAuth,
) -> Result<Json<ListProjectsResponse>, ApiError> {
    let items = list_projects_for_user(&state.pool, &auth.session.user_id).await?;
    Ok(Json(ListProjectsResponse {
        projects: items
            .into_iter()
            .map(|p| ProjectSummary {
                id: p.id,
                name: p.name,
                role: p.role,
            })
            .collect(),
    }))
}
