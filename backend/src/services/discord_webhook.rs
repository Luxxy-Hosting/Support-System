use crate::models::ApiTicketDetail;
use serde_json::json;
use shared::State;

#[derive(Clone)]
pub struct DiscordWebhookConfig {
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

#[derive(Clone, Copy)]
pub enum DiscordWebhookEventKind {
    TicketCreated,
    ClientReply,
    StaffReply,
    InternalNote,
    StatusChanged,
    AssignmentChanged,
    TicketDeleted,
}

impl DiscordWebhookEventKind {
    fn title(self) -> &'static str {
        match self {
            Self::TicketCreated => "New Support Ticket",
            Self::ClientReply => "Client Reply",
            Self::StaffReply => "Staff Reply",
            Self::InternalNote => "Internal Note",
            Self::StatusChanged => "Ticket Status Updated",
            Self::AssignmentChanged => "Ticket Assignment Updated",
            Self::TicketDeleted => "Ticket Deleted",
        }
    }

    fn emoji(self) -> &'static str {
        match self {
            Self::TicketCreated => "Ticket",
            Self::ClientReply => "Client",
            Self::StaffReply => "Staff",
            Self::InternalNote => "Internal",
            Self::StatusChanged => "Status",
            Self::AssignmentChanged => "Assign",
            Self::TicketDeleted => "Deleted",
        }
    }

    fn color(self) -> u32 {
        match self {
            Self::TicketCreated => 0x2563eb,
            Self::ClientReply => 0x7c3aed,
            Self::StaffReply => 0x0891b2,
            Self::InternalNote => 0xd97706,
            Self::StatusChanged => 0x4f46e5,
            Self::AssignmentChanged => 0x059669,
            Self::TicketDeleted => 0xdc2626,
        }
    }

    fn enabled_in(self, config: &DiscordWebhookConfig) -> bool {
        match self {
            Self::TicketCreated => config.notify_on_ticket_created,
            Self::ClientReply => config.notify_on_client_reply,
            Self::StaffReply => config.notify_on_staff_reply,
            Self::InternalNote => config.notify_on_internal_note,
            Self::StatusChanged => config.notify_on_status_change,
            Self::AssignmentChanged => config.notify_on_assignment_change,
            Self::TicketDeleted => config.notify_on_ticket_deleted,
        }
    }
}

pub struct DiscordWebhookEvent {
    pub kind: DiscordWebhookEventKind,
    pub actor_display_name: Option<String>,
    pub actor_username: Option<String>,
    pub latest_message_html: Option<String>,
    pub extra_lines: Vec<String>,
}

pub async fn send_ticket_event(
    state: &State,
    config: &DiscordWebhookConfig,
    detail: &ApiTicketDetail,
    event: DiscordWebhookEvent,
) -> Result<(), anyhow::Error> {
    if !config.enabled || !event.kind.enabled_in(config) {
        return Ok(());
    }

    let Some(webhook_url) = config
        .webhook_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(());
    };

    let settings = state.settings.get().await?;
    let panel_url = settings.app.url.trim_end_matches('/').to_string();
    let app_name = settings.app.name.to_string();
    let app_icon_url = resolve_app_icon_url(&panel_url, settings.app.icon.as_str());
    drop(settings);

    let ticket_url = format!("{panel_url}/admin/support/{}", detail.ticket.uuid);
    let server_url = detail
        .ticket
        .linked_server
        .uuid
        .map(|uuid| format!("{panel_url}/admin/servers/{uuid}"));

    let latest_message = event
        .latest_message_html
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(html_to_discord_text)
        .map(|value| truncate_for_discord(&value, 1200))
        .filter(|value| !value.is_empty());

    let linked_server_label = describe_linked_server(detail);
    let ticket_line = format_markdown_link(&detail.ticket.subject, &ticket_url);
    let server_line = server_url
        .as_deref()
        .map(|url| format_markdown_link(&linked_server_label, url))
        .unwrap_or_else(|| escape_markdown(&linked_server_label));

    let actor_line = event
        .actor_display_name
        .as_deref()
        .map(|actor_display_name| {
            let actor_username_suffix = event
                .actor_username
                .as_deref()
                .map(|value| format!(" (@{})", escape_markdown(value)))
                .unwrap_or_default();
            format!(
                "**Actor:** {}{}",
                escape_markdown(actor_display_name),
                actor_username_suffix
            )
        });

    let mut description_lines = vec![
        format!("**Ticket:** {ticket_line}"),
        format!(
            "**Client:** @{}",
            escape_markdown(&detail.ticket.creator.username)
        ),
        format!("**Linked Server:** {server_line}"),
    ];

    if let Some(category) = detail.ticket.category.as_ref() {
        description_lines.push(format!("**Category:** {}", escape_markdown(&category.name)));
    }

    if let Some(actor_line) = actor_line {
        description_lines.push(actor_line);
    }

    description_lines.extend(event.extra_lines.into_iter());

    if let Some(message) = latest_message.as_deref() {
        description_lines.push(String::new());
        description_lines.push("**Latest Message**".to_string());
        description_lines.push(quote_discord_text(message));
    }

    let mut fields = vec![
        json!({
            "name": "Client",
            "value": truncate_for_discord(&format!("@{}", escape_markdown(&detail.ticket.creator.username)), 1024),
            "inline": true,
        }),
        json!({
            "name": "Status",
            "value": truncate_for_discord(&humanize_slug(&detail.ticket.status), 1024),
            "inline": true,
        }),
        json!({
            "name": "Priority",
            "value": truncate_for_discord(
                &detail
                    .ticket
                    .priority
                    .as_deref()
                    .map(humanize_slug)
                    .unwrap_or_else(|| "Normal".to_string()),
                1024,
            ),
            "inline": true,
        }),
    ];

    if let Some(category) = detail.ticket.category.as_ref() {
        fields.push(json!({
            "name": "Category",
            "value": truncate_for_discord(&escape_markdown(&category.name), 1024),
            "inline": true,
        }));
    }

    if let Some(actor_display_name) = event.actor_display_name.as_deref() {
        let actor_username_suffix = event
            .actor_username
            .as_deref()
            .map(|value| format!(" (@{})", escape_markdown(value)))
            .unwrap_or_default();
        fields.push(json!({
            "name": "Actor",
            "value": truncate_for_discord(
                &format!("{}{}", escape_markdown(actor_display_name), actor_username_suffix),
                1024,
            ),
            "inline": true,
        }));
    }

    let mut buttons = vec![json!({
        "type": 2,
        "style": 5,
        "label": "Open Ticket",
        "url": ticket_url,
    })];

    if let Some(server_url) = server_url {
        buttons.push(json!({
            "type": 2,
            "style": 5,
            "label": "Open Server",
            "url": server_url,
        }));
    }

    state
        .client
        .post(webhook_url)
        .query(&[("wait", "false"), ("with_components", "true")])
        .json(&json!({
            "username": format!("{} Support", app_name),
            "avatar_url": app_icon_url,
            "allowed_mentions": { "parse": [] },
            "embeds": [{
                "author": {
                    "name": format!("{} Support", app_name),
                    "url": panel_url,
                    "icon_url": app_icon_url,
                },
                "title": format!("{} • {}", event.kind.title(), detail.ticket.subject),
                "url": ticket_url,
                "description": truncate_for_discord(&description_lines.join("\n"), 4096),
                "color": event.kind.color(),
                "fields": fields,
                "thumbnail": app_icon_url.as_ref().map(|url| json!({ "url": url })),
                "footer": {
                    "text": format!("{} • Ticket {}", event.kind.emoji(), detail.ticket.uuid),
                },
                "timestamp": detail
                    .ticket
                    .last_reply_at
                    .unwrap_or(detail.ticket.updated)
                    .to_rfc3339(),
            }],
            "components": [{
                "type": 1,
                "components": buttons,
            }],
        }))
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

