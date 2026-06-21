use axum::body::Body;
use axum::extract::State;
use axum::response::Response;
use axum::routing::{Router, get, post};
use http::header::CONTENT_TYPE;

use unitycatalog_sharing_client::models::sharing::v1::*;

use crate::api::sharing::{SharingHandler, SharingQueryHandler};
use crate::sharing::{SharingSkillHandler, SharingVolumeHandler};
use crate::{Error, Result};

/// Response header advertising the Delta Sharing capabilities this server
/// supports. The query path currently emits responses in `parquet` format.
const DELTA_SHARING_CAPABILITIES: &str = "delta-sharing-capabilities";
const DELTA_SHARING_CAPABILITIES_VALUE: &str = "responseformat=parquet";

/// The tabular Delta Sharing routes (shares / schemas / tables / version /
/// metadata / query).
///
/// These are shared verbatim between the Delta Sharing (`/api/v1/delta-sharing`)
/// and Open Sharing (`/api/v1/open-sharing`) mounts — both bind to the same
/// handler methods. The state is applied by the caller so the same routes can be
/// merged with the Open-Sharing-only asset routes before binding.
fn tabular_routes<T, Cx>() -> Router<T>
where
    T: SharingHandler<Cx> + SharingQueryHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    Router::new()
        .route("/shares", get(list_shares::<T, Cx>))
        .route("/shares/{share}", get(get_share::<T, Cx>))
        .route("/shares/{share}/schemas", get(list_schemas::<T, Cx>))
        .route("/shares/{share}/all-tables", get(list_all_tables::<T, Cx>))
        .route(
            "/shares/{share}/schemas/{name}/tables",
            get(list_tables::<T, Cx>),
        )
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

/// Create a [Router] for the **Delta Sharing** REST API
/// (mounted at `/api/v1/delta-sharing`).
///
/// This is the tabular surface only and is preserved for wire-compatibility with
/// existing Delta Sharing clients.
pub fn get_router<T, Cx>(state: T) -> Router
where
    T: SharingHandler<Cx> + SharingQueryHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    tabular_routes::<T, Cx>().with_state(state)
}

/// The Open-Sharing-only asset routes (volumes, agent skills): discovery plus
/// temporary-credential vending. Bound to the per-asset handler traits.
fn asset_routes<T, Cx>() -> Router<T>
where
    T: SharingVolumeHandler<Cx> + SharingSkillHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    Router::new()
        .route(
            "/shares/{share}/all-volumes",
            get(list_all_volumes::<T, Cx>),
        )
        .route(
            "/shares/{share}/schemas/{schema}/volumes",
            get(list_volumes::<T, Cx>),
        )
        .route(
            "/shares/{share}/schemas/{schema}/volumes/{name}",
            get(get_volume::<T, Cx>),
        )
        .route(
            "/shares/{share}/schemas/{schema}/volumes/{name}/temporary-volume-credentials",
            post(generate_temporary_volume_credentials::<T, Cx>),
        )
        .route("/shares/{share}/all-skills", get(list_all_skills::<T, Cx>))
        .route(
            "/shares/{share}/schemas/{schema}/skills",
            get(list_skills::<T, Cx>),
        )
        .route(
            "/shares/{share}/schemas/{schema}/skills/{name}",
            get(get_skill::<T, Cx>),
        )
        .route(
            "/shares/{share}/schemas/{schema}/skills/{name}/temporary-skill-credentials",
            post(generate_temporary_skill_credentials::<T, Cx>),
        )
}

/// Create a [Router] for the **Open Sharing** REST API
/// (mounted at `/api/v1/open-sharing`).
///
/// Open Sharing is a superset of Delta Sharing: it serves the same tabular
/// routes (via the shared [`tabular_routes`] handlers) and additionally exposes
/// the storage-backed asset routes (volumes, agent skills) via [`asset_routes`].
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

