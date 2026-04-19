use crate::{
    models::{
        AdminListTicketsParams, AdminTicketBootstrapResponse, AdminTicketSettingsDetailResponse,
        ApiDiscordWebhookSettings, ApiTicketAttachment, ApiTicketAuditEvent, ApiTicketCategory,
        ApiTicketCategorySummary, ApiTicketDetail, ApiTicketLinkedServer, ApiTicketMessage,
        ApiTicketServerOption, ApiTicketSettings, ApiTicketSummary, ApiTicketUserSummary,
        ClientCreateTicketRequest, ClientListTicketsParams, ClientTicketBootstrapResponse,
        TicketActorType, TicketPriority, TicketStatus,
    },
    services::discord_webhook::{
        DiscordWebhookConfig, DiscordWebhookEvent, DiscordWebhookEventKind, send_ticket_event,
    },
};
use axum::http::StatusCode;
use chrono::Utc;
use reqwest::Url;
use serde_json::json;
use shared::{
    State,
    models::{ByUuid, Pagination, server::Server, user::User},
    response::DisplayError,
};
use sqlx::{FromRow, Postgres, QueryBuilder};
use std::collections::HashMap;
use tokio::io::AsyncReadExt;
use tokio_util::io::StreamReader;
use tracing::warn;

#[derive(Clone, FromRow)]
struct TicketSettingsRow {
    uuid: uuid::Uuid,
    categories_enabled: bool,
    allow_client_close: bool,
    allow_reply_on_closed: bool,
    create_ticket_rate_limit_hits: i32,
    create_ticket_rate_limit_window_seconds: i32,
    max_open_tickets_per_user: i32,
    discord_webhook_enabled: bool,
    discord_webhook_url: Option<String>,
    discord_notify_on_ticket_created: bool,
    discord_notify_on_client_reply: bool,
    discord_notify_on_staff_reply: bool,
    discord_notify_on_internal_note: bool,
    discord_notify_on_status_change: bool,
    discord_notify_on_assignment_change: bool,
    discord_notify_on_ticket_deleted: bool,
    created: chrono::NaiveDateTime,
    updated: chrono::NaiveDateTime,
}

#[derive(FromRow)]
struct TicketCategoryRow {
    uuid: uuid::Uuid,
    name: String,
    description: Option<String>,
    color: Option<String>,
    sort_order: i32,
    enabled: bool,
    created: chrono::NaiveDateTime,
    updated: chrono::NaiveDateTime,
}

#[derive(FromRow)]
struct TicketSummaryRow {
    uuid: uuid::Uuid,
    subject: String,
    status: String,
    priority: Option<String>,
    creator_uuid: uuid::Uuid,
    creator_username: String,
    creator_name_first: String,
    creator_name_last: String,
    creator_admin: bool,
    category_uuid: Option<uuid::Uuid>,
    category_name: Option<String>,
    category_color: Option<String>,
    assigned_user_uuid: Option<uuid::Uuid>,
    assigned_user_username: Option<String>,
    assigned_user_name_first: Option<String>,
    assigned_user_name_last: Option<String>,
    assigned_user_admin: Option<bool>,
    linked_server_uuid: Option<uuid::Uuid>,
    linked_server_snapshot_name: Option<String>,
    linked_server_snapshot_uuid_short: Option<i32>,
    linked_server_deleted_at: Option<chrono::NaiveDateTime>,
    current_server_name: Option<String>,
    current_server_uuid_short: Option<i32>,
    current_server_status: Option<String>,
    current_server_suspended: Option<bool>,
    current_server_owner_username: Option<String>,
    last_reply_at: Option<chrono::NaiveDateTime>,
    last_reply_by_type: Option<String>,
    created: chrono::NaiveDateTime,
    updated: chrono::NaiveDateTime,
    closed_at: Option<chrono::NaiveDateTime>,
    total_count: i64,
}

#[derive(FromRow)]
struct TicketDetailRow {
    uuid: uuid::Uuid,
    subject: String,
    status: String,
    priority: Option<String>,
    metadata: serde_json::Value,
    creator_uuid: uuid::Uuid,
    creator_username: String,
    creator_name_first: String,
    creator_name_last: String,
    creator_admin: bool,
    category_uuid: Option<uuid::Uuid>,
    category_name: Option<String>,
    category_color: Option<String>,
    assigned_user_uuid: Option<uuid::Uuid>,
    assigned_user_username: Option<String>,
    assigned_user_name_first: Option<String>,
    assigned_user_name_last: Option<String>,
    assigned_user_admin: Option<bool>,
    linked_server_uuid: Option<uuid::Uuid>,
    linked_server_snapshot_name: Option<String>,
    linked_server_snapshot_uuid_short: Option<i32>,
    linked_server_deleted_at: Option<chrono::NaiveDateTime>,
    current_server_name: Option<String>,
    current_server_uuid_short: Option<i32>,
    current_server_status: Option<String>,
    current_server_suspended: Option<bool>,
    current_server_owner_username: Option<String>,
    last_reply_at: Option<chrono::NaiveDateTime>,
    last_reply_by_type: Option<String>,
    created: chrono::NaiveDateTime,
    updated: chrono::NaiveDateTime,
    closed_at: Option<chrono::NaiveDateTime>,
}

#[derive(FromRow)]
struct TicketMessageRow {
    uuid: uuid::Uuid,
    author_user_uuid: Option<uuid::Uuid>,
    author_snapshot_username: String,
    author_snapshot_display_name: String,
    author_avatar: Option<String>,
    author_type: String,
    body: String,
    is_internal: bool,
    created: chrono::NaiveDateTime,
    updated: chrono::NaiveDateTime,
}

#[derive(FromRow)]
struct TicketAttachmentRow {
    uuid: uuid::Uuid,
    message_uuid: uuid::Uuid,
    original_name: String,
    content_type: String,
    media_type: String,
    size: i64,
    created: chrono::NaiveDateTime,
}

#[derive(FromRow)]
struct TicketAttachmentDownloadRow {
    original_name: String,
    content_type: String,
    size: i64,
    storage_path: String,
}

#[derive(FromRow)]
struct TicketAuditRow {
    uuid: uuid::Uuid,
    actor_user_uuid: Option<uuid::Uuid>,
    actor_snapshot_username: Option<String>,
    actor_type: String,
    event: String,
    payload: serde_json::Value,
    created: chrono::NaiveDateTime,
}

#[derive(FromRow)]
struct StaffUserRow {
    uuid: uuid::Uuid,
    username: String,
    name_first: String,
    name_last: String,
    admin: bool,
}

pub struct IncomingAttachmentUpload {
    pub original_name: String,
    pub content_type: String,
    pub media_type: String,
    pub data: axum::body::Bytes,
}

pub struct TicketAttachmentDownload {
    pub original_name: String,
    pub content_type: String,
    pub size: i64,
    pub bytes: axum::body::Bytes,
}

struct TicketActor<'a> {
    user_uuid: Option<uuid::Uuid>,
    username: &'a str,
    display_name: String,
    actor_type: TicketActorType,
}

#[derive(Clone, Copy)]
enum AttachmentUrlScope {
    Client,
    Admin,
}

const SUMMARY_SELECT: &str = r#"
    SELECT
        t.uuid,
        t.subject,
        t.status,
        t.priority,
        creator.uuid AS creator_uuid,
        creator.username AS creator_username,
        creator.name_first AS creator_name_first,
        creator.name_last AS creator_name_last,
        creator.admin AS creator_admin,
        category.uuid AS category_uuid,
        category.name AS category_name,
        category.color AS category_color,
        assignee.uuid AS assigned_user_uuid,
        assignee.username AS assigned_user_username,
        assignee.name_first AS assigned_user_name_first,
        assignee.name_last AS assigned_user_name_last,
        assignee.admin AS assigned_user_admin,
        t.linked_server_uuid,
        t.linked_server_snapshot_name,
        t.linked_server_snapshot_uuid_short,
        t.linked_server_deleted_at,
        linked_server.name AS current_server_name,
        linked_server.uuid_short AS current_server_uuid_short,
        linked_server.status::text AS current_server_status,
        linked_server.suspended AS current_server_suspended,
        linked_server_owner.username AS current_server_owner_username,
        t.last_reply_at,
        t.last_reply_by_type,
        t.created,
        t.updated,
        t.closed_at,
        COUNT(*) OVER() AS total_count
    FROM ext_support_tickets t
    JOIN users creator ON creator.uuid = t.creator_user_uuid
    LEFT JOIN ext_support_ticket_categories category ON category.uuid = t.category_uuid
    LEFT JOIN users assignee ON assignee.uuid = t.assigned_user_uuid
    LEFT JOIN servers linked_server ON linked_server.uuid = t.linked_server_uuid
    LEFT JOIN users linked_server_owner ON linked_server_owner.uuid = linked_server.owner_uuid
"#;

