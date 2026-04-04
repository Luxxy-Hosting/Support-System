use crate::{
    models::{
        AdminAssignTicketRequest, AdminListTicketsParams, AdminTicketMessageRequest,
        AdminUpdateTicketCategoryRequest, AdminUpdateTicketPriorityRequest,
        AdminUpdateTicketSettingsRequest, AdminUpdateTicketStatusRequest,
        AdminUpsertTicketCategoryRequest,
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
    models::{admin_activity::GetAdminActivityLogger, user::{GetPermissionManager, GetUser}},
    response::{ApiResponse, ApiResponseResult},
};
use utoipa::ToSchema;
use utoipa_axum::{router::{OpenApiRouter, UtoipaMethodRouterExt}, routes};

use super::State;

fn require_any_admin_permission(
    permissions: &GetPermissionManager,
    required: &[&str],
) -> Result<(), ApiResponse> {
    if required
        .iter()
        .any(|permission| permissions.has_admin_permission(permission).is_ok())
    {
        Ok(())
    } else {
        permissions.has_admin_permission(required[0])
    }
}

mod get_bootstrap {
    use super::*;

    #[derive(ToSchema, Serialize)]
    struct Response {
        support: crate::models::AdminTicketBootstrapResponse,
    }

    #[utoipa::path(get, path = "/bootstrap", responses(
        (status = OK, body = inline(Response)),
        (status = FORBIDDEN, body = ApiError),
    ))]
    pub async fn route(state: GetState, permissions: GetPermissionManager) -> ApiResponseResult {
        require_any_admin_permission(
            &permissions,
            &[
                "tickets.view-all",
                "tickets.manage-settings",
                "tickets.manage-categories",
            ],
        )?;

        let support = manager::get_admin_bootstrap(&state).await?;

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
        ("status" = Option<String>, Query, description = "Filter by status"),
        ("category_uuid" = Option<uuid::Uuid>, Query, description = "Filter by category"),
        ("assigned_user_uuid" = Option<uuid::Uuid>, Query, description = "Filter by assigned staff"),
        ("client" = Option<String>, Query, description = "Filter by client username/email"),
        ("server" = Option<String>, Query, description = "Filter by linked server name"),
        ("priority" = Option<String>, Query, description = "Filter by priority")
    ))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        Query(params): Query<AdminListTicketsParams>,
    ) -> ApiResponseResult {
        permissions.has_admin_permission("tickets.view-all")?;

        if let Err(errors) = shared::utils::validate_data(&params) {
            return ApiResponse::new_serialized(ApiError::new_strings_value(errors))
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }

        let tickets = manager::list_admin_tickets(&state, &params).await?;

        ApiResponse::new_serialized(Response { tickets }).ok()
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
        Path(ticket_uuid): Path<uuid::Uuid>,
    ) -> ApiResponseResult {
        permissions.has_admin_permission("tickets.view-all")?;

        let ticket = manager::get_admin_ticket_detail(&state, ticket_uuid).await?;

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
        Path((ticket_uuid, attachment_uuid)): Path<(uuid::Uuid, uuid::Uuid)>,
    ) -> ApiResponseResult {
        permissions.has_admin_permission("tickets.view-all")?;

        let attachment = manager::get_admin_attachment_download(&state, ticket_uuid, attachment_uuid).await?;

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

mod add_message {
    use super::*;

    #[derive(ToSchema, Serialize)]
    struct Response {
        ticket: crate::models::ApiTicketDetail,
    }

    #[utoipa::path(post, path = "/tickets/{ticket}/messages", request_body = AdminTicketMessageRequest, responses(
        (status = OK, body = inline(Response)),
        (status = BAD_REQUEST, body = ApiError),
        (status = NOT_FOUND, body = ApiError),
        (status = FORBIDDEN, body = ApiError),
    ), params(("ticket" = uuid::Uuid, Path, description = "The ticket UUID")))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        user: GetUser,
        activity_logger: GetAdminActivityLogger,
        Path(ticket_uuid): Path<uuid::Uuid>,
        Payload(request): Payload<AdminTicketMessageRequest>,
    ) -> ApiResponseResult {
        if request.is_internal {
            permissions.has_admin_permission("tickets.add-internal-notes")?;
        } else {
            permissions.has_admin_permission("tickets.reply-all")?;
        }

        if let Err(errors) = shared::utils::validate_data(&request) {
            return ApiResponse::new_serialized(ApiError::new_strings_value(errors))
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }

        let ticket = manager::add_admin_message(
            &state,
            &user,
            ticket_uuid,
            &request.body,
            request.is_internal,
            Vec::new(),
        )
        .await?;

        activity_logger
            .log(
                if request.is_internal { "tickets:add_internal_note" } else { "tickets:reply" },
                serde_json::json!({
                    "ticket_uuid": ticket.ticket.uuid,
                }),
            )
            .await;

        ApiResponse::new_serialized(Response { ticket }).ok()
    }
}

