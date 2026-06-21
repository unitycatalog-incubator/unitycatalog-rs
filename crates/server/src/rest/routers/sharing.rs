use axum::body::Body;
use axum::extract::State;
use axum::response::Response;
use axum::routing::{Router, get, post};
use http::header::CONTENT_TYPE;

use unitycatalog_sharing_client::models::open_sharing::v1::*;

use crate::api::sharing::SharingQueryHandler;
use crate::sharing::codegen::{sharing, sharing_skill, sharing_volume};
use crate::sharing::{SharingHandler, SharingSkillHandler, SharingVolumeHandler};
use crate::{Error, Result};

/// Response header advertising the Delta Sharing capabilities this server
/// supports. The query path currently emits responses in `parquet` format.
const DELTA_SHARING_CAPABILITIES: &str = "delta-sharing-capabilities";
const DELTA_SHARING_CAPABILITIES_VALUE: &str = "responseformat=parquet";

/// The tabular Delta Sharing routes (shares / schemas / tables / version /
/// metadata / query).
///
/// Shared verbatim between the Delta Sharing (`/api/v1/delta-sharing`) and Open
/// Sharing (`/api/v1/open-sharing`) mounts. The discovery routes bind to the
/// trestle-generated [`SharingHandler`] route functions; the three NDJSON query
/// routes (version/metadata/query) bind to the hand-written functions below,
/// since their streaming response contract is not modelled by the generated,
/// JSON-only handlers.
fn tabular_routes<T, Cx>() -> Router<T>
where
    T: SharingHandler<Cx> + SharingQueryHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    Router::new()
        .route("/shares", get(sharing::server::list_shares::<T, Cx>))
        .route("/shares/{name}", get(sharing::server::get_share::<T, Cx>))
        .route(
            "/shares/{share}/schemas",
            get(sharing::server::list_schemas::<T, Cx>),
        )
        .route(
            "/shares/{share}/all-tables",
            get(sharing::server::list_all_tables::<T, Cx>),
        )
        .route(
            "/shares/{share}/schemas/{name}/tables",
            get(sharing::server::list_tables::<T, Cx>),
        )
        // Hand-written NDJSON query path (not part of the generated service).
        .route(
            "/shares/{share}/schemas/{schema}/tables/{name}/version",
            get(get_table_version::<T, Cx>),
        )
        .route(
            "/shares/{share}/schemas/{schema}/tables/{name}/metadata",
            get(get_table_metadata::<T, Cx>),
        )
        .route(
            "/shares/{share}/schemas/{schema}/tables/{name}/query",
            post(get_table_query::<T, Cx>),
        )
}

/// The Open-Sharing-only asset routes (volumes, agent skills), bound to the
/// trestle-generated per-asset route functions.
fn asset_routes<T, Cx>() -> Router<T>
where
    T: SharingVolumeHandler<Cx> + SharingSkillHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    Router::new()
        .route(
            "/shares/{share}/all-volumes",
            get(sharing_volume::server::list_all_volumes::<T, Cx>),
        )
        .route(
            "/shares/{share}/schemas/{schema}/volumes",
            get(sharing_volume::server::list_volumes::<T, Cx>),
        )
        .route(
            "/shares/{share}/schemas/{schema}/volumes/{name}",
            get(sharing_volume::server::get_volume::<T, Cx>),
        )
        .route(
            "/shares/{share}/schemas/{schema}/volumes/{name}/temporary-volume-credentials",
            post(sharing_volume::server::generate_temporary_volume_credentials::<T, Cx>),
        )
        .route(
            "/shares/{share}/all-skills",
            get(sharing_skill::server::list_all_skills::<T, Cx>),
        )
        .route(
            "/shares/{share}/schemas/{schema}/skills",
            get(sharing_skill::server::list_skills::<T, Cx>),
        )
        .route(
            "/shares/{share}/schemas/{schema}/skills/{name}",
            get(sharing_skill::server::get_skill::<T, Cx>),
        )
        .route(
            "/shares/{share}/schemas/{schema}/skills/{name}/temporary-skill-credentials",
            post(sharing_skill::server::generate_temporary_skill_credentials::<T, Cx>),
        )
}

/// Create a [Router] for the **Delta Sharing** REST API
/// (mounted at `/api/v1/delta-sharing`) — the tabular surface only, preserved
/// for wire-compatibility with existing Delta Sharing clients.
pub fn get_router<T, Cx>(state: T) -> Router
where
    T: SharingHandler<Cx> + SharingQueryHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    tabular_routes::<T, Cx>().with_state(state)
}

/// Create a [Router] for the **Open Sharing** REST API
/// (mounted at `/api/v1/open-sharing`): the tabular surface plus the
/// storage-backed asset routes (volumes, agent skills).
pub fn open_sharing_router<T, Cx>(state: T) -> Router
where
    T: SharingHandler<Cx>
        + SharingQueryHandler<Cx>
        + SharingVolumeHandler<Cx>
        + SharingSkillHandler<Cx>
        + Clone
        + Send
        + Sync
        + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    tabular_routes::<T, Cx>()
        .merge(asset_routes::<T, Cx>())
        .with_state(state)
}

// ---------------------------------------------------------------------------
// Hand-written NDJSON query routes (version / metadata / query).
//
// These return `application/x-ndjson` with the `Delta-Table-Version` /
// `delta-sharing-capabilities` headers — a streaming contract the generated
// JSON route functions do not model — so they are bound to `SharingQueryHandler`
// by hand rather than generated.
// ---------------------------------------------------------------------------

async fn get_table_version<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetTableVersionRequest,
) -> Result<Response>
where
    T: SharingQueryHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_table_version(request, context).await?;
    Response::builder()
        .header("Delta-Table-Version", result.version)
        .body(Body::empty())
        .map_err(|e| Error::generic(e.to_string()))
}

async fn get_table_metadata<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetTableMetadataRequest,
) -> Result<Response>
where
    T: SharingQueryHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_table_metadata(request, context).await?;
    Response::builder()
        .header(CONTENT_TYPE, "application/x-ndjson; charset=utf-8")
        .header(DELTA_SHARING_CAPABILITIES, DELTA_SHARING_CAPABILITIES_VALUE)
        .body(Body::from(result))
        .map_err(|e| Error::generic(e.to_string()))
}

async fn get_table_query<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: QueryTableRequest,
) -> Result<Response>
where
    T: SharingQueryHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.query_table(request, context).await?;
    Response::builder()
        .header(CONTENT_TYPE, "application/x-ndjson; charset=utf-8")
        .header(DELTA_SHARING_CAPABILITIES, DELTA_SHARING_CAPABILITIES_VALUE)
        .body(Body::from(result))
        .map_err(|e| Error::generic(e.to_string()))
}
