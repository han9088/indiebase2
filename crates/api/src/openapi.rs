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
            **Manager API** covers Dashboard auth and projects. **Data API** (`/api/data/*`) is a \
            PostgREST gateway: SDK path `/api/data/{project_id}/*` uses API keys; Dashboard path \
            `/api/data/*` uses Dashboard Session + `X-Indiebase-Project-Id`.\n\n\
            **Auth model:** Opaque Token + Redis (no JWT for clients). Project context via header \
            or URL ULID. Membership is resolved via `project_members` on each request.",
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
        crate::routes::data::data_api_sdk_get_docs,
        crate::routes::data::data_api_dashboard_get_docs,
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
        (name = "projects", description = "Project lifecycle on the Manager API (create, list, membership)"),
        (name = "data-api", description = "PostgREST Data API gateway (Dashboard + SDK dual path)")
    )
)]
pub struct ApiDoc;

pub async fn serve_openapi() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}