mod add_message_upload {
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
        activity_logger: GetAdminActivityLogger,
        Path(ticket_uuid): Path<uuid::Uuid>,
        multipart: Multipart,
    ) -> ApiResponseResult {
        let form = multipart::parse_message_form(multipart).await?;

        if form.is_internal {
            permissions.has_admin_permission("tickets.add-internal-notes")?;
        } else {
            permissions.has_admin_permission("tickets.reply-all")?;
        }

        let is_internal = form.is_internal;

        let ticket = manager::add_admin_message(
            &state,
            &user,
            ticket_uuid,
            &form.body,
            is_internal,
            form.attachments,
        )
        .await?;

        activity_logger
            .log(
                if is_internal { "tickets:add_internal_note" } else { "tickets:reply" },
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

    #[utoipa::path(patch, path = "/tickets/{ticket}/status", request_body = AdminUpdateTicketStatusRequest, responses(
        (status = OK, body = inline(Response)),
        (status = BAD_REQUEST, body = ApiError),
        (status = NOT_FOUND, body = ApiError),
        (status = FORBIDDEN, body = ApiError),
    ), params(("ticket" = uuid::Uuid, Path, description = "The ticket UUID")))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        user: GetUser,
        activity_logger: GetAdminActivityLogger,
        Path(ticket_uuid): Path<uuid::Uuid>,
        Payload(request): Payload<AdminUpdateTicketStatusRequest>,
    ) -> ApiResponseResult {
        permissions.has_admin_permission("tickets.change-status")?;

        if let Err(errors) = shared::utils::validate_data(&request) {
            return ApiResponse::new_serialized(ApiError::new_strings_value(errors))
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }

        let ticket = manager::update_admin_ticket_status(&state, &user, ticket_uuid, &request.status).await?;

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

mod assign_ticket {
    use super::*;

    #[derive(ToSchema, Serialize)]
    struct Response {
        ticket: crate::models::ApiTicketDetail,
    }

    #[utoipa::path(patch, path = "/tickets/{ticket}/assignee", request_body = AdminAssignTicketRequest, responses(
        (status = OK, body = inline(Response)),
        (status = BAD_REQUEST, body = ApiError),
        (status = NOT_FOUND, body = ApiError),
        (status = FORBIDDEN, body = ApiError),
    ), params(("ticket" = uuid::Uuid, Path, description = "The ticket UUID")))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        user: GetUser,
        activity_logger: GetAdminActivityLogger,
        Path(ticket_uuid): Path<uuid::Uuid>,
        Payload(request): Payload<AdminAssignTicketRequest>,
    ) -> ApiResponseResult {
        permissions.has_admin_permission("tickets.assign")?;

        let ticket = manager::assign_ticket(&state, &user, ticket_uuid, request.assigned_user_uuid).await?;

        activity_logger
            .log(
                "tickets:assign",
                serde_json::json!({
                    "ticket_uuid": ticket.ticket.uuid,
                    "assigned_user_uuid": ticket.ticket.assigned_user.as_ref().map(|value| value.uuid),
                }),
            )
            .await;

        ApiResponse::new_serialized(Response { ticket }).ok()
    }
}

mod update_priority {
    use super::*;

    #[derive(ToSchema, Serialize)]
    struct Response {
        ticket: crate::models::ApiTicketDetail,
    }

