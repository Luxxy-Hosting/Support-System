use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(ToSchema, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TicketStatus {
    Open,
    Pending,
    Answered,
    WaitingOnClient,
    WaitingOnStaff,
    Closed,
}

impl TicketStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Pending => "pending",
            Self::Answered => "answered",
            Self::WaitingOnClient => "waiting_on_client",
            Self::WaitingOnStaff => "waiting_on_staff",
            Self::Closed => "closed",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "open" => Some(Self::Open),
            "pending" => Some(Self::Pending),
            "answered" => Some(Self::Answered),
            "waiting_on_client" => Some(Self::WaitingOnClient),
            "waiting_on_staff" => Some(Self::WaitingOnStaff),
            "closed" => Some(Self::Closed),
            _ => None,
        }
    }
}

#[derive(ToSchema, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TicketPriority {
    Low,
    Normal,
    High,
    Urgent,
}

impl TicketPriority {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Normal => "normal",
            Self::High => "high",
            Self::Urgent => "urgent",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "low" => Some(Self::Low),
            "normal" => Some(Self::Normal),
            "high" => Some(Self::High),
            "urgent" => Some(Self::Urgent),
            _ => None,
        }
    }
}

#[derive(ToSchema, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TicketActorType {
    Client,
    Staff,
    System,
}

impl TicketActorType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Client => "client",
            Self::Staff => "staff",
            Self::System => "system",
        }
    }
}

#[derive(ToSchema, Serialize, Deserialize, Clone)]
pub struct ApiTicketSettings {
    pub uuid: uuid::Uuid,
    pub categories_enabled: bool,
    pub allow_client_close: bool,
    pub allow_reply_on_closed: bool,
    pub create_ticket_rate_limit_hits: i32,
    pub create_ticket_rate_limit_window_seconds: i32,
    pub max_open_tickets_per_user: i32,
    pub created: chrono::DateTime<chrono::Utc>,
    pub updated: chrono::DateTime<chrono::Utc>,
}

#[derive(ToSchema, Serialize, Deserialize, Clone)]
pub struct ApiDiscordWebhookSettings {
    pub enabled: bool,
    pub webhook_url: Option<String>,
    pub notify_on_ticket_created: bool,
    pub notify_on_client_reply: bool,
    pub notify_on_staff_reply: bool,
    pub notify_on_internal_note: bool,
    pub notify_on_status_change: bool,
    pub notify_on_assignment_change: bool,
    pub notify_on_ticket_deleted: bool,
}

#[derive(ToSchema, Serialize, Deserialize, Clone)]
pub struct ApiTicketCategory {
    pub uuid: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub sort_order: i32,
    pub enabled: bool,
    pub created: chrono::DateTime<chrono::Utc>,
    pub updated: chrono::DateTime<chrono::Utc>,
}

#[derive(ToSchema, Serialize, Deserialize, Clone)]
pub struct ApiTicketCategorySummary {
    pub uuid: uuid::Uuid,
    pub name: String,
    pub color: Option<String>,
}

#[derive(ToSchema, Serialize, Deserialize, Clone)]
pub struct ApiTicketUserSummary {
    pub uuid: uuid::Uuid,
    pub username: String,
    pub name_first: String,
    pub name_last: String,
    pub admin: bool,
}

#[derive(ToSchema, Serialize, Deserialize, Clone)]
pub struct ApiTicketLinkedServer {
    pub uuid: Option<uuid::Uuid>,
    pub snapshot_name: Option<String>,
    pub snapshot_uuid_short: Option<i32>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub current_name: Option<String>,
    pub current_uuid_short: Option<i32>,
    pub current_status: Option<String>,
    pub current_is_suspended: Option<bool>,
    pub current_owner_username: Option<String>,
}

#[derive(ToSchema, Serialize, Deserialize, Clone)]
pub struct ApiTicketServerOption {
    pub uuid: uuid::Uuid,
    pub uuid_short: i32,
    pub name: String,
    pub owner_username: String,
    pub nest_name: String,
    pub egg_name: String,
    pub is_suspended: bool,
    pub status: Option<String>,
}

