use super::handler::ExternalLocationHandler;
use crate::api::RequestContext;
use crate::models::external_locations::v1::*;
use crate::services::Recipient;
use crate::Result;
pub async fn list_external_locations_handler<T: ExternalLocationHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: ListExternalLocationsRequest,
) -> Result<::axum::Json<ListExternalLocationsResponse>> {
    let context = RequestContext { recipient };
    let result = handler.list_external_locations(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_external_location_handler<T: ExternalLocationHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: CreateExternalLocationRequest,
) -> Result<::axum::Json<ExternalLocationInfo>> {
    let context = RequestContext { recipient };
    let result = handler.create_external_location(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_external_location_handler<T: ExternalLocationHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: GetExternalLocationRequest,
) -> Result<::axum::Json<ExternalLocationInfo>> {
    let context = RequestContext { recipient };
    let result = handler.get_external_location(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_external_location_handler<T: ExternalLocationHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: UpdateExternalLocationRequest,
) -> Result<::axum::Json<ExternalLocationInfo>> {
    let context = RequestContext { recipient };
    let result = handler.update_external_location(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_external_location_handler<T: ExternalLocationHandler>(
    ::axum::extract::State(handler): ::axum::extract::State<T>,
    ::axum::extract::Extension(recipient): ::axum::extract::Extension<Recipient>,
    request: DeleteExternalLocationRequest,
) -> Result<()> {
    let context = RequestContext { recipient };
    handler.delete_external_location(request, context).await?;
    Ok(())
}