    #[utoipa::path(patch, path = "/tickets/{ticket}/priority", request_body = AdminUpdateTicketPriorityRequest, responses(
        (status = OK, body = inline(Response)),
        (status = BAD_REQUEST, body = ApiError),
        (status = NOT_FOUND, body = ApiError),
        (status = FORBIDDEN, body = ApiError),
    ), params(("ticket" = uuid::Uuid, Path, description = "The ticket UUID")))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        user: GetUser,
        activity_logger: GetAdminActivityLogger,
        Path(ticket_uuid): Path<uuid::Uuid>,
        Payload(request): Payload<AdminUpdateTicketPriorityRequest>,
    ) -> ApiResponseResult {
        permissions.has_admin_permission("tickets.change-status")?;

        let ticket = manager::update_ticket_priority(&state, &user, ticket_uuid, request.priority.as_deref()).await?;

        activity_logger
            .log(
                "tickets:update_priority",
                serde_json::json!({
                    "ticket_uuid": ticket.ticket.uuid,
                    "priority": ticket.ticket.priority,
                }),
            )
            .await;

        ApiResponse::new_serialized(Response { ticket }).ok()
    }
}

mod update_category {
    use super::*;

    #[derive(ToSchema, Serialize)]
    struct Response {
        ticket: crate::models::ApiTicketDetail,
    }

    #[utoipa::path(patch, path = "/tickets/{ticket}/category", request_body = AdminUpdateTicketCategoryRequest, responses(
        (status = OK, body = inline(Response)),
        (status = BAD_REQUEST, body = ApiError),
        (status = NOT_FOUND, body = ApiError),
        (status = FORBIDDEN, body = ApiError),
    ), params(("ticket" = uuid::Uuid, Path, description = "The ticket UUID")))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        user: GetUser,
        activity_logger: GetAdminActivityLogger,
        Path(ticket_uuid): Path<uuid::Uuid>,
        Payload(request): Payload<AdminUpdateTicketCategoryRequest>,
    ) -> ApiResponseResult {
        permissions.has_admin_permission("tickets.change-status")?;

        let ticket = manager::update_ticket_category(&state, &user, ticket_uuid, request.category_uuid).await?;

        activity_logger
            .log(
                "tickets:update_category",
                serde_json::json!({
                    "ticket_uuid": ticket.ticket.uuid,
                    "category_uuid": ticket.ticket.category.as_ref().map(|value| value.uuid),
                }),
            )
            .await;

        ApiResponse::new_serialized(Response { ticket }).ok()
    }
}

mod delete_ticket {
    use super::*;

    #[derive(ToSchema, Serialize)]
    struct Response {}

    #[utoipa::path(delete, path = "/tickets/{ticket}", responses(
        (status = OK, body = inline(Response)),
        (status = NOT_FOUND, body = ApiError),
        (status = FORBIDDEN, body = ApiError),
    ), params(("ticket" = uuid::Uuid, Path, description = "The ticket UUID")))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        user: GetUser,
        activity_logger: GetAdminActivityLogger,
        Path(ticket_uuid): Path<uuid::Uuid>,
    ) -> ApiResponseResult {
        permissions.has_admin_permission("tickets.delete")?;

        manager::soft_delete_ticket(&state, &user, ticket_uuid).await?;

        activity_logger
            .log(
                "tickets:delete",
                serde_json::json!({
                    "ticket_uuid": ticket_uuid,
                }),
            )
            .await;

        ApiResponse::new_serialized(Response {}).ok()
    }
}

mod update_settings {
    use super::*;

    #[derive(ToSchema, Serialize)]
    struct Response {
        settings: crate::models::AdminTicketSettingsDetailResponse,
    }