const DETAIL_SELECT: &str = r#"
    SELECT
        t.uuid,
        t.subject,
        t.status,
        t.priority,
        t.metadata,
        creator.uuid AS creator_uuid,
        creator.username AS creator_username,
        creator.name_first AS creator_name_first,
        creator.name_last AS creator_name_last,
        creator.admin AS creator_admin,
        category.uuid AS category_uuid,
        category.name AS category_name,
        category.color AS category_color,
        assignee.uuid AS assigned_user_uuid,
        assignee.username AS assigned_user_username,
        assignee.name_first AS assigned_user_name_first,
        assignee.name_last AS assigned_user_name_last,
        assignee.admin AS assigned_user_admin,
        t.linked_server_uuid,
        t.linked_server_snapshot_name,
        t.linked_server_snapshot_uuid_short,
        t.linked_server_deleted_at,
        linked_server.name AS current_server_name,
        linked_server.uuid_short AS current_server_uuid_short,
        linked_server.status::text AS current_server_status,
        linked_server.suspended AS current_server_suspended,
        linked_server_owner.username AS current_server_owner_username,
        t.last_reply_at,
        t.last_reply_by_type,
        t.created,
        t.updated,
        t.closed_at
    FROM ext_support_tickets t
    JOIN users creator ON creator.uuid = t.creator_user_uuid
    LEFT JOIN ext_support_ticket_categories category ON category.uuid = t.category_uuid
    LEFT JOIN users assignee ON assignee.uuid = t.assigned_user_uuid
    LEFT JOIN servers linked_server ON linked_server.uuid = t.linked_server_uuid
    LEFT JOIN users linked_server_owner ON linked_server_owner.uuid = linked_server.owner_uuid
"#;

fn to_settings_api(row: TicketSettingsRow) -> ApiTicketSettings {
    ApiTicketSettings {
        uuid: row.uuid,
        categories_enabled: row.categories_enabled,
        allow_client_close: row.allow_client_close,
        allow_reply_on_closed: row.allow_reply_on_closed,
        create_ticket_rate_limit_hits: row.create_ticket_rate_limit_hits,
        create_ticket_rate_limit_window_seconds: row.create_ticket_rate_limit_window_seconds,
        max_open_tickets_per_user: row.max_open_tickets_per_user,
        created: row.created.and_utc(),
        updated: row.updated.and_utc(),
    }
}

fn to_discord_webhook_settings_api(row: &TicketSettingsRow) -> ApiDiscordWebhookSettings {
    ApiDiscordWebhookSettings {
        enabled: row.discord_webhook_enabled,
        webhook_url: row.discord_webhook_url.clone(),
        notify_on_ticket_created: row.discord_notify_on_ticket_created,
        notify_on_client_reply: row.discord_notify_on_client_reply,
        notify_on_staff_reply: row.discord_notify_on_staff_reply,
        notify_on_internal_note: row.discord_notify_on_internal_note,
        notify_on_status_change: row.discord_notify_on_status_change,
        notify_on_assignment_change: row.discord_notify_on_assignment_change,
        notify_on_ticket_deleted: row.discord_notify_on_ticket_deleted,
    }
}

fn to_discord_webhook_config(row: &TicketSettingsRow) -> DiscordWebhookConfig {
    DiscordWebhookConfig {
        enabled: row.discord_webhook_enabled,
        webhook_url: row.discord_webhook_url.clone(),
        notify_on_ticket_created: row.discord_notify_on_ticket_created,
        notify_on_client_reply: row.discord_notify_on_client_reply,
        notify_on_staff_reply: row.discord_notify_on_staff_reply,
        notify_on_internal_note: row.discord_notify_on_internal_note,
        notify_on_status_change: row.discord_notify_on_status_change,
        notify_on_assignment_change: row.discord_notify_on_assignment_change,
        notify_on_ticket_deleted: row.discord_notify_on_ticket_deleted,
    }
}

fn to_category_api(row: TicketCategoryRow) -> ApiTicketCategory {
    ApiTicketCategory {
        uuid: row.uuid,
        name: row.name,
        description: row.description,
        color: row.color,
        sort_order: row.sort_order,
        enabled: row.enabled,
        created: row.created.and_utc(),
        updated: row.updated.and_utc(),
    }
}

fn optional_category_summary(row: &TicketSummaryRow) -> Option<ApiTicketCategorySummary> {
    Some(ApiTicketCategorySummary {
        uuid: row.category_uuid?,
        name: row.category_name.clone()?,
        color: row.category_color.clone(),
    })
}

fn optional_category_summary_detail(row: &TicketDetailRow) -> Option<ApiTicketCategorySummary> {
    Some(ApiTicketCategorySummary {
        uuid: row.category_uuid?,
        name: row.category_name.clone()?,
        color: row.category_color.clone(),
    })
}

fn to_user_summary(
    uuid: uuid::Uuid,
    username: String,
    name_first: String,
    name_last: String,
    admin: bool,
) -> ApiTicketUserSummary {
    ApiTicketUserSummary {
        uuid,
        username,
        name_first,
        name_last,
        admin,
    }
}

fn optional_assigned_user(row: &TicketSummaryRow) -> Option<ApiTicketUserSummary> {
    Some(to_user_summary(
        row.assigned_user_uuid?,
        row.assigned_user_username.clone()?,
        row.assigned_user_name_first.clone()?,
        row.assigned_user_name_last.clone()?,
        row.assigned_user_admin.unwrap_or(false),
    ))
}

fn optional_assigned_user_detail(row: &TicketDetailRow) -> Option<ApiTicketUserSummary> {
    Some(to_user_summary(
        row.assigned_user_uuid?,
        row.assigned_user_username.clone()?,
        row.assigned_user_name_first.clone()?,
        row.assigned_user_name_last.clone()?,
        row.assigned_user_admin.unwrap_or(false),
    ))
}

fn to_linked_server_summary_from_summary(row: &TicketSummaryRow) -> ApiTicketLinkedServer {
    ApiTicketLinkedServer {
        uuid: row.linked_server_uuid,
        snapshot_name: row.linked_server_snapshot_name.clone(),
        snapshot_uuid_short: row.linked_server_snapshot_uuid_short,
        deleted_at: row.linked_server_deleted_at.map(|value| value.and_utc()),
        current_name: row.current_server_name.clone(),
        current_uuid_short: row.current_server_uuid_short,
        current_status: row.current_server_status.clone(),
        current_is_suspended: row.current_server_suspended,
        current_owner_username: row.current_server_owner_username.clone(),
    }
}

fn to_linked_server_summary_from_detail(row: &TicketDetailRow) -> ApiTicketLinkedServer {
    ApiTicketLinkedServer {
        uuid: row.linked_server_uuid,
        snapshot_name: row.linked_server_snapshot_name.clone(),
        snapshot_uuid_short: row.linked_server_snapshot_uuid_short,
        deleted_at: row.linked_server_deleted_at.map(|value| value.and_utc()),
        current_name: row.current_server_name.clone(),
        current_uuid_short: row.current_server_uuid_short,
        current_status: row.current_server_status.clone(),
        current_is_suspended: row.current_server_suspended,
        current_owner_username: row.current_server_owner_username.clone(),
    }
}

fn to_ticket_summary(row: TicketSummaryRow) -> ApiTicketSummary {
    let category = optional_category_summary(&row);
    let assigned_user = optional_assigned_user(&row);
    let linked_server = to_linked_server_summary_from_summary(&row);
    let last_reply_at = row.last_reply_at.map(|value| value.and_utc());
    let last_reply_by_type = row.last_reply_by_type.clone();
    let created = row.created.and_utc();
    let updated = row.updated.and_utc();
    let closed_at = row.closed_at.map(|value| value.and_utc());

    ApiTicketSummary {
        uuid: row.uuid,
        subject: row.subject,
        status: row.status,
        priority: row.priority,
        creator: to_user_summary(
            row.creator_uuid,
            row.creator_username,
            row.creator_name_first,
            row.creator_name_last,
            row.creator_admin,
        ),
        category,
        assigned_user,
        linked_server,
        last_reply_at,
        last_reply_by_type,
        created,
        updated,
        closed_at,
    }
}

fn build_attachment_url(
    scope: AttachmentUrlScope,
    ticket_uuid: uuid::Uuid,
    attachment_uuid: uuid::Uuid,
) -> String {
    match scope {
        AttachmentUrlScope::Client => {
            format!("/api/client/support/tickets/{ticket_uuid}/attachments/{attachment_uuid}")
        }
        AttachmentUrlScope::Admin => {
            format!("/api/admin/support/tickets/{ticket_uuid}/attachments/{attachment_uuid}")
        }
    }
}

fn to_ticket_attachment(
    row: TicketAttachmentRow,
    ticket_uuid: uuid::Uuid,
    scope: AttachmentUrlScope,
) -> ApiTicketAttachment {
    ApiTicketAttachment {
        uuid: row.uuid,
        original_name: row.original_name,
        content_type: row.content_type,
        media_type: row.media_type,
        size: row.size,
        url: build_attachment_url(scope, ticket_uuid, row.uuid),
        created: row.created.and_utc(),
    }
}

fn to_ticket_message(
    row: TicketMessageRow,
    attachments: Vec<ApiTicketAttachment>,
    storage_url_retriever: &shared::storage::StorageUrlRetriever,
) -> ApiTicketMessage {
    ApiTicketMessage {
        uuid: row.uuid,
        author_user_uuid: row.author_user_uuid,
        author_username: row.author_snapshot_username,
        author_display_name: row.author_snapshot_display_name,
        author_avatar: row
            .author_avatar
            .as_ref()
            .map(|avatar| storage_url_retriever.get_url(avatar)),
        author_type: row.author_type,
        body: row.body,
        is_internal: row.is_internal,
        attachments,
        created: row.created.and_utc(),
        updated: row.updated.and_utc(),
    }
}

fn to_ticket_audit(row: TicketAuditRow) -> ApiTicketAuditEvent {
    ApiTicketAuditEvent {
        uuid: row.uuid,
        actor_user_uuid: row.actor_user_uuid,
        actor_username: row.actor_snapshot_username,
        actor_type: row.actor_type,
        event: row.event,
        payload: row.payload,
        created: row.created.and_utc(),
    }
}