pub async fn list_shares<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListSharesRequest,
) -> Result<::axum::Json<ListSharesResponse>>
where
    T: SharingHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_shares(request, context).await?;
    Ok(axum::Json(result))
}

pub async fn get_share<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetShareRequest,
) -> Result<::axum::Json<Share>>
where
    T: SharingHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_share(request, context).await?;
    Ok(axum::Json(result))
}

pub async fn list_schemas<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListSchemasRequest,
) -> Result<::axum::Json<ListSchemasResponse>>
where
    T: SharingHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_sharing_schemas(request, context).await?;
    Ok(axum::Json(result))
}

pub async fn list_tables<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListTablesRequest,
) -> Result<::axum::Json<ListTablesResponse>>
where
    T: SharingHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_tables(request, context).await?;
    Ok(axum::Json(result))
}

pub async fn list_all_tables<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListAllTablesRequest,
) -> Result<::axum::Json<ListAllTablesResponse>>
where
    T: SharingHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_all_tables(request, context).await?;
    Ok(axum::Json(result))
}

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
    let response = Response::builder()
        .header(CONTENT_TYPE, "application/x-ndjson; charset=utf-8")
        .header(DELTA_SHARING_CAPABILITIES, DELTA_SHARING_CAPABILITIES_VALUE)
        .body(Body::from(result))
        .map_err(|e| Error::generic(e.to_string()))?;
    Ok(response)
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
    let response = Response::builder()
        .header(CONTENT_TYPE, "application/x-ndjson; charset=utf-8")
        .header(DELTA_SHARING_CAPABILITIES, DELTA_SHARING_CAPABILITIES_VALUE)
        .body(Body::from(result))
        .map_err(|e| Error::generic(e.to_string()))?;
    Ok(response)
}

// ---------------------------------------------------------------------------
// Open Sharing: volume routes
// ---------------------------------------------------------------------------

async fn list_volumes<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListVolumesRequest,
) -> Result<::axum::Json<ListVolumesResponse>>
where
    T: SharingVolumeHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    Ok(axum::Json(handler.list_volumes(request, context).await?))
}

async fn list_all_volumes<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListAllVolumesRequest,
) -> Result<::axum::Json<ListAllVolumesResponse>>
where
    T: SharingVolumeHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    Ok(axum::Json(
        handler.list_all_volumes(request, context).await?,
    ))
}

async fn get_volume<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetVolumeRequest,
) -> Result<::axum::Json<SharingVolume>>
where
    T: SharingVolumeHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    Ok(axum::Json(handler.get_volume(request, context).await?))
}

async fn generate_temporary_volume_credentials<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GenerateTemporaryVolumeCredentialsRequest,
) -> Result<::axum::Json<SharingTemporaryCredentials>>
where
    T: SharingVolumeHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    Ok(axum::Json(
        handler
            .generate_temporary_volume_credentials(request, context)
            .await?,
    ))
}

// ---------------------------------------------------------------------------
// Open Sharing: agent-skill routes
// ---------------------------------------------------------------------------

async fn list_skills<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListSkillsRequest,
) -> Result<::axum::Json<ListSkillsResponse>>
where
    T: SharingSkillHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    Ok(axum::Json(handler.list_skills(request, context).await?))
}

async fn list_all_skills<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListAllSkillsRequest,
) -> Result<::axum::Json<ListAllSkillsResponse>>
where
    T: SharingSkillHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    Ok(axum::Json(handler.list_all_skills(request, context).await?))
}

async fn get_skill<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetSkillRequest,
) -> Result<::axum::Json<SharingSkill>>
where
    T: SharingSkillHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    Ok(axum::Json(handler.get_skill(request, context).await?))
}

async fn generate_temporary_skill_credentials<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GenerateTemporarySkillCredentialsRequest,
) -> Result<::axum::Json<SharingTemporaryCredentials>>
where
    T: SharingSkillHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    Ok(axum::Json(
        handler
            .generate_temporary_skill_credentials(request, context)
            .await?,
    ))
}