fn describe_linked_server(detail: &ApiTicketDetail) -> String {
    let linked_server = &detail.ticket.linked_server;

    if let Some(name) = linked_server.current_name.as_ref() {
        return match linked_server.current_uuid_short {
            Some(uuid_short) => format!("{name} (#{uuid_short})"),
            None => name.clone(),
        };
    }

    if let Some(name) = linked_server.snapshot_name.as_ref() {
        return match linked_server.snapshot_uuid_short {
            Some(uuid_short) => format!("{name} (#{uuid_short})"),
            None => name.clone(),
        };
    }

    "General Ticket".to_string()
}

fn humanize_slug(value: &str) -> String {
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

fn escape_markdown(value: &str) -> String {
    value
        .replace('*', "\\*")
        .replace('_', "\\_")
        .replace('`', "\\`")
}

fn escape_markdown_link_label(value: &str) -> String {
    escape_markdown(value)
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

fn format_markdown_link(label: &str, url: &str) -> String {
    format!("[{}]({})", escape_markdown_link_label(label), url)
}

fn quote_discord_text(value: &str) -> String {
    value
        .lines()
        .map(|line| {
            if line.trim().is_empty() {
                ">".to_string()
            } else {
                format!("> {}", line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn resolve_app_icon_url(panel_url: &str, icon: &str) -> Option<String> {
    let trimmed = icon.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return Some(trimmed.to_string());
    }

    if trimmed.starts_with('/') {
        return Some(format!("{}{}", panel_url.trim_end_matches('/'), trimmed));
    }

    Some(format!(
        "{}/{}",
        panel_url.trim_end_matches('/'),
        trimmed.trim_start_matches('/')
    ))
}

fn truncate_for_discord(value: &str, max_len: usize) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= max_len {
        return trimmed.to_string();
    }

    let truncated = trimmed
        .chars()
        .take(max_len.saturating_sub(1))
        .collect::<String>();
    format!("{}…", truncated.trim_end())
}

fn html_to_discord_text(value: &str) -> String {
    let mut normalized = value
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .replace("</p>", "\n\n")
        .replace("</div>", "\n")
        .replace("</li>", "\n")
        .replace("<li>", "• ")
        .replace("<strong>", "**")
        .replace("</strong>", "**")
        .replace("<b>", "**")
        .replace("</b>", "**")
        .replace("<em>", "*")
        .replace("</em>", "*")
        .replace("<i>", "*")
        .replace("</i>", "*")
        .replace("<s>", "~~")
        .replace("</s>", "~~")
        .replace("<code>", "`")
        .replace("</code>", "`")
        .replace("<pre>", "```\n")
        .replace("</pre>", "\n```")
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'");

    let mut output = String::with_capacity(normalized.len());
    let mut inside_tag = false;

    for character in normalized.drain(..) {
        match character {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => output.push(character),
            _ => {}
        }
    }

    let mut collapsed = String::with_capacity(output.len());
    let mut newline_run = 0usize;
    for character in output.chars() {
        if character == '\n' {
            newline_run += 1;
            if newline_run <= 2 {
                collapsed.push(character);
            }
        } else {
            newline_run = 0;
            collapsed.push(character);
        }
    }

    collapsed.trim().to_string()
}