fn to_client_server_option(server: Server) -> ApiTicketServerOption {
    ApiTicketServerOption {
        uuid: server.uuid,
        uuid_short: server.uuid_short,
        name: server.name.to_string(),
        owner_username: server.owner.username.to_string(),
        nest_name: server.nest.name.to_string(),
        egg_name: server.egg.name.to_string(),
        is_suspended: server.suspended,
        status: server.status.map(|status| match status {
            shared::models::server::ServerStatus::Installing => "installing".to_string(),
            shared::models::server::ServerStatus::InstallFailed => "install_failed".to_string(),
            shared::models::server::ServerStatus::RestoringBackup => "restoring_backup".to_string(),
        }),
    }
}

fn normalize_subject(value: &str) -> String {
    value.trim().to_string()
}

fn normalize_body(value: &str) -> String {
    value.trim().to_string()
}

fn ensure_message_or_attachments(
    body: &str,
    attachments: &[IncomingAttachmentUpload],
    empty_message: &str,
) -> Result<(), anyhow::Error> {
    if body.is_empty() && attachments.is_empty() {
        return Err(DisplayError::new(empty_message.to_string()).into());
    }

    Ok(())
}

fn build_attachment_storage_path(
    ticket_uuid: uuid::Uuid,
    message_uuid: uuid::Uuid,
    attachment_uuid: uuid::Uuid,
    original_name: &str,
) -> String {
    format!(
        "support/tickets/{ticket_uuid}/messages/{message_uuid}/{attachment_uuid}_{original_name}"
    )
}

async fn load_attachment_bytes_from_storage(
    state: &State,
    storage_path: &str,
) -> Result<axum::body::Bytes, anyhow::Error> {
    let settings = state.settings.get().await?;

    match &settings.storage_driver {
        shared::settings::StorageDriver::Filesystem { .. } => {
            let base_filesystem = settings
                .storage_driver
                .get_cap_filesystem()
                .await
                .expect("filesystem storage driver must provide a filesystem")?;
            drop(settings);

            let mut file = base_filesystem
                .async_open(storage_path)
                .await
                .map_err(|err| {
                    if err
                        .downcast_ref::<std::io::Error>()
                        .is_some_and(|io_error| io_error.kind() == std::io::ErrorKind::NotFound)
                    {
                        DisplayError::new("attachment not found")
                            .with_status(axum::http::StatusCode::NOT_FOUND)
                            .into()
                    } else {
                        err
                    }
                })?;

            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).await?;
            Ok(bytes.into())
        }
        shared::settings::StorageDriver::S3 {
            access_key,
            secret_key,
            bucket,
            region,
            endpoint,
            path_style,
            ..
        } => {
            let credentials = s3::creds::Credentials::new(
                Some(access_key.as_str()),
                Some(secret_key.as_str()),
                None,
                None,
                None,
            )?;
            let mut s3_bucket = s3::Bucket::new(
                bucket.as_str(),
                s3::Region::Custom {
                    region: region.to_string(),
                    endpoint: endpoint.to_string(),
                },
                credentials,
            )?;

            if *path_style {
                s3_bucket.set_path_style();
            }

            drop(settings);

            let response = s3_bucket.get_object(storage_path).await?;
            match response.status_code() {
                200..=299 => Ok(response.into_bytes()),
                404 => Err(DisplayError::new("attachment not found")
                    .with_status(axum::http::StatusCode::NOT_FOUND)
                    .into()),
                _ => Err(DisplayError::new("failed to read attachment")
                    .with_status(axum::http::StatusCode::BAD_GATEWAY)
                    .into()),
            }
        }
    }
}

fn normalize_priority(value: Option<&str>) -> Result<Option<&'static str>, anyhow::Error> {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => Ok(Some(
            TicketPriority::from_str(value)
                .ok_or_else(|| DisplayError::new("invalid ticket priority"))?
                .as_str(),
        )),
        None => Ok(None),
    }
}

fn normalize_status(value: &str) -> Result<&'static str, anyhow::Error> {
    Ok(TicketStatus::from_str(value.trim())
        .ok_or_else(|| DisplayError::new("invalid ticket status"))?
        .as_str())
}

fn validate_category_color(color: Option<&str>) -> Result<(), anyhow::Error> {
    let Some(color) = color else {
        return Ok(());
    };

    let color = color.trim();
    let valid = (color.len() == 7 || color.len() == 4)
        && color.starts_with('#')
        && color.chars().skip(1).all(|char| char.is_ascii_hexdigit());

    if !valid {
        return Err(DisplayError::new("category color must be a hex value like #4f46e5").into());
    }

    Ok(())
}

fn normalize_metadata(
    metadata: Option<serde_json::Value>,
) -> Result<serde_json::Value, anyhow::Error> {
    match metadata {
        None => Ok(json!({})),
        Some(serde_json::Value::Object(map)) => {
            let value = serde_json::Value::Object(map);
            if serde_json::to_vec(&value)?.len() > 8192 {
                return Err(DisplayError::new("ticket metadata is too large").into());
            }
            Ok(value)
        }
        Some(_) => Err(DisplayError::new("ticket metadata must be a JSON object").into()),
    }
}

fn normalize_discord_webhook_url(value: Option<String>) -> Result<Option<String>, anyhow::Error> {
    let Some(webhook_url) = value
        .map(|entry| entry.trim().to_string())
        .filter(|entry| !entry.is_empty())
    else {
        return Ok(None);
    };

    let parsed = Url::parse(&webhook_url)
        .map_err(|_| DisplayError::new("discord webhook url is invalid"))?;

    let host = parsed.host_str().unwrap_or_default().to_ascii_lowercase();
    let is_supported_host = matches!(
        host.as_str(),
        "discord.com"
            | "canary.discord.com"
            | "ptb.discord.com"
            | "discordapp.com"
            | "canary.discordapp.com"
            | "ptb.discordapp.com"
    );

    if !is_supported_host || !parsed.path().starts_with("/api/webhooks/") {
        return Err(
            DisplayError::new("discord webhook url must be a Discord incoming webhook").into(),
        );
    }

    Ok(Some(webhook_url))
}

