use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::auth::extractors::DashboardAuth;
use crate::error::ApiError;
use crate::projects::service::{create_project, list_projects_for_user};
use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateProjectRequest {
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProjectKeys {
    pub publishable: String,
    pub secret: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreateProjectResponse {
    pub id: String,
    pub name: String,
    pub keys: ProjectKeys,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    pub role: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListProjectsResponse {
    pub projects: Vec<ProjectSummary>,
}

#[utoipa::path(
    post,
    path = "/api/projects",
    request_body = CreateProjectRequest,
    responses(
        (status = 200, description = "Project created with one-time plaintext API keys", body = CreateProjectResponse),
        (status = 401, description = "Missing dashboard session")
    ),
    security(("bearer_auth" = [])),
    tag = "projects"
)]
pub async fn create(
    State(state): State<AppState>,
    auth: DashboardAuth,
    Json(body): Json<CreateProjectRequest>,
) -> Result<Json<CreateProjectResponse>, ApiError> {
    let created = create_project(
        &state.pool,
        state.config.as_ref(),
        &auth.session.user_id,
        &body.name,
    )
    .await?;

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
    responses(
        (status = 200, description = "Projects for the current dashboard user", body = ListProjectsResponse),
        (status = 401, description = "Missing dashboard session")
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
