CREATE TABLE IF NOT EXISTS ext_support_ticket_settings (
    uuid uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    categories_enabled boolean NOT NULL DEFAULT TRUE,
    allow_client_close boolean NOT NULL DEFAULT TRUE,
    allow_reply_on_closed boolean NOT NULL DEFAULT FALSE,
    created timestamp NOT NULL DEFAULT NOW(),
    updated timestamp NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS ext_support_ticket_categories (
    uuid uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name varchar(255) NOT NULL UNIQUE,
    description text,
    color varchar(32),
    sort_order integer NOT NULL DEFAULT 0,
    enabled boolean NOT NULL DEFAULT TRUE,
    created timestamp NOT NULL DEFAULT NOW(),
    updated timestamp NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS ext_support_ticket_categories_sort_idx
    ON ext_support_ticket_categories(sort_order, name);

CREATE TABLE IF NOT EXISTS ext_support_tickets (
    uuid uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    creator_user_uuid uuid NOT NULL REFERENCES users(uuid) ON DELETE RESTRICT,
    creator_snapshot_username varchar(255) NOT NULL,
    creator_snapshot_email varchar(255) NOT NULL,
    linked_server_uuid uuid,
    linked_server_snapshot_name varchar(255),
    linked_server_snapshot_uuid_short integer,
    linked_server_deleted_at timestamp,
    category_uuid uuid REFERENCES ext_support_ticket_categories(uuid) ON DELETE SET NULL,
    subject varchar(255) NOT NULL,
    status varchar(32) NOT NULL DEFAULT 'waiting_on_staff',
    priority varchar(16) NOT NULL DEFAULT 'normal',
    assigned_user_uuid uuid REFERENCES users(uuid) ON DELETE SET NULL,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    last_reply_at timestamp,
    last_reply_by_type varchar(16),
    closed_at timestamp,
    closed_by_user_uuid uuid REFERENCES users(uuid) ON DELETE SET NULL,
    deleted_at timestamp,
    created timestamp NOT NULL DEFAULT NOW(),
    updated timestamp NOT NULL DEFAULT NOW(),
    CHECK (status IN ('open', 'pending', 'answered', 'waiting_on_client', 'waiting_on_staff', 'closed')),
    CHECK (priority IN ('low', 'normal', 'high', 'urgent')),
    CHECK (last_reply_by_type IS NULL OR last_reply_by_type IN ('client', 'staff', 'system'))
);

CREATE INDEX IF NOT EXISTS ext_support_tickets_creator_idx
    ON ext_support_tickets(creator_user_uuid, created DESC);

CREATE INDEX IF NOT EXISTS ext_support_tickets_status_idx
    ON ext_support_tickets(status, deleted_at, created DESC);

CREATE INDEX IF NOT EXISTS ext_support_tickets_assigned_idx
    ON ext_support_tickets(assigned_user_uuid, status, created DESC);

CREATE INDEX IF NOT EXISTS ext_support_tickets_category_idx
    ON ext_support_tickets(category_uuid, status, created DESC);

CREATE INDEX IF NOT EXISTS ext_support_tickets_linked_server_idx
    ON ext_support_tickets(linked_server_uuid, created DESC);

CREATE INDEX IF NOT EXISTS ext_support_tickets_last_reply_idx
    ON ext_support_tickets(last_reply_at DESC);

CREATE TABLE IF NOT EXISTS ext_support_ticket_messages (
    uuid uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    ticket_uuid uuid NOT NULL REFERENCES ext_support_tickets(uuid) ON DELETE CASCADE,
    author_user_uuid uuid REFERENCES users(uuid) ON DELETE SET NULL,
    author_snapshot_username varchar(255) NOT NULL,
    author_snapshot_display_name varchar(255) NOT NULL,
    author_type varchar(16) NOT NULL,
    body text NOT NULL,
    is_internal boolean NOT NULL DEFAULT FALSE,
    created timestamp NOT NULL DEFAULT NOW(),
    updated timestamp NOT NULL DEFAULT NOW(),
    CHECK (author_type IN ('client', 'staff', 'system'))
);

CREATE INDEX IF NOT EXISTS ext_support_ticket_messages_ticket_idx
    ON ext_support_ticket_messages(ticket_uuid, created ASC);

CREATE INDEX IF NOT EXISTS ext_support_ticket_messages_ticket_internal_idx
    ON ext_support_ticket_messages(ticket_uuid, is_internal, created ASC);

CREATE TABLE IF NOT EXISTS ext_support_ticket_audit_events (
    uuid uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    ticket_uuid uuid NOT NULL REFERENCES ext_support_tickets(uuid) ON DELETE CASCADE,
    actor_user_uuid uuid REFERENCES users(uuid) ON DELETE SET NULL,
    actor_snapshot_username varchar(255),
    actor_type varchar(16) NOT NULL,
    event varchar(64) NOT NULL,
    payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    created timestamp NOT NULL DEFAULT NOW(),
    CHECK (actor_type IN ('client', 'staff', 'system'))
);

CREATE INDEX IF NOT EXISTS ext_support_ticket_audit_events_ticket_idx
    ON ext_support_ticket_audit_events(ticket_uuid, created ASC);
