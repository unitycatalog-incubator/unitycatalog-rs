// @generated — do not edit by hand.
pub mod sharing;
pub mod sharing_skill;
pub mod sharing_volume;
use futures::Future;
pub(super) fn stream_paginated<F, Fut, S, T>(
    state: S,
    op: F,
) -> impl futures::Stream<Item = crate::Result<T>>
where
    F: Fn(S, Option<String>) -> Fut + Copy,
    Fut: Future<Output = crate::Result<(T, S, Option<String>)>>,
{
    enum PaginationState<T> {
        Start(T),
        HasMore(T, String),
        Done,
    }
    futures::stream::unfold(PaginationState::Start(state), move |state| async move {
        let (s, page_token) = match state {
            PaginationState::Start(s) => (s, None),
            PaginationState::HasMore(s, page_token) if !page_token.is_empty() => {
                (s, Some(page_token))
            }
            _ => {
                return None;
            }
        };
        let (resp, s, continuation) = match op(s, page_token).await {
            Ok(resp) => resp,
            Err(e) => return Some((Err(e), PaginationState::Done)),
        };
        let next_state = match continuation {
            Some(token) => PaginationState::HasMore(s, token),
            None => PaginationState::Done,
        };
        Some((Ok(resp), next_state))
    })
}