#[derive(ToSchema, Serialize, Deserialize, Clone)]
pub struct ApiTicketAttachment {
    pub uuid: uuid::Uuid,
    pub original_name: String,
    pub content_type: String,
    pub media_type: String,
    pub size: i64,
    pub url: String,
    pub created: chrono::DateTime<chrono::Utc>,
}

#[derive(ToSchema, Serialize, Deserialize, Clone)]
pub struct ApiTicketMessage {
    pub uuid: uuid::Uuid,
    pub author_user_uuid: Option<uuid::Uuid>,
    pub author_username: String,
    pub author_display_name: String,
    pub author_avatar: Option<String>,
    pub author_type: String,
    pub body: String,
    pub is_internal: bool,
    pub attachments: Vec<ApiTicketAttachment>,
    pub created: chrono::DateTime<chrono::Utc>,
    pub updated: chrono::DateTime<chrono::Utc>,
}

#[derive(ToSchema, Serialize, Deserialize, Clone)]
pub struct ApiTicketAuditEvent {
    pub uuid: uuid::Uuid,
    pub actor_user_uuid: Option<uuid::Uuid>,
    pub actor_username: Option<String>,
    pub actor_type: String,
    pub event: String,
    pub payload: serde_json::Value,
    pub created: chrono::DateTime<chrono::Utc>,
}

#[derive(ToSchema, Serialize, Deserialize, Clone)]
pub struct ApiTicketSummary {
    pub uuid: uuid::Uuid,
    pub subject: String,
    pub status: String,
    pub priority: Option<String>,
    pub creator: ApiTicketUserSummary,
    pub category: Option<ApiTicketCategorySummary>,
    pub assigned_user: Option<ApiTicketUserSummary>,
    pub linked_server: ApiTicketLinkedServer,
    pub last_reply_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_reply_by_type: Option<String>,
    pub created: chrono::DateTime<chrono::Utc>,
    pub updated: chrono::DateTime<chrono::Utc>,
    pub closed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(ToSchema, Serialize, Deserialize, Clone)]
pub struct ApiTicketDetail {
    pub ticket: ApiTicketSummary,
    pub metadata: serde_json::Value,
    pub messages: Vec<ApiTicketMessage>,
    pub audit_events: Vec<ApiTicketAuditEvent>,
}

#[derive(ToSchema, Serialize, Deserialize, Clone)]
pub struct ClientTicketBootstrapResponse {
    pub settings: ApiTicketSettings,
    pub categories: Vec<ApiTicketCategory>,
    pub servers: Vec<ApiTicketServerOption>,
}

#[derive(ToSchema, Serialize, Deserialize, Clone)]
pub struct AdminTicketBootstrapResponse {
    pub settings: ApiTicketSettings,
    pub categories: Vec<ApiTicketCategory>,
    pub staff_users: Vec<ApiTicketUserSummary>,
}

#[derive(ToSchema, Serialize, Deserialize, Clone)]
pub struct AdminTicketSettingsDetailResponse {
    pub settings: ApiTicketSettings,
    pub discord_webhook: ApiDiscordWebhookSettings,
}

#[derive(ToSchema, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ClientCreateTicketRequest {
    #[garde(skip)]
    #[serde(default)]
    pub server_uuid: Option<uuid::Uuid>,
    #[garde(skip)]
    #[serde(default)]
    pub category_uuid: Option<uuid::Uuid>,
    #[garde(length(chars, min = 3, max = 255))]
    pub subject: String,
    #[garde(length(chars, min = 5, max = 20000))]
    pub message: String,
    #[garde(skip)]
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(ToSchema, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ClientReplyTicketRequest {
    #[garde(length(chars, min = 1, max = 20000))]
    pub body: String,
}

#[derive(ToSchema, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ClientUpdateTicketStatusRequest {
    #[garde(length(chars, min = 1, max = 32))]
    pub status: String,
}

#[derive(ToSchema, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AdminTicketMessageRequest {
    #[garde(length(chars, min = 1, max = 20000))]
    pub body: String,
    #[garde(skip)]
    #[serde(default)]
    pub is_internal: bool,
}

#[derive(ToSchema, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AdminUpdateTicketStatusRequest {
    #[garde(length(chars, min = 1, max = 32))]
    pub status: String,
}

