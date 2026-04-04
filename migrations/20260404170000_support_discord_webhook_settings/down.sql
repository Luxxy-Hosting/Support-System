ALTER TABLE ext_support_ticket_settings
    DROP COLUMN IF EXISTS discord_notify_on_ticket_deleted,
    DROP COLUMN IF EXISTS discord_notify_on_assignment_change,
    DROP COLUMN IF EXISTS discord_notify_on_status_change,
    DROP COLUMN IF EXISTS discord_notify_on_internal_note,
    DROP COLUMN IF EXISTS discord_notify_on_staff_reply,
    DROP COLUMN IF EXISTS discord_notify_on_client_reply,
    DROP COLUMN IF EXISTS discord_notify_on_ticket_created,
    DROP COLUMN IF EXISTS discord_webhook_url,
    DROP COLUMN IF EXISTS discord_webhook_enabled;
