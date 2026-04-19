use crate::{
    models::{
        ClientCreateTicketRequest, ClientListTicketsParams, ClientReplyTicketRequest,
        ClientUpdateTicketStatusRequest,
    },
    routes::multipart,
    services::manager,
};
use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Multipart, Path, Query},
    http::StatusCode,
};
use serde::Serialize;
use shared::{
    ApiError, GetState, Payload,
    models::{
        user::{GetPermissionManager, GetUser},
        user_activity::GetUserActivityLogger,
    },
    response::{ApiResponse, ApiResponseResult},
};
use utoipa::ToSchema;
use utoipa_axum::{
    router::{OpenApiRouter, UtoipaMethodRouterExt},
    routes,
};

use super::State;

mod get_bootstrap {
    use super::*;

    #[derive(ToSchema, Serialize)]
    struct Response {
        support: crate::models::ClientTicketBootstrapResponse,
    }

    #[utoipa::path(get, path = "/bootstrap", responses(
        (status = OK, body = inline(Response)),
        (status = FORBIDDEN, body = ApiError),
    ))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        user: GetUser,
    ) -> ApiResponseResult {
        permissions.has_user_permission("tickets.view-own")?;

        let support = manager::get_client_bootstrap(&state, &user).await?;

        ApiResponse::new_serialized(Response { support }).ok()
    }
}

mod list_tickets {
    use super::*;

    #[derive(ToSchema, Serialize)]
    struct Response {
        #[schema(inline)]
        tickets: shared::models::Pagination<crate::models::ApiTicketSummary>,
    }

    #[utoipa::path(get, path = "/tickets", responses(
        (status = OK, body = inline(Response)),
        (status = BAD_REQUEST, body = ApiError),
        (status = FORBIDDEN, body = ApiError),
    ), params(
        ("page" = i64, Query, description = "The page number"),
        ("per_page" = i64, Query, description = "The number of items per page"),
        ("search" = Option<String>, Query, description = "Search term"),
        ("status" = Option<String>, Query, description = "Filter by status")
    ))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        user: GetUser,
        Query(params): Query<ClientListTicketsParams>,
    ) -> ApiResponseResult {
        permissions.has_user_permission("tickets.view-own")?;

        if let Err(errors) = shared::utils::validate_data(&params) {
            return ApiResponse::new_serialized(ApiError::new_strings_value(errors))
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }

        let tickets = manager::list_client_tickets(&state, &user, &params).await?;

        ApiResponse::new_serialized(Response { tickets }).ok()
    }
}

mod create_ticket {
    use super::*;

    #[derive(ToSchema, Serialize)]
    struct Response {
        ticket: crate::models::ApiTicketDetail,
    }

    #[utoipa::path(post, path = "/tickets", request_body = ClientCreateTicketRequest, responses(
        (status = OK, body = inline(Response)),
        (status = BAD_REQUEST, body = ApiError),
        (status = FORBIDDEN, body = ApiError),
    ))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        user: GetUser,
        activity_logger: GetUserActivityLogger,
        Payload(request): Payload<ClientCreateTicketRequest>,
    ) -> ApiResponseResult {
        permissions.has_user_permission("tickets.create")?;

        if let Err(errors) = shared::utils::validate_data(&request) {
            return ApiResponse::new_serialized(ApiError::new_strings_value(errors))
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }

        let ticket = manager::create_ticket(&state, &user, request, Vec::new()).await?;

        activity_logger
            .log(
                "tickets:create",
                serde_json::json!({
                    "ticket_uuid": ticket.ticket.uuid,
                    "subject": ticket.ticket.subject,
                    "linked_server_uuid": ticket.ticket.linked_server.uuid,
                }),
            )
            .await;

        ApiResponse::new_serialized(Response { ticket }).ok()
    }
}

mod create_ticket_upload {
    use super::*;

    #[derive(ToSchema, Serialize)]
    struct Response {
        ticket: crate::models::ApiTicketDetail,
    }

