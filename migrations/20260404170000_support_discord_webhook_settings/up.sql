ALTER TABLE ext_support_ticket_settings
    ADD COLUMN IF NOT EXISTS discord_webhook_enabled boolean NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS discord_webhook_url varchar(2048),
    ADD COLUMN IF NOT EXISTS discord_notify_on_ticket_created boolean NOT NULL DEFAULT TRUE,
    ADD COLUMN IF NOT EXISTS discord_notify_on_client_reply boolean NOT NULL DEFAULT TRUE,
    ADD COLUMN IF NOT EXISTS discord_notify_on_staff_reply boolean NOT NULL DEFAULT TRUE,
    ADD COLUMN IF NOT EXISTS discord_notify_on_internal_note boolean NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS discord_notify_on_status_change boolean NOT NULL DEFAULT TRUE,
    ADD COLUMN IF NOT EXISTS discord_notify_on_assignment_change boolean NOT NULL DEFAULT TRUE,
    ADD COLUMN IF NOT EXISTS discord_notify_on_ticket_deleted boolean NOT NULL DEFAULT FALSE;
