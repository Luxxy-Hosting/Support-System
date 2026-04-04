# Support Center

Calagopus support ticket extension for client-to-staff support workflows.

## Features

- Client support ticket creation from the panel
- Optional server-linked tickets
- Staff/admin ticket queue and ticket detail views
- Replies, internal notes, status changes, assignment, and categories
- Image/video attachments inside ticket conversations
- Discord webhook notifications for ticket activity

## Included

- Backend extension: `backend-extensions/dev_luxxy_supportsystem`
- Frontend extension: `frontend/extensions/dev_luxxy_supportsystem`
- Extension migrations: `database/extension-migrations/dev_luxxy_supportsystem`

## Admin Pages

- Ticket queue: `/admin/support`
- Extension settings: `/admin/extensions/dev.luxxy.supportsystem`

## Notes

- Ticket settings and category management live in the extension settings page.
- Discord webhook settings are configured from the extension settings page.
- Run the support extension migrations before using it.

## Dev Check

```bash
SQLX_OFFLINE=true cargo check -p dev_luxxy_supportsystem
```
