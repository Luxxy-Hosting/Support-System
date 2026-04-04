use axum::{body::Bytes, extract::Multipart, http::StatusCode};
use shared::response::DisplayError;

use crate::services::manager::IncomingAttachmentUpload;

pub const MAX_ATTACHMENTS_PER_MESSAGE: usize = 5;
pub const MAX_IMAGE_ATTACHMENT_BYTES: usize = 12 * 1024 * 1024;
pub const MAX_VIDEO_ATTACHMENT_BYTES: usize = 80 * 1024 * 1024;
pub const MAX_TOTAL_ATTACHMENT_BYTES: usize = 100 * 1024 * 1024;

pub struct CreateTicketMultipartForm {
    pub server_uuid: Option<uuid::Uuid>,
    pub category_uuid: Option<uuid::Uuid>,
    pub subject: String,
    pub message: String,
    pub metadata: Option<serde_json::Value>,
    pub attachments: Vec<IncomingAttachmentUpload>,
}

pub struct MessageMultipartForm {
    pub body: String,
    pub is_internal: bool,
    pub attachments: Vec<IncomingAttachmentUpload>,
}

pub async fn parse_create_ticket_form(
    mut multipart: Multipart,
) -> Result<CreateTicketMultipartForm, anyhow::Error> {
    let mut server_uuid = None;
    let mut category_uuid = None;
    let mut subject = String::new();
    let mut message = String::new();
    let mut metadata = None;
    let mut attachments = Vec::new();
    let mut total_attachment_bytes = 0usize;

    while let Some(field) = multipart.next_field().await? {
        let field_name = field.name().unwrap_or_default().to_string();

        match field_name.as_str() {
            "serverUuid" | "server_uuid" => {
                let value = field.text().await?;
                server_uuid = parse_optional_uuid(&value)?;
            }
            "categoryUuid" | "category_uuid" => {
                let value = field.text().await?;
                category_uuid = parse_optional_uuid(&value)?;
            }
            "subject" => {
                subject = field.text().await?;
            }
            "message" | "body" => {
                message = field.text().await?;
            }
            "metadata" => {
                let value = field.text().await?;
                metadata = if value.trim().is_empty() {
                    None
                } else {
                    Some(serde_json::from_str(&value).map_err(|_| {
                        DisplayError::new("ticket metadata must be valid JSON")
                            .with_status(StatusCode::BAD_REQUEST)
                    })?)
                };
            }
            "files" | "attachments" => {
                let attachment = parse_attachment_field(field, &mut total_attachment_bytes).await?;
                attachments.push(attachment);
            }
            _ => {}
        }
    }

    enforce_attachment_limits(&attachments)?;

    Ok(CreateTicketMultipartForm {
        server_uuid,
        category_uuid,
        subject,
        message,
        metadata,
        attachments,
    })
}

pub async fn parse_message_form(mut multipart: Multipart) -> Result<MessageMultipartForm, anyhow::Error> {
    let mut body = String::new();
    let mut is_internal = false;
    let mut attachments = Vec::new();
    let mut total_attachment_bytes = 0usize;

    while let Some(field) = multipart.next_field().await? {
        let field_name = field.name().unwrap_or_default().to_string();

        match field_name.as_str() {
            "body" | "message" => {
                body = field.text().await?;
            }
            "isInternal" | "is_internal" => {
                let value = field.text().await?;
                is_internal = matches!(value.trim(), "true" | "1" | "yes" | "on");
            }
            "files" | "attachments" => {
                let attachment = parse_attachment_field(field, &mut total_attachment_bytes).await?;
                attachments.push(attachment);
            }
            _ => {}
        }
    }

    enforce_attachment_limits(&attachments)?;

    Ok(MessageMultipartForm {
        body,
        is_internal,
        attachments,
    })
}

fn parse_optional_uuid(value: &str) -> Result<Option<uuid::Uuid>, anyhow::Error> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    Ok(Some(uuid::Uuid::parse_str(trimmed).map_err(|_| {
        DisplayError::new("invalid UUID value").with_status(StatusCode::BAD_REQUEST)
    })?))
}

fn enforce_attachment_limits(attachments: &[IncomingAttachmentUpload]) -> Result<(), anyhow::Error> {
    if attachments.len() > MAX_ATTACHMENTS_PER_MESSAGE {
        return Err(DisplayError::new(format!(
            "you can upload up to {MAX_ATTACHMENTS_PER_MESSAGE} attachments per message"
        ))
        .with_status(StatusCode::BAD_REQUEST)
        .into());
    }

    Ok(())
}

async fn parse_attachment_field(
    field: axum::extract::multipart::Field<'_>,
    total_attachment_bytes: &mut usize,
) -> Result<IncomingAttachmentUpload, anyhow::Error> {
    let original_name = sanitize_filename(field.file_name().unwrap_or("attachment"));
    let content_type = field
        .content_type()
        .unwrap_or("application/octet-stream")
        .trim()
        .to_string();

    let media_type = classify_media_type(&content_type)?;
    let data: Bytes = field.bytes().await?;
    let size = data.len();

    let per_file_limit = match media_type.as_str() {
        "image" => MAX_IMAGE_ATTACHMENT_BYTES,
        "video" => MAX_VIDEO_ATTACHMENT_BYTES,
        _ => unreachable!(),
    };

    if size == 0 {
        return Err(DisplayError::new("attachments cannot be empty")
            .with_status(StatusCode::BAD_REQUEST)
            .into());
    }

    if size > per_file_limit {
        let limit_mb = per_file_limit / 1024 / 1024;
        return Err(DisplayError::new(format!(
            "{media_type} attachments must be smaller than {limit_mb} MiB"
        ))
        .with_status(StatusCode::BAD_REQUEST)
        .into());
    }

    *total_attachment_bytes += size;
    if *total_attachment_bytes > MAX_TOTAL_ATTACHMENT_BYTES {
        let limit_mb = MAX_TOTAL_ATTACHMENT_BYTES / 1024 / 1024;
        return Err(DisplayError::new(format!(
            "total attachment size must be smaller than {limit_mb} MiB"
        ))
        .with_status(StatusCode::BAD_REQUEST)
        .into());
    }

    Ok(IncomingAttachmentUpload {
        original_name,
        content_type,
        media_type,
        data,
    })
}

fn classify_media_type(content_type: &str) -> Result<String, anyhow::Error> {
    let media_type = match content_type {
        "image/png" | "image/jpeg" | "image/jpg" | "image/gif" | "image/webp" | "image/avif" => {
            "image"
        }
        "video/mp4" | "video/webm" | "video/ogg" | "video/quicktime" => "video",
        _ => {
            return Err(DisplayError::new(
                "attachments must be PNG, JPEG, GIF, WebP, AVIF, MP4, WebM, OGG, or MOV",
            )
            .with_status(StatusCode::BAD_REQUEST)
            .into())
        }
    };

    Ok(media_type.to_string())
}

fn sanitize_filename(value: &str) -> String {
    let file_name = value
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or("attachment")
        .trim();

    let mut sanitized = file_name
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_') {
                character
            } else {
                '-'
            }
        })
        .collect::<String>();

    sanitized.truncate(96);
    while sanitized.contains("--") {
        sanitized = sanitized.replace("--", "-");
    }
    sanitized = sanitized.trim_matches(['-', '.']).to_string();

    if sanitized.is_empty() {
        "attachment".to_string()
    } else {
        sanitized
    }
}