    #[utoipa::path(post, path = "/tickets/upload", responses(
        (status = OK, body = inline(Response)),
        (status = BAD_REQUEST, body = ApiError),
        (status = FORBIDDEN, body = ApiError),
    ), params())]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        user: GetUser,
        activity_logger: GetUserActivityLogger,
        multipart: Multipart,
    ) -> ApiResponseResult {
        permissions.has_user_permission("tickets.create")?;

        let form = multipart::parse_create_ticket_form(multipart).await?;
        let ticket = manager::create_ticket(
            &state,
            &user,
            ClientCreateTicketRequest {
                server_uuid: form.server_uuid,
                category_uuid: form.category_uuid,
                subject: form.subject,
                message: form.message,
                metadata: form.metadata,
            },
            form.attachments,
        )
        .await?;

        activity_logger
            .log(
                "tickets:create",
                serde_json::json!({
                    "ticket_uuid": ticket.ticket.uuid,
                    "subject": ticket.ticket.subject,
                    "linked_server_uuid": ticket.ticket.linked_server.uuid,
                }),
            )
            .await;

        ApiResponse::new_serialized(Response { ticket }).ok()
    }
}

mod get_ticket {
    use super::*;

    #[derive(ToSchema, Serialize)]
    struct Response {
        ticket: crate::models::ApiTicketDetail,
    }

    #[utoipa::path(get, path = "/tickets/{ticket}", responses(
        (status = OK, body = inline(Response)),
        (status = NOT_FOUND, body = ApiError),
        (status = FORBIDDEN, body = ApiError),
    ), params(("ticket" = uuid::Uuid, Path, description = "The ticket UUID")))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        user: GetUser,
        Path(ticket_uuid): Path<uuid::Uuid>,
    ) -> ApiResponseResult {
        permissions.has_user_permission("tickets.view-own")?;

        let ticket = manager::get_client_ticket_detail(&state, &user, ticket_uuid).await?;

        ApiResponse::new_serialized(Response { ticket }).ok()
    }
}

mod get_attachment {
    use super::*;

    #[utoipa::path(get, path = "/tickets/{ticket}/attachments/{attachment}", responses(
        (status = OK, description = "Ticket attachment content"),
        (status = NOT_FOUND, body = ApiError),
        (status = FORBIDDEN, body = ApiError),
    ), params(
        ("ticket" = uuid::Uuid, Path, description = "The ticket UUID"),
        ("attachment" = uuid::Uuid, Path, description = "The attachment UUID")
    ))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        user: GetUser,
        Path((ticket_uuid, attachment_uuid)): Path<(uuid::Uuid, uuid::Uuid)>,
    ) -> ApiResponseResult {
        permissions.has_user_permission("tickets.view-own")?;

        let attachment =
            manager::get_client_attachment_download(&state, &user, ticket_uuid, attachment_uuid)
                .await?;

        ApiResponse::new(Body::from(attachment.bytes))
            .with_header("Content-Type", attachment.content_type)
            .with_header("Content-Length", attachment.size.to_string())
            .with_header(
                "Content-Disposition",
                format!("inline; filename=\"{}\"", attachment.original_name),
            )
            .ok()
    }
}

mod reply_ticket {
    use super::*;

    #[derive(ToSchema, Serialize)]
    struct Response {
        ticket: crate::models::ApiTicketDetail,
    }