    #[utoipa::path(get, path = "/settings", responses(
        (status = OK, body = inline(Response)),
        (status = FORBIDDEN, body = ApiError),
    ))]
    pub async fn get_route(
        state: GetState,
        permissions: GetPermissionManager,
    ) -> ApiResponseResult {
        permissions.has_admin_permission("tickets.manage-settings")?;

        let settings = manager::get_admin_settings_detail(&state).await?;

        ApiResponse::new_serialized(Response { settings }).ok()
    }

    #[utoipa::path(put, path = "/settings", request_body = AdminUpdateTicketSettingsRequest, responses(
        (status = OK, body = inline(Response)),
        (status = BAD_REQUEST, body = ApiError),
        (status = FORBIDDEN, body = ApiError),
    ))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        activity_logger: GetAdminActivityLogger,
        Payload(request): Payload<AdminUpdateTicketSettingsRequest>,
    ) -> ApiResponseResult {
        permissions.has_admin_permission("tickets.manage-settings")?;

        if let Err(errors) = shared::utils::validate_data(&request) {
            return ApiResponse::new_serialized(ApiError::new_strings_value(errors))
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }

        let settings = manager::update_settings(
            &state,
            request.categories_enabled,
            request.allow_client_close,
            request.allow_reply_on_closed,
            request.discord_webhook_enabled,
            request.discord_webhook_url,
            request.discord_notify_on_ticket_created,
            request.discord_notify_on_client_reply,
            request.discord_notify_on_staff_reply,
            request.discord_notify_on_internal_note,
            request.discord_notify_on_status_change,
            request.discord_notify_on_assignment_change,
            request.discord_notify_on_ticket_deleted,
        )
        .await?;

        activity_logger
            .log(
                "tickets:update_settings",
                serde_json::json!({
                    "categories_enabled": settings.settings.categories_enabled,
                    "allow_client_close": settings.settings.allow_client_close,
                    "allow_reply_on_closed": settings.settings.allow_reply_on_closed,
                    "discord_webhook_enabled": settings.discord_webhook.enabled,
                }),
            )
            .await;

        ApiResponse::new_serialized(Response { settings }).ok()
    }
}

mod upsert_category {
    use super::*;

    #[derive(ToSchema, Serialize)]
    struct Response {
        category: crate::models::ApiTicketCategory,
    }

    #[utoipa::path(put, path = "/categories", request_body = AdminUpsertTicketCategoryRequest, responses(
        (status = OK, body = inline(Response)),
        (status = BAD_REQUEST, body = ApiError),
        (status = NOT_FOUND, body = ApiError),
        (status = FORBIDDEN, body = ApiError),
    ))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        activity_logger: GetAdminActivityLogger,
        Payload(request): Payload<AdminUpsertTicketCategoryRequest>,
    ) -> ApiResponseResult {
        permissions.has_admin_permission("tickets.manage-categories")?;

        if let Err(errors) = shared::utils::validate_data(&request) {
            return ApiResponse::new_serialized(ApiError::new_strings_value(errors))
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }

        let category = manager::upsert_category(
            &state,
            request.uuid,
            &request.name,
            request.description.as_deref(),
            request.color.as_deref(),
            request.sort_order,
            request.enabled,
        )
        .await?;

        activity_logger
            .log(
                if request.uuid.is_some() { "tickets:update_category_definition" } else { "tickets:create_category" },
                serde_json::json!({
                    "category_uuid": category.uuid,
                    "name": category.name,
                }),
            )
            .await;

        ApiResponse::new_serialized(Response { category }).ok()
    }
}

mod delete_category {
    use super::*;

    #[derive(ToSchema, Serialize)]
    struct Response {}

    #[utoipa::path(delete, path = "/categories/{category}", responses(
        (status = OK, body = inline(Response)),
        (status = NOT_FOUND, body = ApiError),
        (status = FORBIDDEN, body = ApiError),
    ), params(("category" = uuid::Uuid, Path, description = "The category UUID")))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        activity_logger: GetAdminActivityLogger,
        Path(category_uuid): Path<uuid::Uuid>,
    ) -> ApiResponseResult {
        permissions.has_admin_permission("tickets.manage-categories")?;

        manager::delete_category(&state, category_uuid).await?;

        activity_logger
            .log(
                "tickets:delete_category",
                serde_json::json!({
                    "category_uuid": category_uuid,
                }),
            )
            .await;

        ApiResponse::new_serialized(Response {}).ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get_bootstrap::route))
        .routes(routes!(list_tickets::route))
        .routes(routes!(get_ticket::route))
        .routes(routes!(get_attachment::route))
        .routes(routes!(add_message::route))
        .routes(routes!(add_message_upload::route).layer(DefaultBodyLimit::disable()))
        .routes(routes!(update_status::route))
        .routes(routes!(assign_ticket::route))
        .routes(routes!(update_priority::route))
        .routes(routes!(update_category::route))
        .routes(routes!(delete_ticket::route))
        .routes(routes!(update_settings::get_route))
        .routes(routes!(update_settings::route))
        .routes(routes!(upsert_category::route))
        .routes(routes!(delete_category::route))
        .with_state(state.clone())
}