fn humanize_ticket_value(value: &str) -> String {
    value
        .split('_')
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

async fn get_or_create_settings_row(state: &State) -> Result<TicketSettingsRow, anyhow::Error> {
    if let Some(row) = sqlx::query_as::<_, TicketSettingsRow>(
        r#"
        SELECT
            uuid,
            categories_enabled,
            allow_client_close,
            allow_reply_on_closed,
            create_ticket_rate_limit_hits,
            create_ticket_rate_limit_window_seconds,
            max_open_tickets_per_user,
            discord_webhook_enabled,
            discord_webhook_url,
            discord_notify_on_ticket_created,
            discord_notify_on_client_reply,
            discord_notify_on_staff_reply,
            discord_notify_on_internal_note,
            discord_notify_on_status_change,
            discord_notify_on_assignment_change,
            discord_notify_on_ticket_deleted,
            created,
            updated
        FROM ext_support_ticket_settings
        ORDER BY created
        LIMIT 1
        "#,
    )
    .fetch_optional(state.database.read())
    .await?
    {
        return Ok(row);
    }

    Ok(sqlx::query_as::<_, TicketSettingsRow>(
        r#"
        INSERT INTO ext_support_ticket_settings (
            categories_enabled,
            allow_client_close,
            allow_reply_on_closed,
            create_ticket_rate_limit_hits,
            create_ticket_rate_limit_window_seconds,
            max_open_tickets_per_user,
            discord_webhook_enabled,
            discord_notify_on_ticket_created,
            discord_notify_on_client_reply,
            discord_notify_on_staff_reply,
            discord_notify_on_internal_note,
            discord_notify_on_status_change,
            discord_notify_on_assignment_change,
            discord_notify_on_ticket_deleted
        )
        VALUES (TRUE, TRUE, FALSE, 20, 300, 0, FALSE, TRUE, TRUE, TRUE, FALSE, TRUE, TRUE, FALSE)
        RETURNING
            uuid,
            categories_enabled,
            allow_client_close,
            allow_reply_on_closed,
            create_ticket_rate_limit_hits,
            create_ticket_rate_limit_window_seconds,
            max_open_tickets_per_user,
            discord_webhook_enabled,
            discord_webhook_url,
            discord_notify_on_ticket_created,
            discord_notify_on_client_reply,
            discord_notify_on_staff_reply,
            discord_notify_on_internal_note,
            discord_notify_on_status_change,
            discord_notify_on_assignment_change,
            discord_notify_on_ticket_deleted,
            created,
            updated
        "#,
    )
    .fetch_one(state.database.write())
    .await?)
}

async fn get_category_row(
    state: &State,
    category_uuid: uuid::Uuid,
) -> Result<Option<TicketCategoryRow>, anyhow::Error> {
    Ok(sqlx::query_as::<_, TicketCategoryRow>(
        r#"
        SELECT uuid, name, description, color, sort_order, enabled, created, updated
        FROM ext_support_ticket_categories
        WHERE uuid = $1
        "#,
    )
    .bind(category_uuid)
    .fetch_optional(state.database.read())
    .await?)
}

async fn build_ticket_detail(
    state: &State,
    ticket_uuid: uuid::Uuid,
    include_internal: bool,
    include_audit_events: bool,
    attachment_url_scope: AttachmentUrlScope,
) -> Result<ApiTicketDetail, anyhow::Error> {
    let storage_url_retriever = state.storage.retrieve_urls().await?;

    let row = sqlx::query_as::<_, TicketDetailRow>(&format!(
        r#"{}
        WHERE t.uuid = $1 AND t.deleted_at IS NULL
        "#,
        DETAIL_SELECT
    ))
    .bind(ticket_uuid)
    .fetch_optional(state.database.read())
    .await?
    .ok_or_else(|| {
        DisplayError::new("ticket not found").with_status(axum::http::StatusCode::NOT_FOUND)
    })?;

    let messages = sqlx::query_as::<_, TicketMessageRow>(
        r#"
        SELECT
            messages.uuid,
            messages.author_user_uuid,
            messages.author_snapshot_username,
            messages.author_snapshot_display_name,
            users.avatar AS author_avatar,
            messages.author_type,
            messages.body,
            messages.is_internal,
            messages.created,
            messages.updated
        FROM ext_support_ticket_messages messages
        LEFT JOIN users ON users.uuid = messages.author_user_uuid
        WHERE ticket_uuid = $1 AND ($2 OR is_internal = FALSE)
        ORDER BY messages.created ASC
        "#,
    )
    .bind(ticket_uuid)
    .bind(include_internal)
    .fetch_all(state.database.read())
    .await?;

    let attachment_rows = sqlx::query_as::<_, TicketAttachmentRow>(
        r#"
        SELECT uuid, message_uuid, original_name, content_type, media_type, size, created
        FROM ext_support_ticket_attachments
        WHERE ticket_uuid = $1
        ORDER BY created ASC
        "#,
    )
    .bind(ticket_uuid)
    .fetch_all(state.database.read())
    .await?;

    let mut attachments_by_message = HashMap::<uuid::Uuid, Vec<ApiTicketAttachment>>::new();
    for row in attachment_rows {
        attachments_by_message
            .entry(row.message_uuid)
            .or_default()
            .push(to_ticket_attachment(row, ticket_uuid, attachment_url_scope));
    }

    let messages = messages
        .into_iter()
        .map(|row| {
            let attachments = attachments_by_message.remove(&row.uuid).unwrap_or_default();
            to_ticket_message(row, attachments, &storage_url_retriever)
        })
        .collect();

    let audit_events = if include_audit_events {
        sqlx::query_as::<_, TicketAuditRow>(
            r#"
            SELECT uuid, actor_user_uuid, actor_snapshot_username, actor_type, event, payload, created
            FROM ext_support_ticket_audit_events
            WHERE ticket_uuid = $1
            ORDER BY created ASC
            "#,
        )
        .bind(ticket_uuid)
        .fetch_all(state.database.read())
        .await?
        .into_iter()
        .map(to_ticket_audit)
        .collect()
    } else {
        Vec::new()
    };

    let category = optional_category_summary_detail(&row);
    let assigned_user = optional_assigned_user_detail(&row);
    let linked_server = to_linked_server_summary_from_detail(&row);
    let last_reply_at = row.last_reply_at.map(|value| value.and_utc());
    let last_reply_by_type = row.last_reply_by_type.clone();
    let created = row.created.and_utc();
    let updated = row.updated.and_utc();
    let closed_at = row.closed_at.map(|value| value.and_utc());

    let ticket = ApiTicketSummary {
        uuid: row.uuid,
        subject: row.subject,
        status: row.status,
        priority: row.priority,
        creator: to_user_summary(
            row.creator_uuid,
            row.creator_username,
            row.creator_name_first,
            row.creator_name_last,
            row.creator_admin,
        ),
        category,
        assigned_user,
        linked_server,
        last_reply_at,
        last_reply_by_type,
        created,
        updated,
        closed_at,
    };

    Ok(ApiTicketDetail {
        ticket,
        metadata: row.metadata,
        messages,
        audit_events,
    })
}

async fn create_message_with_attachments(
    state: &State,
    ticket_uuid: uuid::Uuid,
    actor: &TicketActor<'_>,
    author_type: &'static str,
    body: &str,
    is_internal: bool,
    attachments: Vec<IncomingAttachmentUpload>,
) -> Result<uuid::Uuid, anyhow::Error> {
    let mut transaction = state.database.write().begin().await?;
    let mut stored_paths = Vec::new();

    let result = async {
        let message_uuid: uuid::Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO ext_support_ticket_messages (
                ticket_uuid,
                author_user_uuid,
                author_snapshot_username,
                author_snapshot_display_name,
                author_type,
                body,
                is_internal
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING uuid
            "#,
        )
        .bind(ticket_uuid)
        .bind(actor.user_uuid)
        .bind(actor.username)
        .bind(&actor.display_name)
        .bind(author_type)
        .bind(body)
        .bind(is_internal)
        .fetch_one(&mut *transaction)
        .await?;

        for attachment in attachments {
            let attachment_uuid = uuid::Uuid::new_v4();
            let storage_path = build_attachment_storage_path(
                ticket_uuid,
                message_uuid,
                attachment_uuid,
                &attachment.original_name,
            );

            let reader = StreamReader::new(futures_util::stream::iter(vec![Ok::<
                axum::body::Bytes,
                std::io::Error,
            >(
                attachment.data
            )]));

            let stored_size = state
                .storage
                .store(&storage_path, reader, &attachment.content_type)
                .await?;

            stored_paths.push(storage_path.clone());

            sqlx::query(
                r#"
                INSERT INTO ext_support_ticket_attachments (
                    uuid,
                    ticket_uuid,
                    message_uuid,
                    uploader_user_uuid,
                    storage_path,
                    original_name,
                    content_type,
                    media_type,
                    size
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
            )
            .bind(attachment_uuid)
            .bind(ticket_uuid)
            .bind(message_uuid)
            .bind(actor.user_uuid)
            .bind(&storage_path)
            .bind(&attachment.original_name)
            .bind(&attachment.content_type)
            .bind(&attachment.media_type)
            .bind(i64::try_from(stored_size)?)
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;
        Ok::<uuid::Uuid, anyhow::Error>(message_uuid)
    }
    .await;

    match result {
        Ok(message_uuid) => Ok(message_uuid),
        Err(error) => {
            for path in stored_paths {
                let _ = state.storage.remove(Some(path)).await;
            }

            Err(error)
        }
    }
}

async fn insert_audit_event(
    state: &State,
    ticket_uuid: uuid::Uuid,
    actor: Option<&TicketActor<'_>>,
    event: &str,
    payload: serde_json::Value,
) -> Result<(), anyhow::Error> {
    sqlx::query(
        r#"
        INSERT INTO ext_support_ticket_audit_events (
            ticket_uuid,
            actor_user_uuid,
            actor_snapshot_username,
            actor_type,
            event,
            payload
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(ticket_uuid)
    .bind(actor.and_then(|value| value.user_uuid))
    .bind(actor.map(|value| value.username))
    .bind(actor.map_or(TicketActorType::System.as_str(), |value| {
        value.actor_type.as_str()
    }))
    .bind(event)
    .bind(payload)
    .execute(state.database.write())
    .await?;

    Ok(())
}

async fn dispatch_discord_webhook_event(
    state: &State,
    detail: &ApiTicketDetail,
    event: DiscordWebhookEvent,
) {
    match get_or_create_settings_row(state).await {
        Ok(settings_row) => {
            let config = to_discord_webhook_config(&settings_row);
            if let Err(error) = send_ticket_event(state, &config, detail, event).await {
                warn!(
                    ticket_uuid = %detail.ticket.uuid,
                    ?error,
                    "failed to deliver support discord webhook",
                );
            }
        }
        Err(error) => {
            warn!(
                ticket_uuid = %detail.ticket.uuid,
                ?error,
                "failed to load support webhook settings",
            );
        }
    }
}

async fn ensure_staff_candidate(
    state: &State,
    user_uuid: uuid::Uuid,
) -> Result<User, anyhow::Error> {
    let user = User::by_uuid_optional(&state.database, user_uuid)
        .await?
        .ok_or_else(|| DisplayError::new("assigned user not found"))?;

    let has_support_permissions = user.admin
        || user.role.as_ref().is_some_and(|role| {
            role.admin_permissions
                .iter()
                .any(|permission| permission.starts_with("tickets."))
        });

    if !has_support_permissions {
        return Err(DisplayError::new("assigned user must have support staff permissions").into());
    }

    Ok(user)
}

fn server_snapshot_metadata(server: &Server) -> serde_json::Value {
    json!({
        "uuid": server.uuid,
        "uuidShort": server.uuid_short,
        "name": server.name,
        "description": server.description,
        "status": server.status.map(|status| serde_json::Value::String(match status {
            shared::models::server::ServerStatus::Installing => "installing".to_string(),
            shared::models::server::ServerStatus::InstallFailed => "install_failed".to_string(),
            shared::models::server::ServerStatus::RestoringBackup => "restoring_backup".to_string(),
        })).unwrap_or(serde_json::Value::Null),
        "isSuspended": server.suspended,
        "owner": {
            "uuid": server.owner.uuid,
            "username": server.owner.username,
            "email": server.owner.email,
        },
        "node": {
            "uuid": server.node.uuid,
        },
        "egg": {
            "uuid": server.egg.uuid,
            "name": server.egg.name,
            "nest": server.nest.name,
        },
        "allocation": server.allocation.as_ref().map(|allocation| json!({
            "uuid": allocation.uuid,
            "ip": allocation.allocation.ip.ip().to_string(),
            "ipAlias": allocation.allocation.ip_alias,
            "port": allocation.allocation.port,
            "notes": allocation.notes,
        })).unwrap_or(serde_json::Value::Null),
    })
}

pub async fn get_client_bootstrap(
    state: &State,
    user: &User,
) -> Result<ClientTicketBootstrapResponse, anyhow::Error> {
    let settings = to_settings_api(get_or_create_settings_row(state).await?);
    let categories = list_categories(state, true).await?;
    let servers = Server::by_user_uuid_with_pagination(&state.database, user.uuid, 1, 10_000, None)
        .await?
        .data
        .into_iter()
        .map(to_client_server_option)
        .collect();

    Ok(ClientTicketBootstrapResponse {
        settings,
        categories,
        servers,
    })
}

pub async fn get_admin_bootstrap(
    state: &State,
) -> Result<AdminTicketBootstrapResponse, anyhow::Error> {
    Ok(AdminTicketBootstrapResponse {
        settings: to_settings_api(get_or_create_settings_row(state).await?),
        categories: list_categories(state, false).await?,
        staff_users: list_staff_users(state).await?,
    })
}

pub async fn get_admin_settings_detail(
    state: &State,
) -> Result<AdminTicketSettingsDetailResponse, anyhow::Error> {
    let row = get_or_create_settings_row(state).await?;

    Ok(AdminTicketSettingsDetailResponse {
        settings: to_settings_api(row.clone()),
        discord_webhook: to_discord_webhook_settings_api(&row),
    })
}

pub async fn list_categories(
    state: &State,
    enabled_only: bool,
) -> Result<Vec<ApiTicketCategory>, anyhow::Error> {
    let mut query = QueryBuilder::<Postgres>::new(
        "SELECT uuid, name, description, color, sort_order, enabled, created, updated FROM ext_support_ticket_categories",
    );

    if enabled_only {
        query.push(" WHERE enabled = TRUE");
    }

    query.push(" ORDER BY sort_order ASC, name ASC");

    Ok(query
        .build_query_as::<TicketCategoryRow>()
        .fetch_all(state.database.read())
        .await?
        .into_iter()
        .map(to_category_api)
        .collect())
}

pub async fn list_staff_users(state: &State) -> Result<Vec<ApiTicketUserSummary>, anyhow::Error> {
    let rows = sqlx::query_as::<_, StaffUserRow>(
        r#"
        SELECT DISTINCT users.uuid, users.username, users.name_first, users.name_last, users.admin
        FROM users
        LEFT JOIN roles ON roles.uuid = users.role_uuid
        WHERE users.admin = TRUE
            OR EXISTS (
                SELECT 1
                FROM unnest(COALESCE(roles.admin_permissions, ARRAY[]::varchar[])) AS permission
                WHERE permission LIKE 'tickets.%'
            )
        ORDER BY users.admin DESC, users.username ASC
        "#,
    )
    .fetch_all(state.database.read())
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            to_user_summary(
                row.uuid,
                row.username,
                row.name_first,
                row.name_last,
                row.admin,
            )
        })
        .collect())
}

pub async fn upsert_category(
    state: &State,
    uuid: Option<uuid::Uuid>,
    name: &str,
    description: Option<&str>,
    color: Option<&str>,
    sort_order: i32,
    enabled: bool,
) -> Result<ApiTicketCategory, anyhow::Error> {
    validate_category_color(color)?;

    let name = name.trim();
    if name.is_empty() {
        return Err(DisplayError::new("category name cannot be empty").into());
    }
    let description = description.map(str::trim).filter(|value| !value.is_empty());
    let color = color.map(str::trim).filter(|value| !value.is_empty());

    let row = if let Some(uuid) = uuid {
        sqlx::query_as::<_, TicketCategoryRow>(
            r#"
            UPDATE ext_support_ticket_categories
            SET name = $2, description = $3, color = $4, sort_order = $5, enabled = $6, updated = NOW()
            WHERE uuid = $1
            RETURNING uuid, name, description, color, sort_order, enabled, created, updated
            "#,
        )
        .bind(uuid)
        .bind(name)
        .bind(description)
        .bind(color)
        .bind(sort_order)
        .bind(enabled)
        .fetch_optional(state.database.write())
        .await?
        .ok_or_else(|| DisplayError::new("category not found").with_status(axum::http::StatusCode::NOT_FOUND))?
    } else {
        sqlx::query_as::<_, TicketCategoryRow>(
            r#"
            INSERT INTO ext_support_ticket_categories (name, description, color, sort_order, enabled)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING uuid, name, description, color, sort_order, enabled, created, updated
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(color)
        .bind(sort_order)
        .bind(enabled)
        .fetch_one(state.database.write())
        .await?
    };

    Ok(to_category_api(row))
}

pub async fn delete_category(
    state: &State,
    category_uuid: uuid::Uuid,
) -> Result<(), anyhow::Error> {
    let rows_affected = sqlx::query(
        r#"
        DELETE FROM ext_support_ticket_categories
        WHERE uuid = $1
        "#,
    )
    .bind(category_uuid)
    .execute(state.database.write())
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(DisplayError::new("category not found")
            .with_status(axum::http::StatusCode::NOT_FOUND)
            .into());
    }

    Ok(())
}

pub async fn update_settings(
    state: &State,
    categories_enabled: bool,
    allow_client_close: bool,
    allow_reply_on_closed: bool,
    create_ticket_rate_limit_hits: i32,
    create_ticket_rate_limit_window_seconds: i32,
    max_open_tickets_per_user: i32,
    discord_webhook_enabled: bool,
    discord_webhook_url: Option<String>,
    discord_notify_on_ticket_created: bool,
    discord_notify_on_client_reply: bool,
    discord_notify_on_staff_reply: bool,
    discord_notify_on_internal_note: bool,
    discord_notify_on_status_change: bool,
    discord_notify_on_assignment_change: bool,
    discord_notify_on_ticket_deleted: bool,
) -> Result<AdminTicketSettingsDetailResponse, anyhow::Error> {
    let existing = get_or_create_settings_row(state).await?;
    let discord_webhook_url = normalize_discord_webhook_url(discord_webhook_url)?;

    if discord_webhook_enabled && discord_webhook_url.is_none() {
        return Err(DisplayError::new(
            "discord webhook url is required when webhook delivery is enabled",
        )
        .into());
    }

    let row = sqlx::query_as::<_, TicketSettingsRow>(
        r#"
        UPDATE ext_support_ticket_settings
        SET categories_enabled = $2,
            allow_client_close = $3,
            allow_reply_on_closed = $4,
            create_ticket_rate_limit_hits = $5,
            create_ticket_rate_limit_window_seconds = $6,
            max_open_tickets_per_user = $7,
            discord_webhook_enabled = $8,
            discord_webhook_url = $9,
            discord_notify_on_ticket_created = $10,
            discord_notify_on_client_reply = $11,
            discord_notify_on_staff_reply = $12,
            discord_notify_on_internal_note = $13,
            discord_notify_on_status_change = $14,
            discord_notify_on_assignment_change = $15,
            discord_notify_on_ticket_deleted = $16,
            updated = NOW()
        WHERE uuid = $1
        RETURNING
            uuid,
            categories_enabled,
            allow_client_close,
            allow_reply_on_closed,
            create_ticket_rate_limit_hits,
            create_ticket_rate_limit_window_seconds,
            max_open_tickets_per_user,
            discord_webhook_enabled,
            discord_webhook_url,
            discord_notify_on_ticket_created,
            discord_notify_on_client_reply,
            discord_notify_on_staff_reply,
            discord_notify_on_internal_note,
            discord_notify_on_status_change,
            discord_notify_on_assignment_change,
            discord_notify_on_ticket_deleted,
            created,
            updated
        "#,
    )
    .bind(existing.uuid)
    .bind(categories_enabled)
    .bind(allow_client_close)
    .bind(allow_reply_on_closed)
    .bind(create_ticket_rate_limit_hits)
    .bind(create_ticket_rate_limit_window_seconds)
    .bind(max_open_tickets_per_user)
    .bind(discord_webhook_enabled)
    .bind(discord_webhook_url)
    .bind(discord_notify_on_ticket_created)
    .bind(discord_notify_on_client_reply)
    .bind(discord_notify_on_staff_reply)
    .bind(discord_notify_on_internal_note)
    .bind(discord_notify_on_status_change)
    .bind(discord_notify_on_assignment_change)
    .bind(discord_notify_on_ticket_deleted)
    .fetch_one(state.database.write())
    .await?;

    Ok(AdminTicketSettingsDetailResponse {
        settings: to_settings_api(row.clone()),
        discord_webhook: to_discord_webhook_settings_api(&row),
    })
}

pub async fn list_client_tickets(
    state: &State,
    user: &User,
    params: &ClientListTicketsParams,
) -> Result<Pagination<ApiTicketSummary>, anyhow::Error> {
    let mut query = QueryBuilder::<Postgres>::new(SUMMARY_SELECT);
    query.push(" WHERE t.deleted_at IS NULL AND t.creator_user_uuid = ");
    query.push_bind(user.uuid);

    if let Some(search) = params
        .search
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        query.push(" AND (t.subject ILIKE '%' || ");
        query.push_bind(search);
        query.push(" || '%' OR creator.username ILIKE '%' || ");
        query.push_bind(search);
        query.push(" || '%' OR COALESCE(t.linked_server_snapshot_name, '') ILIKE '%' || ");
        query.push_bind(search);
        query.push(" || '%')");
    }

    if let Some(status) = params
        .status
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        query.push(" AND t.status = ");
        query.push_bind(normalize_status(status)?);
    }

    query.push(" ORDER BY COALESCE(t.last_reply_at, t.created) DESC, t.created DESC LIMIT ");
    query.push_bind(params.per_page);
    query.push(" OFFSET ");
    query.push_bind((params.page - 1) * params.per_page);

    let rows = query
        .build_query_as::<TicketSummaryRow>()
        .fetch_all(state.database.read())
        .await?;

    Ok(Pagination {
        total: rows.first().map(|row| row.total_count).unwrap_or(0),
        per_page: params.per_page,
        page: params.page,
        data: rows.into_iter().map(to_ticket_summary).collect(),
    })
}