    #[utoipa::path(post, path = "/tickets/{ticket}/messages", request_body = ClientReplyTicketRequest, responses(
        (status = OK, body = inline(Response)),
        (status = BAD_REQUEST, body = ApiError),
        (status = NOT_FOUND, body = ApiError),
        (status = FORBIDDEN, body = ApiError),
    ), params(("ticket" = uuid::Uuid, Path, description = "The ticket UUID")))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        user: GetUser,
        activity_logger: GetUserActivityLogger,
        Path(ticket_uuid): Path<uuid::Uuid>,
        Payload(request): Payload<ClientReplyTicketRequest>,
    ) -> ApiResponseResult {
        permissions.has_user_permission("tickets.reply-own")?;

        if let Err(errors) = shared::utils::validate_data(&request) {
            return ApiResponse::new_serialized(ApiError::new_strings_value(errors))
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }

        let ticket =
            manager::add_client_reply(&state, &user, ticket_uuid, &request.body, Vec::new())
                .await?;

        activity_logger
            .log(
                "tickets:reply",
                serde_json::json!({
                    "ticket_uuid": ticket.ticket.uuid,
                }),
            )
            .await;

        ApiResponse::new_serialized(Response { ticket }).ok()
    }
}

mod reply_ticket_upload {
    use super::*;

    #[derive(ToSchema, Serialize)]
    struct Response {
        ticket: crate::models::ApiTicketDetail,
    }

    #[utoipa::path(post, path = "/tickets/{ticket}/messages/upload", responses(
        (status = OK, body = inline(Response)),
        (status = BAD_REQUEST, body = ApiError),
        (status = NOT_FOUND, body = ApiError),
        (status = FORBIDDEN, body = ApiError),
    ), params(("ticket" = uuid::Uuid, Path, description = "The ticket UUID")))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        user: GetUser,
        activity_logger: GetUserActivityLogger,
        Path(ticket_uuid): Path<uuid::Uuid>,
        multipart: Multipart,
    ) -> ApiResponseResult {
        permissions.has_user_permission("tickets.reply-own")?;

        let form = multipart::parse_message_form(multipart).await?;
        let ticket =
            manager::add_client_reply(&state, &user, ticket_uuid, &form.body, form.attachments)
                .await?;

        activity_logger
            .log(
                "tickets:reply",
                serde_json::json!({
                    "ticket_uuid": ticket.ticket.uuid,
                }),
            )
            .await;

        ApiResponse::new_serialized(Response { ticket }).ok()
    }
}

mod update_status {
    use super::*;

    #[derive(ToSchema, Serialize)]
    struct Response {
        ticket: crate::models::ApiTicketDetail,
    }

    #[utoipa::path(patch, path = "/tickets/{ticket}/status", request_body = ClientUpdateTicketStatusRequest, responses(
        (status = OK, body = inline(Response)),
        (status = BAD_REQUEST, body = ApiError),
        (status = NOT_FOUND, body = ApiError),
        (status = FORBIDDEN, body = ApiError),
    ), params(("ticket" = uuid::Uuid, Path, description = "The ticket UUID")))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        user: GetUser,
        activity_logger: GetUserActivityLogger,
        Path(ticket_uuid): Path<uuid::Uuid>,
        Payload(request): Payload<ClientUpdateTicketStatusRequest>,
    ) -> ApiResponseResult {
        permissions.has_user_permission("tickets.reply-own")?;

        if let Err(errors) = shared::utils::validate_data(&request) {
            return ApiResponse::new_serialized(ApiError::new_strings_value(errors))
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }

        let ticket =
            manager::update_client_ticket_status(&state, &user, ticket_uuid, &request.status)
                .await?;

        activity_logger
            .log(
                "tickets:update_status",
                serde_json::json!({
                    "ticket_uuid": ticket.ticket.uuid,
                    "status": ticket.ticket.status,
                }),
            )
            .await;

        ApiResponse::new_serialized(Response { ticket }).ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get_bootstrap::route))
        .routes(routes!(list_tickets::route))
        .routes(routes!(create_ticket::route))
        .routes(routes!(create_ticket_upload::route).layer(DefaultBodyLimit::disable()))
        .routes(routes!(get_ticket::route))
        .routes(routes!(get_attachment::route))
        .routes(routes!(reply_ticket::route))
        .routes(routes!(reply_ticket_upload::route).layer(DefaultBodyLimit::disable()))
        .routes(routes!(update_status::route))
        .with_state(state.clone())
}
