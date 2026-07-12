use axum::Json;
use serde_json::json;
use utoipa::openapi::extensions::Extensions;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::routes::auth::{LoginRequest, ProjectLoginRequest, TokenResponse};
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
                        .bearer_format("opaque")
                        .build(),
                ),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Indiebase API",
        version = "0.1.0",
        description = "Self-hosted BaaS — Manager API and Data API (MVP in progress)."
    ),
    modifiers(&IndiebaseLogo, &BearerSecurity),
    paths(
        crate::routes::health::health,
        crate::routes::auth::login,
        crate::routes::auth::logout,
        crate::routes::auth::project_login,
        crate::routes::auth::project_logout,
        crate::routes::projects::create,
        crate::routes::projects::list,
    ),
    components(schemas(
        HealthResponse,
        LoginRequest,
        ProjectLoginRequest,
        TokenResponse,
        CreateProjectRequest,
        CreateProjectResponse,
        ProjectKeys,
        ProjectSummary,
        ListProjectsResponse,
    )),
    tags(
        (name = "system", description = "Liveness and docs"),
        (name = "auth", description = "Dashboard and Project sessions"),
        (name = "projects", description = "Project lifecycle (Manager API)")
    )
)]
pub struct ApiDoc;

pub async fn serve_openapi() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}