pub async fn list_admin_tickets(
    state: &State,
    params: &AdminListTicketsParams,
) -> Result<Pagination<ApiTicketSummary>, anyhow::Error> {
    let mut query = QueryBuilder::<Postgres>::new(SUMMARY_SELECT);
    query.push(" WHERE t.deleted_at IS NULL");

    if let Some(search) = params
        .search
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        query.push(" AND (");
        query.push("t.subject ILIKE '%' || ");
        query.push_bind(search);
        query.push(" || '%' OR creator.username ILIKE '%' || ");
        query.push_bind(search);
        query.push(" || '%' OR COALESCE(t.linked_server_snapshot_name, '') ILIKE '%' || ");
        query.push_bind(search);
        query.push(" || '%')");
    }

    if let Some(status) = params
        .status
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        query.push(" AND t.status = ");
        query.push_bind(normalize_status(status)?);
    }

    if let Some(category_uuid) = params.category_uuid {
        query.push(" AND t.category_uuid = ");
        query.push_bind(category_uuid);
    }

    if let Some(assigned_user_uuid) = params.assigned_user_uuid {
        query.push(" AND t.assigned_user_uuid = ");
        query.push_bind(assigned_user_uuid);
    }

    if let Some(client) = params
        .client
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        query.push(" AND (creator.username ILIKE '%' || ");
        query.push_bind(client);
        query.push(" || '%' OR creator.email ILIKE '%' || ");
        query.push_bind(client);
        query.push(" || '%')");
    }

    if let Some(server) = params
        .server
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        query.push(" AND COALESCE(t.linked_server_snapshot_name, '') ILIKE '%' || ");
        query.push_bind(server);
        query.push(" || '%'");
    }

    if let Some(priority) = normalize_priority(params.priority.as_deref())? {
        query.push(" AND t.priority = ");
        query.push_bind(priority);
    }

    query.push(" ORDER BY COALESCE(t.last_reply_at, t.created) DESC, t.created DESC LIMIT ");
    query.push_bind(params.per_page);
    query.push(" OFFSET ");
    query.push_bind((params.page - 1) * params.per_page);

    let rows = query
        .build_query_as::<TicketSummaryRow>()
        .fetch_all(state.database.read())
        .await?;

    Ok(Pagination {
        total: rows.first().map(|row| row.total_count).unwrap_or(0),
        per_page: params.per_page,
        page: params.page,
        data: rows.into_iter().map(to_ticket_summary).collect(),
    })
}