#[derive(ToSchema, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AdminAssignTicketRequest {
    #[garde(skip)]
    #[serde(default)]
    pub assigned_user_uuid: Option<uuid::Uuid>,
}

#[derive(ToSchema, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AdminUpdateTicketPriorityRequest {
    #[garde(skip)]
    #[serde(default)]
    pub priority: Option<String>,
}

#[derive(ToSchema, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AdminUpdateTicketCategoryRequest {
    #[garde(skip)]
    #[serde(default)]
    pub category_uuid: Option<uuid::Uuid>,
}

#[derive(ToSchema, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AdminUpdateTicketSettingsRequest {
    #[garde(skip)]
    pub categories_enabled: bool,
    #[garde(skip)]
    pub allow_client_close: bool,
    #[garde(skip)]
    pub allow_reply_on_closed: bool,
    #[garde(range(min = 0, max = 10_000))]
    pub create_ticket_rate_limit_hits: i32,
    #[garde(range(min = 1, max = 86_400))]
    pub create_ticket_rate_limit_window_seconds: i32,
    #[garde(range(min = 0, max = 1_000))]
    pub max_open_tickets_per_user: i32,
    #[garde(skip)]
    pub discord_webhook_enabled: bool,
    #[garde(length(chars, max = 2048))]
    #[serde(default)]
    pub discord_webhook_url: Option<String>,
    #[garde(skip)]
    pub discord_notify_on_ticket_created: bool,
    #[garde(skip)]
    pub discord_notify_on_client_reply: bool,
    #[garde(skip)]
    pub discord_notify_on_staff_reply: bool,
    #[garde(skip)]
    pub discord_notify_on_internal_note: bool,
    #[garde(skip)]
    pub discord_notify_on_status_change: bool,
    #[garde(skip)]
    pub discord_notify_on_assignment_change: bool,
    #[garde(skip)]
    pub discord_notify_on_ticket_deleted: bool,
}

#[derive(ToSchema, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AdminUpsertTicketCategoryRequest {
    #[garde(skip)]
    #[serde(default)]
    pub uuid: Option<uuid::Uuid>,
    #[garde(length(chars, min = 2, max = 255))]
    pub name: String,
    #[garde(length(chars, min = 2, max = 2048))]
    #[serde(default)]
    pub description: Option<String>,
    #[garde(length(chars, min = 3, max = 32))]
    #[serde(default)]
    pub color: Option<String>,
    #[garde(range(min = -1000, max = 1000))]
    pub sort_order: i32,
    #[garde(skip)]
    #[serde(default)]
    pub enabled: bool,
}

#[derive(ToSchema, Serialize, Deserialize, Validate)]
pub struct ClientListTicketsParams {
    #[garde(range(min = 1))]
    #[serde(default = "shared::models::Pagination::default_page")]
    pub page: i64,
    #[garde(range(min = 1, max = 100))]
    #[serde(default = "shared::models::Pagination::default_per_page")]
    pub per_page: i64,
    #[garde(length(chars, min = 1, max = 128))]
    #[serde(default)]
    pub search: Option<String>,
    #[garde(length(chars, min = 1, max = 32))]
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(ToSchema, Serialize, Deserialize, Validate)]
pub struct AdminListTicketsParams {
    #[garde(range(min = 1))]
    #[serde(default = "shared::models::Pagination::default_page")]
    pub page: i64,
    #[garde(range(min = 1, max = 100))]
    #[serde(default = "shared::models::Pagination::default_per_page")]
    pub per_page: i64,
    #[garde(length(chars, min = 1, max = 128))]
    #[serde(default)]
    pub search: Option<String>,
    #[garde(length(chars, min = 1, max = 32))]
    #[serde(default)]
    pub status: Option<String>,
    #[garde(skip)]
    #[serde(default)]
    pub category_uuid: Option<uuid::Uuid>,
    #[garde(skip)]
    #[serde(default)]
    pub assigned_user_uuid: Option<uuid::Uuid>,
    #[garde(length(chars, min = 1, max = 128))]
    #[serde(default)]
    pub client: Option<String>,
    #[garde(length(chars, min = 1, max = 128))]
    #[serde(default)]
    pub server: Option<String>,
    #[garde(length(chars, min = 1, max = 32))]
    #[serde(default)]
    pub priority: Option<String>,
}
