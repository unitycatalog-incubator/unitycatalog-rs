//! Authorization policies.
//!
//! Policies are used to determine whether a recipient is allowed to perform a specific action on a
//! resource. The action is represented by a [`Permission`] and the resource is represented by a
//! [`Resource`]. The [`Decision`] represents whether the action is allowed or denied for the given
//! recipient.

use std::sync::Arc;

use strum::AsRefStr;
use unitycatalog_common::models::{ResourceExt, ResourceIdent};

pub use self::constant::*;
use crate::api::SecuredAction;
use crate::{Error, Result};

mod constant;

#[derive(Clone, Debug)]
pub enum Principal {
    Anonymous,
    User(String),
}

impl Principal {
    pub fn anonymous() -> Self {
        Self::Anonymous
    }

    pub fn user(name: impl Into<String>) -> Self {
        Self::User(name.into())
    }
}

/// Permission that a policy can authorize.
#[derive(Debug, Clone, AsRefStr, PartialEq, Eq, strum::EnumString)]
#[strum(serialize_all = "snake_case", ascii_case_insensitive)]
pub enum Permission {
    Read,
    Write,
    Manage,
    Create,
    Use,
    Browse,
    Select,
}

impl From<Permission> for String {
    fn from(val: Permission) -> Self {
        val.as_ref().to_string()
    }
}

/// Decision made by a policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Allow the action.
    Allow,
    /// Deny the action.
    Deny,
}

/// Policy for access control.
#[async_trait::async_trait]
pub trait Policy<Cx: Send + Sync + 'static>: Send + Sync + 'static {
    async fn check(&self, obj: &dyn SecuredAction, context: &Cx) -> Result<Decision> {
        self.authorize(&obj.resource(), obj.permission(), context)
            .await
    }

    async fn check_required(&self, obj: &dyn SecuredAction, context: &Cx) -> Result<()> {
        match self.check(obj, context).await? {
            Decision::Allow => Ok(()),
            Decision::Deny => Err(Error::NotAllowed),
        }
    }

    /// Check if the policy allows the action.
    ///
    /// Specifically, this method should return [`Decision::Allow`] if the context
    /// is granted the requested permission on the resource, and [`Decision::Deny`] otherwise.
    async fn authorize(
        &self,
        resource: &ResourceIdent,
        permission: &Permission,
        context: &Cx,
    ) -> Result<Decision>;

    async fn authorize_many(
        &self,
        resources: &[ResourceIdent],
        permission: &Permission,
        context: &Cx,
    ) -> Result<Vec<Decision>> {
        let mut decisions = Vec::with_capacity(resources.len());
        for resource in resources {
            decisions.push(self.authorize(resource, permission, context).await?);
        }
        Ok(decisions)
    }

    /// Check if the policy allows the action, and return an error if denied.
    async fn authorize_checked(
        &self,
        resource: &ResourceIdent,
        permission: &Permission,
        context: &Cx,
    ) -> Result<()> {
        match self.authorize(resource, permission, context).await? {
            Decision::Allow => Ok(()),
            Decision::Deny => Err(Error::NotAllowed),
        }
    }
}

pub trait ProvidesPolicy<Cx: Send + Sync + 'static>: Send + Sync + 'static {
    fn policy(&self) -> &Arc<dyn Policy<Cx>>;
}

#[async_trait::async_trait]
impl<T: Policy<Cx>, Cx: Send + Sync + 'static> Policy<Cx> for Arc<T> {
    async fn authorize(
        &self,
        resource: &ResourceIdent,
        permission: &Permission,
        context: &Cx,
    ) -> Result<Decision> {
        T::authorize(self, resource, permission, context).await
    }

    async fn authorize_many(
        &self,
        resources: &[ResourceIdent],
        permission: &Permission,
        context: &Cx,
    ) -> Result<Vec<Decision>> {
        T::authorize_many(self, resources, permission, context).await
    }
}

/// Checks if the context has the given permission for each resource,
/// and retains only those that receive an allow decision.
pub async fn process_resources<
    T: Policy<Cx> + Sized,
    Cx: Send + Sync + 'static,
    R: ResourceExt + Send,
>(
    handler: &T,
    context: &Cx,
    permission: &Permission,
    resources: &mut Vec<R>,
) -> Result<()> {
    filter_authorized(handler, context, permission, resources).await
}

/// [`process_resources`] for a `dyn`-typed policy.
///
/// Identical filtering behavior, but takes `&dyn Policy<Cx>` so it can be used
/// with an `Arc<dyn Policy<Cx>>` (which does not satisfy the `Sized` bound on
/// [`process_resources`]). Handler patterns that hold the policy behind a trait
/// object — e.g. proxy/decorator handlers — use this.
pub async fn filter_authorized<Cx: Send + Sync + 'static, R: ResourceExt + Send>(
    policy: &dyn Policy<Cx>,
    context: &Cx,
    permission: &Permission,
    resources: &mut Vec<R>,
) -> Result<()> {
    let res = resources.iter().map(|r| r.into()).collect::<Vec<_>>();
    let mut decisions = policy.authorize_many(&res, permission, context).await?;
    resources.retain(|_| decisions.pop() == Some(Decision::Allow));
    Ok(())
}
