mod admin;
mod client;
mod multipart;

use shared::State;
use utoipa_axum::router::OpenApiRouter;

pub fn admin_router(state: &State) -> OpenApiRouter<State> {
    admin::router(state)
}

pub fn client_router(state: &State) -> OpenApiRouter<State> {
    client::router(state)
}
