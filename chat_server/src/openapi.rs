use crate::handlers::*;
use crate::AppState;
use crate::{CreateChat, CreateMessage, CreateUser, ErrorOutPut, ListMessages, SigninUser};
use axum::Router;
use chat_core::{Chat, ChatType, ChatUser, Message, User, Workspace};
use utoipa::OpenApi;
use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify,
};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

pub(crate) trait OpenApiRouter {
    fn openapi(self) -> Self;
}

#[derive(OpenApi)]
#[openapi(
    paths(signup_handler, signin_handler, list_chat_handler, create_chat_handler, get_chat_handler, list_message_handler, send_message_handler, file_handler, upload_handler, list_chat_users_handler),
    modifiers(&SecurityAddon),
    components(
        schemas(User, Chat, ChatType, ChatUser, Message, Workspace, SigninUser, CreateUser, CreateChat, CreateMessage, ListMessages, AuthOutPut, ErrorOutPut),
    ),
    tags(
        (name = "chat", description = "Chat related operations"),
    )
)]
pub(crate) struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "token",
                SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Bearer).build()),
            )
        }
    }
}

impl OpenApiRouter for Router<AppState> {
    fn openapi(self) -> Self {
        self.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
            .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
            .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
    }
}
