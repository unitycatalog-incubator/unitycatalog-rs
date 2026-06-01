// @generated — do not edit by hand.
#![allow(unused_mut)]
use super::handler::DeltaCommitHandler;
use crate::Result;
use axum::extract::State;
use unitycatalog_common::models::delta_commits::v1::*;
pub async fn commit<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: CommitRequest,
) -> Result<()>
where
    T: DeltaCommitHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    handler.commit(request, context).await?;
    Ok(())
}
pub async fn get_commits<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetCommitsRequest,
) -> Result<::axum::Json<GetCommitsResponse>>
where
    T: DeltaCommitHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_commits(request, context).await?;
    Ok(axum::Json(result))
}