pub async fn create_ticket(
    state: &State,
    user: &User,
    request: ClientCreateTicketRequest,
    attachments: Vec<IncomingAttachmentUpload>,
) -> Result<ApiTicketDetail, anyhow::Error> {
    let settings = get_or_create_settings_row(state).await?;

    if settings.create_ticket_rate_limit_hits > 0 {
        if state
            .database
            .cache
            .ratelimit(
                "support/tickets/create",
                settings.create_ticket_rate_limit_hits as u64,
                settings.create_ticket_rate_limit_window_seconds as u64,
                user.uuid.to_string(),
            )
            .await
            .is_err()
        {
            return Err(DisplayError::new(
                "ticket creation rate limit reached, please try again in a few minutes",
            )
            .with_status(StatusCode::TOO_MANY_REQUESTS)
            .into());
        }
    }

    if settings.max_open_tickets_per_user > 0 {
        let open_ticket_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM ext_support_tickets
            WHERE creator_user_uuid = $1
              AND deleted_at IS NULL
              AND closed_at IS NULL
            "#,
        )
        .bind(user.uuid)
        .fetch_one(state.database.read())
        .await?;

        if open_ticket_count >= i64::from(settings.max_open_tickets_per_user) {
            return Err(DisplayError::new(format!(
                "you can only have {} open support tickets at a time",
                settings.max_open_tickets_per_user
            ))
            .with_status(StatusCode::BAD_REQUEST)
            .into());
        }
    }

    let subject = normalize_subject(&request.subject);
    let message = normalize_body(&request.message);
    if subject.is_empty() {
        return Err(DisplayError::new("ticket subject cannot be empty").into());
    }
    ensure_message_or_attachments(&message, &attachments, "ticket message cannot be empty")?;
    let client_metadata = normalize_metadata(request.metadata)?;

    let category: Option<TicketCategoryRow> = if let Some(category_uuid) = request.category_uuid {
        if !settings.categories_enabled {
            return Err(DisplayError::new("ticket categories are disabled").into());
        }

        let category: TicketCategoryRow = get_category_row(state, category_uuid)
            .await?
            .ok_or_else(|| DisplayError::new("selected category not found"))?;

        if !category.enabled {
            return Err(DisplayError::new("selected category is disabled").into());
        }

        Some(category)
    } else {
        None
    };

    let linked_server: Option<Server> = if let Some(server_uuid) = request.server_uuid {
        let identifier = server_uuid.to_string();
        Some(
            Server::by_user_identifier(&state.database, user, &identifier)
                .await?
                .ok_or_else(|| {
                    DisplayError::new("selected server was not found or is not accessible")
                })?,
        )
    } else {
        None
    };

    let metadata = if let Some(server) = linked_server.as_ref() {
        json!({
            "client": client_metadata,
            "server": server_snapshot_metadata(server),
        })
    } else {
        json!({
            "client": client_metadata,
        })
    };

    let row = sqlx::query(
        r#"
        INSERT INTO ext_support_tickets (
            creator_user_uuid,
            creator_snapshot_username,
            creator_snapshot_email,
            linked_server_uuid,
            linked_server_snapshot_name,
            linked_server_snapshot_uuid_short,
            category_uuid,
            subject,
            status,
            priority,
            metadata,
            last_reply_at,
            last_reply_by_type
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'waiting_on_staff', 'normal', $9, NOW(), 'client')
        RETURNING uuid
        "#,
    )
    .bind(user.uuid)
    .bind(user.username.to_string())
    .bind(user.email.to_string())
    .bind(linked_server.as_ref().map(|server| server.uuid))
    .bind(linked_server.as_ref().map(|server| server.name.to_string()))
    .bind(linked_server.as_ref().map(|server| server.uuid_short))
    .bind(category.as_ref().map(|category| category.uuid))
    .bind(&subject)
    .bind(metadata)
    .fetch_one(state.database.write())
    .await?;

    let ticket_uuid: uuid::Uuid = sqlx::Row::try_get(&row, "uuid")?;

    let actor = TicketActor {
        user_uuid: Some(user.uuid),
        username: user.username.as_str(),
        display_name: format!("{} {}", user.name_first, user.name_last)
            .trim()
            .to_string(),
        actor_type: TicketActorType::Client,
    };

    create_message_with_attachments(
        state,
        ticket_uuid,
        &actor,
        TicketActorType::Client.as_str(),
        &message,
        false,
        attachments,
    )
    .await?;

    insert_audit_event(
        state,
        ticket_uuid,
        Some(&actor),
        "ticket_created",
        json!({
            "subject": subject,
            "categoryUuid": category.as_ref().map(|value| value.uuid),
            "linkedServerUuid": linked_server.as_ref().map(|value| value.uuid),
        }),
    )
    .await?;

    let detail =
        build_ticket_detail(state, ticket_uuid, false, false, AttachmentUrlScope::Client).await?;

    dispatch_discord_webhook_event(
        state,
        &detail,
        DiscordWebhookEvent {
            kind: DiscordWebhookEventKind::TicketCreated,
            actor_display_name: Some(actor.display_name.clone()),
            actor_username: Some(actor.username.to_string()),
            latest_message_html: Some(message),
            extra_lines: Vec::new(),
        },
    )
    .await;

    Ok(detail)
}

