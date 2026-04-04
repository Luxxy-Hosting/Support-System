CREATE TABLE IF NOT EXISTS ext_support_ticket_attachments (
    uuid uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    ticket_uuid uuid NOT NULL REFERENCES ext_support_tickets(uuid) ON DELETE CASCADE,
    message_uuid uuid NOT NULL REFERENCES ext_support_ticket_messages(uuid) ON DELETE CASCADE,
    uploader_user_uuid uuid REFERENCES users(uuid) ON DELETE SET NULL,
    storage_path varchar(1024) NOT NULL UNIQUE,
    original_name varchar(255) NOT NULL,
    content_type varchar(255) NOT NULL,
    media_type varchar(16) NOT NULL,
    size bigint NOT NULL,
    created timestamp NOT NULL DEFAULT NOW(),
    CHECK (media_type IN ('image', 'video'))
);

CREATE INDEX IF NOT EXISTS ext_support_ticket_attachments_ticket_idx
    ON ext_support_ticket_attachments(ticket_uuid, created ASC);

CREATE INDEX IF NOT EXISTS ext_support_ticket_attachments_message_idx
    ON ext_support_ticket_attachments(message_uuid, created ASC);
