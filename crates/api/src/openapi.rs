use axum::Json;
use serde_json::json;
use utoipa::openapi::extensions::Extensions;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::routes::auth::{LoginRequest, OkResponse, ProjectContextResponse, TokenResponse};
use crate::routes::health::HealthResponse;
use crate::routes::projects::{
    CreateProjectRequest, CreateProjectResponse, ListProjectsResponse, ProjectKeys, ProjectSummary,
};

struct IndiebaseLogo;

impl Modify for IndiebaseLogo {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.info.extensions = Some(Extensions::from_iter([(
            "x-logo",
            json!({
                "url": "/logo.svg",
                "altText": "Indiebase",
                "href": "https://indiebase.deskbtm.com"
            }),
        )]));
    }
}

struct BearerSecurity;

impl Modify for BearerSecurity {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("Opaque Token")
                        .description(Some(
                            "Dashboard Session from `POST /api/auth/login`. \
                             Send as `Authorization: Bearer <token>`. Not a JWT.\n\n\
                             Project-scoped routes also require header \
                             `X-Indiebase-Project-Id: <project_ulid>` (membership checked per request).",
                        ))
                        .build(),
                ),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Indiebase Manager API",
        version = "0.1.0",
        description = "Indiebase is a self-hosted BaaS for indie teams.\n\n\
            **This document covers the Manager API** (Dashboard auth, projects, and platform \
            governance). Tenant CRUD goes through the Data API (`/api/data/*`) and is not fully \
            documented here yet.\n\n\
            **Auth model:** One **Dashboard Session** (Opaque Token + Redis) — no JWT, no second \
            Project login. Project context is supplied per request with \
            `X-Indiebase-Project-Id` (or `project_id` in the URL for nested Manager routes). \
            Membership is resolved via `project_members` on each request.",
        contact(name = "Indiebase", url = "https://indiebase.deskbtm.com")
    ),
    modifiers(&IndiebaseLogo, &BearerSecurity),
    paths(
        crate::routes::health::health,
        crate::routes::auth::login,
        crate::routes::auth::logout,
        crate::routes::auth::project_context,
        crate::routes::projects::create,
        crate::routes::projects::list,
    ),
    components(schemas(
        HealthResponse,
        LoginRequest,
        TokenResponse,
        OkResponse,
        ProjectContextResponse,
        CreateProjectRequest,
        CreateProjectResponse,
        ProjectKeys,
        ProjectSummary,
        ListProjectsResponse,
    )),
    tags(
        (name = "system", description = "Liveness probes and documentation endpoints"),
        (name = "auth", description = "Dashboard Session (Opaque Token + Redis); project context via header"),
        (name = "projects", description = "Project lifecycle on the Manager API (create, list, membership)")
    )
)]
pub struct ApiDoc;

pub async fn serve_openapi() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}
