ALTER TABLE ext_support_ticket_settings
    DROP COLUMN IF EXISTS max_open_tickets_per_user,
    DROP COLUMN IF EXISTS create_ticket_rate_limit_window_seconds,
    DROP COLUMN IF EXISTS create_ticket_rate_limit_hits;