pub async fn get_client_ticket_detail(
    state: &State,
    user: &User,
    ticket_uuid: uuid::Uuid,
) -> Result<ApiTicketDetail, anyhow::Error> {
    let owner_uuid: Option<uuid::Uuid> = sqlx::query_scalar(
        r#"
        SELECT creator_user_uuid
        FROM ext_support_tickets
        WHERE uuid = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(ticket_uuid)
    .fetch_optional(state.database.read())
    .await?;

    match owner_uuid {
        Some(owner_uuid) if owner_uuid == user.uuid => {
            build_ticket_detail(state, ticket_uuid, false, false, AttachmentUrlScope::Client).await
        }
        Some(_) => Err(DisplayError::new("ticket not found")
            .with_status(axum::http::StatusCode::NOT_FOUND)
            .into()),
        None => Err(DisplayError::new("ticket not found")
            .with_status(axum::http::StatusCode::NOT_FOUND)
            .into()),
    }
}

pub async fn get_admin_ticket_detail(
    state: &State,
    ticket_uuid: uuid::Uuid,
) -> Result<ApiTicketDetail, anyhow::Error> {
    build_ticket_detail(state, ticket_uuid, true, true, AttachmentUrlScope::Admin).await
}

pub async fn get_client_attachment_download(
    state: &State,
    user: &User,
    ticket_uuid: uuid::Uuid,
    attachment_uuid: uuid::Uuid,
) -> Result<TicketAttachmentDownload, anyhow::Error> {
    let row = sqlx::query_as::<_, TicketAttachmentDownloadRow>(
        r#"
        SELECT a.original_name, a.content_type, a.size, a.storage_path
        FROM ext_support_ticket_attachments a
        JOIN ext_support_ticket_messages m ON m.uuid = a.message_uuid
        JOIN ext_support_tickets t ON t.uuid = a.ticket_uuid
        WHERE a.uuid = $1
          AND a.ticket_uuid = $2
          AND t.creator_user_uuid = $3
          AND t.deleted_at IS NULL
          AND m.is_internal = FALSE
        "#,
    )
    .bind(attachment_uuid)
    .bind(ticket_uuid)
    .bind(user.uuid)
    .fetch_optional(state.database.read())
    .await?
    .ok_or_else(|| {
        DisplayError::new("attachment not found").with_status(axum::http::StatusCode::NOT_FOUND)
    })?;

    let bytes = load_attachment_bytes_from_storage(state, &row.storage_path).await?;

    Ok(TicketAttachmentDownload {
        original_name: row.original_name,
        content_type: row.content_type,
        size: row.size,
        bytes,
    })
}

pub async fn get_admin_attachment_download(
    state: &State,
    ticket_uuid: uuid::Uuid,
    attachment_uuid: uuid::Uuid,
) -> Result<TicketAttachmentDownload, anyhow::Error> {
    let row = sqlx::query_as::<_, TicketAttachmentDownloadRow>(
        r#"
        SELECT a.original_name, a.content_type, a.size, a.storage_path
        FROM ext_support_ticket_attachments a
        JOIN ext_support_tickets t ON t.uuid = a.ticket_uuid
        WHERE a.uuid = $1
          AND a.ticket_uuid = $2
          AND t.deleted_at IS NULL
        "#,
    )
    .bind(attachment_uuid)
    .bind(ticket_uuid)
    .fetch_optional(state.database.read())
    .await?
    .ok_or_else(|| {
        DisplayError::new("attachment not found").with_status(axum::http::StatusCode::NOT_FOUND)
    })?;

    let bytes = load_attachment_bytes_from_storage(state, &row.storage_path).await?;

    Ok(TicketAttachmentDownload {
        original_name: row.original_name,
        content_type: row.content_type,
        size: row.size,
        bytes,
    })
}

pub async fn add_client_reply(
    state: &State,
    user: &User,
    ticket_uuid: uuid::Uuid,
    body: &str,
    attachments: Vec<IncomingAttachmentUpload>,
) -> Result<ApiTicketDetail, anyhow::Error> {
    let settings = get_or_create_settings_row(state).await?;
    let detail = get_client_ticket_detail(state, user, ticket_uuid).await?;

    if detail.ticket.status == TicketStatus::Closed.as_str() && !settings.allow_reply_on_closed {
        return Err(DisplayError::new("ticket is closed and cannot receive replies").into());
    }

    let message = normalize_body(body);
    ensure_message_or_attachments(&message, &attachments, "reply cannot be empty")?;
    let actor = TicketActor {
        user_uuid: Some(user.uuid),
        username: user.username.as_str(),
        display_name: format!("{} {}", user.name_first, user.name_last)
            .trim()
            .to_string(),
        actor_type: TicketActorType::Client,
    };

    create_message_with_attachments(
        state,
        ticket_uuid,
        &actor,
        TicketActorType::Client.as_str(),
        &message,
        false,
        attachments,
    )
    .await?;

    sqlx::query(
        r#"
        UPDATE ext_support_tickets
        SET status = 'waiting_on_staff',
            last_reply_at = NOW(),
            last_reply_by_type = 'client',
            closed_at = NULL,
            closed_by_user_uuid = NULL,
            updated = NOW()
        WHERE uuid = $1
        "#,
    )
    .bind(ticket_uuid)
    .execute(state.database.write())
    .await?;

    insert_audit_event(
        state,
        ticket_uuid,
        Some(&actor),
        if detail.ticket.status == TicketStatus::Closed.as_str() {
            "ticket_reopened_by_reply"
        } else {
            "reply_added"
        },
        json!({
            "isInternal": false,
        }),
    )
    .await?;

    let detail =
        build_ticket_detail(state, ticket_uuid, false, false, AttachmentUrlScope::Client).await?;

    dispatch_discord_webhook_event(
        state,
        &detail,
        DiscordWebhookEvent {
            kind: DiscordWebhookEventKind::ClientReply,
            actor_display_name: Some(actor.display_name.clone()),
            actor_username: Some(actor.username.to_string()),
            latest_message_html: Some(message),
            extra_lines: Vec::new(),
        },
    )
    .await;

    Ok(detail)
}

pub async fn update_client_ticket_status(
    state: &State,
    user: &User,
    ticket_uuid: uuid::Uuid,
    status: &str,
) -> Result<ApiTicketDetail, anyhow::Error> {
    let settings = get_or_create_settings_row(state).await?;
    let detail = get_client_ticket_detail(state, user, ticket_uuid).await?;
    let normalized = normalize_status(status)?;

    let actor = TicketActor {
        user_uuid: Some(user.uuid),
        username: user.username.as_str(),
        display_name: format!("{} {}", user.name_first, user.name_last)
            .trim()
            .to_string(),
        actor_type: TicketActorType::Client,
    };

    match normalized {
        "closed" => {
            if !settings.allow_client_close {
                return Err(DisplayError::new("clients cannot close tickets right now").into());
            }

            sqlx::query(
                r#"
                UPDATE ext_support_tickets
                SET status = 'closed',
                    closed_at = NOW(),
                    closed_by_user_uuid = $2,
                    updated = NOW()
                WHERE uuid = $1
                "#,
            )
            .bind(ticket_uuid)
            .bind(user.uuid)
            .execute(state.database.write())
            .await?;

            insert_audit_event(state, ticket_uuid, Some(&actor), "ticket_closed", json!({}))
                .await?;
        }
        "open" | "waiting_on_staff" => {
            if detail.ticket.status != TicketStatus::Closed.as_str() {
                return Err(DisplayError::new("only closed tickets can be reopened").into());
            }

            sqlx::query(
                r#"
                UPDATE ext_support_tickets
                SET status = 'waiting_on_staff',
                    closed_at = NULL,
                    closed_by_user_uuid = NULL,
                    updated = NOW()
                WHERE uuid = $1
                "#,
            )
            .bind(ticket_uuid)
            .execute(state.database.write())
            .await?;

            insert_audit_event(
                state,
                ticket_uuid,
                Some(&actor),
                "ticket_reopened",
                json!({}),
            )
            .await?;
        }
        _ => return Err(DisplayError::new("clients may only close or reopen tickets").into()),
    }

    let next_detail =
        build_ticket_detail(state, ticket_uuid, false, false, AttachmentUrlScope::Client).await?;

    dispatch_discord_webhook_event(
        state,
        &next_detail,
        DiscordWebhookEvent {
            kind: DiscordWebhookEventKind::StatusChanged,
            actor_display_name: Some(actor.display_name.clone()),
            actor_username: Some(actor.username.to_string()),
            latest_message_html: None,
            extra_lines: vec![
                format!(
                    "**Previous Status:** {}",
                    detail.ticket.status.replace('_', " ")
                ),
                format!(
                    "**New Status:** {}",
                    next_detail.ticket.status.replace('_', " ")
                ),
            ],
        },
    )
    .await;

    Ok(next_detail)
}

pub async fn add_admin_message(
    state: &State,
    user: &User,
    ticket_uuid: uuid::Uuid,
    body: &str,
    is_internal: bool,
    attachments: Vec<IncomingAttachmentUpload>,
) -> Result<ApiTicketDetail, anyhow::Error> {
    let previous_detail = get_admin_ticket_detail(state, ticket_uuid).await?;
    let message = normalize_body(body);
    ensure_message_or_attachments(&message, &attachments, "message cannot be empty")?;
    let actor = TicketActor {
        user_uuid: Some(user.uuid),
        username: user.username.as_str(),
        display_name: format!("{} {}", user.name_first, user.name_last)
            .trim()
            .to_string(),
        actor_type: TicketActorType::Staff,
    };

    create_message_with_attachments(
        state,
        ticket_uuid,
        &actor,
        TicketActorType::Staff.as_str(),
        &message,
        is_internal,
        attachments,
    )
    .await?;

    if is_internal {
        sqlx::query(
            r#"
            UPDATE ext_support_tickets
            SET updated = NOW()
            WHERE uuid = $1
            "#,
        )
        .bind(ticket_uuid)
        .execute(state.database.write())
        .await?;

        insert_audit_event(
            state,
            ticket_uuid,
            Some(&actor),
            "internal_note_added",
            json!({}),
        )
        .await?;
    } else {
        sqlx::query(
            r#"
            UPDATE ext_support_tickets
            SET status = 'waiting_on_client',
                last_reply_at = NOW(),
                last_reply_by_type = 'staff',
                closed_at = NULL,
                closed_by_user_uuid = NULL,
                updated = NOW()
            WHERE uuid = $1
            "#,
        )
        .bind(ticket_uuid)
        .execute(state.database.write())
        .await?;

        insert_audit_event(
            state,
            ticket_uuid,
            Some(&actor),
            "reply_added",
            json!({
                "isInternal": false,
            }),
        )
        .await?;
    }

    let detail =
        build_ticket_detail(state, ticket_uuid, true, true, AttachmentUrlScope::Admin).await?;

    dispatch_discord_webhook_event(
        state,
        &detail,
        DiscordWebhookEvent {
            kind: if is_internal {
                DiscordWebhookEventKind::InternalNote
            } else {
                DiscordWebhookEventKind::StaffReply
            },
            actor_display_name: Some(actor.display_name.clone()),
            actor_username: Some(actor.username.to_string()),
            latest_message_html: Some(message),
            extra_lines: if previous_detail.ticket.status != detail.ticket.status {
                vec![
                    format!(
                        "**Previous Status:** {}",
                        humanize_ticket_value(&previous_detail.ticket.status)
                    ),
                    format!(
                        "**New Status:** {}",
                        humanize_ticket_value(&detail.ticket.status)
                    ),
                ]
            } else {
                Vec::new()
            },
        },
    )
    .await;

    Ok(detail)
}

pub async fn update_admin_ticket_status(
    state: &State,
    user: &User,
    ticket_uuid: uuid::Uuid,
    status: &str,
) -> Result<ApiTicketDetail, anyhow::Error> {
    let previous_detail = get_admin_ticket_detail(state, ticket_uuid).await?;
    let normalized = normalize_status(status)?;
    let actor = TicketActor {
        user_uuid: Some(user.uuid),
        username: user.username.as_str(),
        display_name: format!("{} {}", user.name_first, user.name_last)
            .trim()
            .to_string(),
        actor_type: TicketActorType::Staff,
    };

    let (closed_at, closed_by_user_uuid) = if normalized == TicketStatus::Closed.as_str() {
        (Some(Utc::now().naive_utc()), Some(user.uuid))
    } else {
        (None, None)
    };

    sqlx::query(
        r#"
        UPDATE ext_support_tickets
        SET status = $2,
            closed_at = $3,
            closed_by_user_uuid = $4,
            updated = NOW()
        WHERE uuid = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(ticket_uuid)
    .bind(normalized)
    .bind(closed_at)
    .bind(closed_by_user_uuid)
    .execute(state.database.write())
    .await?;

    insert_audit_event(
        state,
        ticket_uuid,
        Some(&actor),
        "status_changed",
        json!({ "status": normalized }),
    )
    .await?;

    let detail =
        build_ticket_detail(state, ticket_uuid, true, true, AttachmentUrlScope::Admin).await?;

    dispatch_discord_webhook_event(
        state,
        &detail,
        DiscordWebhookEvent {
            kind: DiscordWebhookEventKind::StatusChanged,
            actor_display_name: Some(actor.display_name.clone()),
            actor_username: Some(actor.username.to_string()),
            latest_message_html: None,
            extra_lines: vec![
                format!(
                    "**Previous Status:** {}",
                    humanize_ticket_value(&previous_detail.ticket.status)
                ),
                format!(
                    "**New Status:** {}",
                    humanize_ticket_value(&detail.ticket.status)
                ),
            ],
        },
    )
    .await;

    Ok(detail)
}

pub async fn assign_ticket(
    state: &State,
    user: &User,
    ticket_uuid: uuid::Uuid,
    assigned_user_uuid: Option<uuid::Uuid>,
) -> Result<ApiTicketDetail, anyhow::Error> {
    let previous_detail = get_admin_ticket_detail(state, ticket_uuid).await?;
    let assignee = if let Some(assigned_user_uuid) = assigned_user_uuid {
        Some(ensure_staff_candidate(state, assigned_user_uuid).await?)
    } else {
        None
    };

    let actor = TicketActor {
        user_uuid: Some(user.uuid),
        username: user.username.as_str(),
        display_name: format!("{} {}", user.name_first, user.name_last)
            .trim()
            .to_string(),
        actor_type: TicketActorType::Staff,
    };

    sqlx::query(
        r#"
        UPDATE ext_support_tickets
        SET assigned_user_uuid = $2,
            updated = NOW()
        WHERE uuid = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(ticket_uuid)
    .bind(assignee.as_ref().map(|value| value.uuid))
    .execute(state.database.write())
    .await?;

    insert_audit_event(
        state,
        ticket_uuid,
        Some(&actor),
        "assignee_changed",
        json!({
            "assignedUserUuid": assignee.as_ref().map(|value| value.uuid),
            "assignedUsername": assignee.as_ref().map(|value| value.username.to_string()),
        }),
    )
    .await?;

    let detail =
        build_ticket_detail(state, ticket_uuid, true, true, AttachmentUrlScope::Admin).await?;

    dispatch_discord_webhook_event(
        state,
        &detail,
        DiscordWebhookEvent {
            kind: DiscordWebhookEventKind::AssignmentChanged,
            actor_display_name: Some(actor.display_name.clone()),
            actor_username: Some(actor.username.to_string()),
            latest_message_html: None,
            extra_lines: vec![
                format!(
                    "**Previous Assignee:** {}",
                    previous_detail
                        .ticket
                        .assigned_user
                        .as_ref()
                        .map(|value| value.username.as_str())
                        .unwrap_or("Unassigned")
                ),
                format!(
                    "**New Assignee:** {}",
                    detail
                        .ticket
                        .assigned_user
                        .as_ref()
                        .map(|value| value.username.as_str())
                        .unwrap_or("Unassigned")
                ),
            ],
        },
    )
    .await;

    Ok(detail)
}

pub async fn update_ticket_priority(
    state: &State,
    user: &User,
    ticket_uuid: uuid::Uuid,
    priority: Option<&str>,
) -> Result<ApiTicketDetail, anyhow::Error> {
    let _ = get_admin_ticket_detail(state, ticket_uuid).await?;
    let normalized = normalize_priority(priority)?;
    let actor = TicketActor {
        user_uuid: Some(user.uuid),
        username: user.username.as_str(),
        display_name: format!("{} {}", user.name_first, user.name_last)
            .trim()
            .to_string(),
        actor_type: TicketActorType::Staff,
    };

    sqlx::query(
        r#"
        UPDATE ext_support_tickets
        SET priority = $2,
            updated = NOW()
        WHERE uuid = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(ticket_uuid)
    .bind(normalized)
    .execute(state.database.write())
    .await?;

    insert_audit_event(
        state,
        ticket_uuid,
        Some(&actor),
        "priority_changed",
        json!({ "priority": normalized }),
    )
    .await?;

    build_ticket_detail(state, ticket_uuid, true, true, AttachmentUrlScope::Admin).await
}

pub async fn update_ticket_category(
    state: &State,
    user: &User,
    ticket_uuid: uuid::Uuid,
    category_uuid: Option<uuid::Uuid>,
) -> Result<ApiTicketDetail, anyhow::Error> {
    let _ = get_admin_ticket_detail(state, ticket_uuid).await?;
    let category: Option<TicketCategoryRow> = if let Some(category_uuid) = category_uuid {
        Some(
            get_category_row(state, category_uuid)
                .await?
                .ok_or_else(|| DisplayError::new("selected category not found"))?,
        )
    } else {
        None
    };

    let actor = TicketActor {
        user_uuid: Some(user.uuid),
        username: user.username.as_str(),
        display_name: format!("{} {}", user.name_first, user.name_last)
            .trim()
            .to_string(),
        actor_type: TicketActorType::Staff,
    };

    sqlx::query(
        r#"
        UPDATE ext_support_tickets
        SET category_uuid = $2,
            updated = NOW()
        WHERE uuid = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(ticket_uuid)
    .bind(category.as_ref().map(|value| value.uuid))
    .execute(state.database.write())
    .await?;

    insert_audit_event(
        state,
        ticket_uuid,
        Some(&actor),
        "category_changed",
        json!({
            "categoryUuid": category.as_ref().map(|value| value.uuid),
            "categoryName": category.as_ref().map(|value| value.name.clone()),
        }),
    )
    .await?;

    build_ticket_detail(state, ticket_uuid, true, true, AttachmentUrlScope::Admin).await
}

pub async fn soft_delete_ticket(
    state: &State,
    user: &User,
    ticket_uuid: uuid::Uuid,
) -> Result<(), anyhow::Error> {
    let detail = get_admin_ticket_detail(state, ticket_uuid).await?;
    let actor = TicketActor {
        user_uuid: Some(user.uuid),
        username: user.username.as_str(),
        display_name: format!("{} {}", user.name_first, user.name_last)
            .trim()
            .to_string(),
        actor_type: TicketActorType::Staff,
    };

    let rows_affected = sqlx::query(
        r#"
        UPDATE ext_support_tickets
        SET deleted_at = NOW(),
            status = 'closed',
            closed_at = COALESCE(closed_at, NOW()),
            closed_by_user_uuid = COALESCE(closed_by_user_uuid, $2),
            updated = NOW()
        WHERE uuid = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(ticket_uuid)
    .bind(user.uuid)
    .execute(state.database.write())
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(DisplayError::new("ticket not found")
            .with_status(axum::http::StatusCode::NOT_FOUND)
            .into());
    }

    insert_audit_event(
        state,
        ticket_uuid,
        Some(&actor),
        "ticket_deleted",
        json!({}),
    )
    .await?;

    dispatch_discord_webhook_event(
        state,
        &detail,
        DiscordWebhookEvent {
            kind: DiscordWebhookEventKind::TicketDeleted,
            actor_display_name: Some(actor.display_name.clone()),
            actor_username: Some(actor.username.to_string()),
            latest_message_html: None,
            extra_lines: Vec::new(),
        },
    )
    .await;

    Ok(())
}

pub async fn mark_linked_server_deleted(
    state: &State,
    server_uuid: uuid::Uuid,
) -> Result<(), anyhow::Error> {
    sqlx::query(
        r#"
        WITH updated AS (
            UPDATE ext_support_tickets
            SET linked_server_deleted_at = NOW(),
                updated = NOW()
            WHERE linked_server_uuid = $1 AND deleted_at IS NULL AND linked_server_deleted_at IS NULL
            RETURNING uuid, linked_server_uuid
        )
        INSERT INTO ext_support_ticket_audit_events (
            uuid,
            ticket_uuid,
            actor_user_uuid,
            actor_snapshot_username,
            actor_type,
            event,
            payload,
            created
        )
        SELECT
            gen_random_uuid(),
            updated.uuid,
            NULL,
            NULL,
            'system',
            'linked_server_deleted',
            jsonb_build_object('linkedServerUuid', updated.linked_server_uuid),
            NOW()
        FROM updated
        "#,
    )
    .bind(server_uuid)
    .execute(state.database.write())
    .await?;

    Ok(())
}
