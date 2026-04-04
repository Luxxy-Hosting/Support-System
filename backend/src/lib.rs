use indexmap::IndexMap;
use shared::{
    State,
    extensions::{Extension, ExtensionPermissionsBuilder, ExtensionRouteBuilder},
    permissions::PermissionGroup,
};

mod models;
mod routes;
mod services;

#[derive(Default)]
pub struct ExtensionStruct;

#[async_trait::async_trait]
impl Extension for ExtensionStruct {
    async fn initialize(&mut self, _state: State) {
        services::lifecycle::register_handlers().await;
        tracing::info!("support system extension initialized");
    }

    async fn initialize_router(
        &mut self,
        state: State,
        builder: ExtensionRouteBuilder,
    ) -> ExtensionRouteBuilder {
        builder
            .add_admin_api_router(|router| router.nest("/support", routes::admin_router(&state)))
            .add_client_api_router(|router| router.nest("/support", routes::client_router(&state)))
    }

    async fn initialize_permissions(
        &mut self,
        _state: State,
        builder: ExtensionPermissionsBuilder,
    ) -> ExtensionPermissionsBuilder {
        builder
            .add_user_permission_group(
                "tickets",
                PermissionGroup {
                    description: "Permissions that control the ability to create and manage your own support tickets.",
                    permissions: IndexMap::from([
                        ("view-own", "Allows viewing your own support tickets."),
                        ("create", "Allows creating new support tickets."),
                        ("reply-own", "Allows replying to and closing your own support tickets."),
                    ]),
                },
            )
            .add_admin_permission_group(
                "tickets",
                PermissionGroup {
                    description: "Permissions that control the ability to manage support tickets for clients.",
                    permissions: IndexMap::from([
                        ("view-all", "Allows viewing all support tickets and queues."),
                        ("reply-all", "Allows replying publicly to all support tickets."),
                        ("change-status", "Allows changing ticket status, priority, and category."),
                        ("assign", "Allows assigning tickets to staff users."),
                        ("add-internal-notes", "Allows creating staff-only internal notes on tickets."),
                        ("delete", "Allows soft-deleting support tickets."),
                        ("manage-categories", "Allows creating, updating, and deleting support ticket categories."),
                        ("manage-settings", "Allows updating support ticket settings."),
                    ]),
                },
            )
    }
}
