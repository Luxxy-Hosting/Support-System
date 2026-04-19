ALTER TABLE ext_support_ticket_settings
    ADD COLUMN IF NOT EXISTS create_ticket_rate_limit_hits integer NOT NULL DEFAULT 20,
    ADD COLUMN IF NOT EXISTS create_ticket_rate_limit_window_seconds integer NOT NULL DEFAULT 300,
    ADD COLUMN IF NOT EXISTS max_open_tickets_per_user integer NOT NULL DEFAULT 0;
