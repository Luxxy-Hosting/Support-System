use shared::models::{DeletableModel, ListenerPriority, server::Server};

use super::manager;

pub async fn register_handlers() {
    Server::register_delete_handler(ListenerPriority::Low, |server, _options, state, _tx| {
        Box::pin(async move {
            if let Err(err) = manager::mark_linked_server_deleted(state, server.uuid).await {
                tracing::error!(server = %server.uuid, "support ticket server-delete lifecycle sync failed: {err:#}");
            }

            Ok(())
        })
    })
    .await;
}
