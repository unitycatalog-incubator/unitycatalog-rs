# Authorization Framework Review: unitycatalog-rs

> **Status:** Architecture is sound for Cedar/ABAC integration; one structural change required to fully unlock extensibility.
> This document reviews the policy enforcement and decision separation, identifies design gaps, and provides a
> concrete Cedar policy language reference implementation to validate the model against the Databricks Unity Catalog
> permission model.

---

## Goal

The desired architecture is ABAC (Attribute-Based Access Control) where:

- This codebase handles **policy enforcement** — determining *when* and *which* permission checks are made
- An external implementor handles **policy decisions** — determining *whether* a given action is permitted
- A user of our crates defines a custom `RequestContext` (e.g., carrying group memberships, workspace ID, IP address)
- That context is populated from HTTP headers by a custom `Authenticator`
- A custom `Policy<Cx>` impl (e.g., backed by Cedar) evaluates the decision using the rich context

---

## Current Architecture

### Key Types

| Type | Location | Role |
|---|---|---|
| `Principal` | `crates/server/src/policy/mod.rs:20` | Who is making the request (`Anonymous \| User(String) \| Custom(Bytes)`) |
| `RequestContext` | `crates/server/src/api/mod.rs:56` | Default `Cx` type; wraps `Principal` |
| `Permission` | `crates/server/src/policy/mod.rs:44` | What is requested (`Read \| Write \| Manage \| Create \| Use \| Browse \| Select`) |
| `ResourceIdent` | `crates/common/src/models/resources/mod.rs:64` | What resource is acted on (typed enum over `ResourceRef`) |
| `ResourceRef` | `crates/common/src/models/resources/mod.rs:11` | How a resource is identified (`Uuid \| Name \| Undefined`) |
| `Decision` | `crates/server/src/policy/mod.rs:62` | `Allow \| Deny` |

### Key Traits

| Trait | Location | Role |
|---|---|---|
| `Authenticator` | `crates/server/src/rest/auth.rs:13` | Converts `HTTP Request → Principal` (runs in middleware) |
| `Policy<Cx>` | `crates/server/src/policy/mod.rs:71` | Policy decision point: `authorize(resource, permission, &Cx) → Decision` |
| `ProvidesPolicy<Cx>` | `crates/server/src/policy/mod.rs:122` | Exposes `Arc<dyn Policy<Cx>>` from a handler struct |
| `SecuredAction` | `crates/server/src/api/mod.rs:73` | Policy enforcement point: `resource() + permission()` on each request type |

### Request Flow

```
HTTP Request
  ↓
AuthenticationMiddleware
  Authenticator::authenticate() → Principal
  req.extensions_mut().insert(principal)
  ↓
Axum route handler: axum extracts Cx via FromRequestParts
  RequestContext::from_request_parts() → reads Principal from extensions
  ↓
Handler method called with (request: XRequest, context: Cx)
  ↓
handler.check_required(&request, &context)          ← POLICY ENFORCEMENT POINT
  SecuredAction::resource() → ResourceIdent
  SecuredAction::permission() → &Permission
  Policy::authorize(resource, permission, &Cx)       ← POLICY DECISION POINT
  If Deny → Err(Error::NotAllowed)
  If Allow → continue
  ↓
(For list operations) process_resources(handler, &context, permission, &mut resources)
  authorize_many([resource1, resource2, ...], permission, &Cx)
  retain only Allow'd resources
  ↓
Handler performs storage operation and returns response
```

### Handler Implementation Pattern

All resource handlers are blanket impls hardcoded to `RequestContext`:

```rust
// crates/server/src/api/catalogs.rs:13
impl<T: ResourceStore + Policy<RequestContext>> CatalogHandler<RequestContext> for T {
    async fn create_catalog(&self, request: CreateCatalogRequest, context: RequestContext) -> Result<Catalog> {
        self.check_required(&request, &context).await?;
        // ... storage logic
    }
}
```

### Existing Policy Implementations

| Implementation | Location | Behavior |
|---|---|---|
| `ConstantPolicy` | `crates/server/src/policy/constant.rs` | Always returns a fixed `Decision` (default: `Allow`) |
| `AnonymousAuthenticator` | `crates/server/src/rest/auth.rs:32` | Always returns `Principal::Anonymous` |

---

## Strengths: What the Architecture Gets Right

### 1. Clean PDP/PEP Separation

`SecuredAction` (enforcement: declares what resource and permission a request requires) and
`Policy<Cx>` (decision: evaluates whether the principal can perform that action) are clearly
distinct. Enforcement points are placed at handler entry before any side effects — consistent
across all 10 handler modules.

### 2. `Cx` Type Parameter is Genuinely Generic

Generated handler traits are `Handler<Cx = RequestContext>`, which means downstream crates
can substitute their own context type with custom ABAC fields. The code generation pipeline
makes `context_type_path` configurable (`crates/build/src/codegen/mod.rs:102`).

### 3. N+1 Prevention via `authorize_many`

`process_resources()` (`policy/mod.rs:149`) batches all list authorization into a single
`authorize_many()` call, avoiding per-resource round-trips to a policy engine. This is critical
for performance when filtering large lists.

### 4. `ResourceRef::Undefined` for Wildcard Pre-checks

List operations check a wildcard resource first (e.g., `ResourceIdent::catalog(ResourceRef::Undefined)`)
before fetching any rows. This implements the correct "can the caller list at all?" gate
without fetching data unnecessarily.

### 5. `Arc<dyn Policy<Cx>>` Injection Point

`ServerHandler` holds `Arc<dyn Policy<Cx>>`, making the policy implementation fully swappable
at construction time without touching any framework code. `ConstantPolicy` serves as both
the default and a useful testing baseline.

---

## Design Issues and Gaps

### Issue 1: `SecuredAction::permission()` returns `&'static Permission`

**Location:** `crates/server/src/api/mod.rs:78`

**Problem:** The `'static` lifetime forces every `SecuredAction` impl to return a reference
to a compile-time constant. This prevents permission derivation from request fields. While no
current handlers need dynamic permissions, this unnecessarily constrains future request types
(e.g., a credential operation that should require `Manage` only if the credential is shared,
`Write` otherwise).

**Impact:** Low right now, but a footgun for future request types.

**Recommendation:**
```rust
// Before
fn permission(&self) -> &'static Permission;

// After — backwards compatible; existing impls that return &STATIC_PERM still compile
fn permission(&self) -> &Permission;
```

---

### Issue 2: Handler Blanket Impls Hardcode `RequestContext` — Blocks ABAC

**Location:** `crates/server/src/api/catalogs.rs:13` (and all other api/*.rs files)

**Problem:** This is the most significant architectural gap. All blanket impls are:

```rust
impl<T: ResourceStore + Policy<RequestContext>> CatalogHandler<RequestContext> for T { ... }
```

A downstream crate wanting ABAC with custom context (e.g., group memberships, workspace ID)
cannot reuse these impls. They must either:
1. Fork the entire handler module, or
2. Wrap `RequestContext` and lose type safety in the `Cx` parameter

The `Cx` type parameter in the generated trait signature is correct, but the hand-written
blanket impls defeat the purpose by concretizing it.

**Recommendation:** Make all blanket impls generic over `Cx` with the constraint that the
handler actually needs:

```rust
impl<T, Cx> CatalogHandler<Cx> for T
where
    T: ResourceStore + Policy<Cx>,
    Cx: AsRef<Principal> + Send + Sync + 'static,
{
    async fn create_catalog(&self, request: CreateCatalogRequest, context: Cx) -> Result<Catalog> {
        self.check_required(&request, &context).await?;
        // ... unchanged
    }
}
```

The only use of `context` in most handler methods is passing it to `check_required()` and
`process_resources()` — both of which already operate on `&Cx`. The `AsRef<Principal>` bound
covers the audit trail TODO (recording the actor's identity). This change requires no logic
changes to the handlers themselves.

**Files affected:** `catalogs.rs`, `schemas.rs`, `tables.rs`, `volumes.rs`, `functions.rs`,
`credentials.rs`, `external_locations.rs`, `recipients.rs`, `shares.rs`, `temporary_credentials.rs`

---

### Issue 3: `Policy<Cx>` Receives No Action Metadata Beyond `Permission`

**Location:** `crates/server/src/policy/mod.rs:88`

**Problem:** The `authorize()` signature is:

```rust
async fn authorize(&self, resource: &ResourceIdent, permission: &Permission, context: &Cx) -> Result<Decision>;
```

Cedar and OPA-style ABAC policies work at the level of named actions (e.g., `Action::"CreateCatalog"`,
`Action::"DeleteTable"`). The Databricks Unity Catalog permission model has dozens of distinct
privilege types. Collapsing these into 7 `Permission` variants (`Read`, `Write`, `Manage`, etc.)
means Cedar policies cannot distinguish "create a catalog" from "create a table" — both are just
`Permission::Create` on a `Catalog` or `Table` resource.

A `CedarPolicy` impl must reconstruct the action name from `(ResourceIdent variant, Permission)`:

```rust
let action_name = match (resource, permission) {
    (ResourceIdent::Catalog(_), Permission::Create) => "CreateCatalog",
    (ResourceIdent::Table(_), Permission::Read) => "ReadTable",
    // ... n × m combinations
};
```

This is fragile, loses information, and couples the Cedar impl to the internal structure of
`ResourceIdent` and `Permission`.

**Recommendation:** Add an `action_type()` method to `SecuredAction` with a default impl
(non-breaking):

```rust
pub trait SecuredAction: Send + Sync {
    fn resource(&self) -> ResourceIdent;
    fn permission(&self) -> &Permission;

    /// The action type string, used by Cedar/OPA policy engines to match on named actions.
    /// Defaults to "unknown". Production implementations should return a stable action name
    /// (e.g., "CreateCatalog", "ListTables", "DeleteSchema").
    fn action_type(&self) -> &'static str { "unknown" }
}
```

And expose it to the policy via `check()`:

```rust
async fn check(&self, obj: &dyn SecuredAction, context: &Cx) -> Result<Decision> {
    self.authorize_action(obj.resource(), obj.permission(), obj.action_type(), context).await
}
```

Or simply expose the full `&dyn SecuredAction` to `authorize()` (richer but slightly more
coupled).

---

### Issue 4: `Decision::Deny` is Opaque — No Reason or Audit Hook

**Location:** `crates/server/src/policy/mod.rs:60`

**Problem:** `check_required()` returns `Err(Error::NotAllowed)` on any `Deny` with no
additional context. There is no way to:

1. Log *why* the policy denied the request (for debugging or audit)
2. Distinguish a policy denial from a policy engine error
3. Provide structured denial metadata (e.g., Cedar returns `diagnostics` with matching/failing policy IDs)

**Recommendation:** Add optional reason to `Deny`:

```rust
#[derive(Debug, Clone)]
pub enum Decision {
    Allow,
    Deny { reason: Option<String> },
}
```

The reason should be logged internally (e.g., via `tracing::debug!`) before `check_required()`
returns `Err(Error::NotAllowed)` — it must **not** be surfaced in the HTTP response (information
leakage risk). Cedar's `Response::diagnostics()` can populate this field.

---

### Issue 5: `authorize_many` Default Implementation is Serial

**Location:** `crates/server/src/policy/mod.rs:95`

**Problem:** The default `authorize_many` loops over resources calling `authorize()` one at a
time. For any policy engine with per-call overhead (Cedar evaluation, remote OPA call, DB query),
this is O(n) latency on list operations with large result sets.

Cedar's Rust API evaluates each `Request` independently, but a production implementation can
pre-compile the policy set once and evaluate all requests in a tight loop without per-call
overhead. An external policy engine (OPA, Rego) would want to batch all requests into a single
HTTP call.

**Recommendation:** Add a doc comment to `authorize_many` making clear that implementors
should override it for production use:

```rust
/// Check authorization for multiple resources in a single call.
///
/// The default implementation calls [`authorize`] sequentially for each resource.
/// **Implementors of production [`Policy`] types should override this method** to
/// take advantage of batch evaluation (e.g., a single Cedar policy evaluation pass,
/// a single OPA bulk query, or a single DB lookup).
async fn authorize_many(...) -> Result<Vec<Decision>> { ... }
```

---

### Issue 6: `Principal::Custom(Bytes)` Overlaps with the `Cx` Extension Mechanism

**Location:** `crates/server/src/policy/mod.rs:24`

**Problem:** The codebase has two independent extension mechanisms for rich principal data:

1. `Principal::Custom(Bytes)` — embed arbitrary bytes in the principal identity
2. Generic `Cx` type parameter — carry rich typed data alongside the principal

These serve the same purpose (ABAC attributes) but are incompatible. A `CedarPolicy<Cx>` impl
that receives `Principal::Custom(bytes)` has no type-safe way to deserialize those bytes.
Conversely, a caller using `Cx` to carry group memberships doesn't need `Custom` at all.

**Recommendation:** Keep `Custom(Bytes)` for legacy/opaque use cases (e.g., forwarding an
existing bearer token to an upstream system) but explicitly document that ABAC attribute
injection should use the `Cx` type parameter, not `Custom`. Long-term, `Custom` may be
removable once use cases are clearer.

---

### Issue 7: `ProvidesPolicy<Cx>` is Vestigial

**Location:** `crates/server/src/policy/mod.rs:122`

**Problem:** The trait `ProvidesPolicy<Cx>` exposes `fn policy(&self) -> &Arc<dyn Policy<Cx>>`,
but no handler impl calls `self.policy().authorize(...)` — they call `self.authorize(...)` via
the `T: Policy<Cx>` direct bound. `ServerHandler` implements `Policy<Cx>` by forwarding to its
internal `Arc<dyn Policy<Cx>>`, bypassing `ProvidesPolicy`.

**Recommendation:** Either:
- Remove `ProvidesPolicy<Cx>` as dead code, or
- Use it consistently: change handler bounds to `T: ResourceStore + ProvidesPolicy<Cx>` and
  have `check_required` call `self.policy().check_required(...)`, which would allow middleware
  wrapping of the policy without wrapping the whole handler

---

## Cedar Policy Language: Reference Implementation

Cedar is a formal, open-source authorization language with a Rust crate (`cedar-policy = "4"`).
Its entity-action-resource model maps directly to the `Policy<Cx>` abstraction.

### Conceptual Mapping

| Cedar concept | unitycatalog-rs concept |
|---|---|
| `Principal` entity | `Principal` enum + ABAC attributes from `Cx` |
| `Action` entity | `(ResourceIdent variant, Permission)` — or `action_type()` if added |
| `Resource` entity | `ResourceIdent` |
| `Context` record | Additional fields from `Cx` (e.g., IP address, request time) |
| `Decision` | `Decision` enum |
| `PolicySet` | Loaded from files, DB, or environment config |
| `Entities` | Users, groups, resource ownership — loaded from UC store or identity provider |

### Cedar Schema (Simplified UC Permission Model)

```cedar
namespace UnityCatalog {
  entity User = {
    groups: Set<String>,
    workspace_id?: String,
  };
  entity Group;

  entity Catalog = {
    owner?: String,
    workspace_id?: String,
  };
  entity Schema in [Catalog];
  entity Table  in [Schema];
  entity Volume in [Schema];

  action CreateCatalog   appliesTo { principal: [User, Group], resource: [Catalog] };
  action ReadCatalog     appliesTo { principal: [User, Group], resource: [Catalog] };
  action ManageCatalog   appliesTo { principal: [User, Group], resource: [Catalog] };
  action CreateSchema    appliesTo { principal: [User, Group], resource: [Schema] };
  action ReadSchema      appliesTo { principal: [User, Group], resource: [Schema] };
  action CreateTable     appliesTo { principal: [User, Group], resource: [Table] };
  action ReadTable       appliesTo { principal: [User, Group], resource: [Table] };
  action WriteTable      appliesTo { principal: [User, Group], resource: [Table] };
  action ManageTable     appliesTo { principal: [User, Group], resource: [Table] };
}
```

### Example Cedar Policies

```cedar
// Admins can do anything
permit(
  principal in UnityCatalog::Group::"admins",
  action,
  resource
);

// Catalog owners can manage their own catalogs
permit(
  principal,
  action == UnityCatalog::Action::"ManageCatalog",
  resource
) when {
  resource.owner == principal.id
};

// "data-readers" group can read all catalogs and schemas
permit(
  principal in UnityCatalog::Group::"data-readers",
  action in [UnityCatalog::Action::"ReadCatalog", UnityCatalog::Action::"ReadSchema"],
  resource
);

// Workspace isolation: users in workspace A cannot access workspace B resources
forbid(
  principal,
  action,
  resource
) when {
  principal.workspace_id != resource.workspace_id
};

// Default deny is implicit in Cedar (no matching permit = deny)
```

### Rust `Policy<Cx>` Implementation

```rust
// crates/server/src/policy/cedar.rs  — not yet implemented; reference design only
use cedar_policy::{
    Authorizer, Context, Decision as CedarDecision, Entities,
    EntityUid, PolicySet, Request,
};
use unitycatalog_common::models::ResourceIdent;

use super::{Decision, Permission, Policy, Principal};
use crate::api::RequestContext;
use crate::Result;

/// Cedar-backed policy implementation.
///
/// # Entity store
/// The `entities` parameter should contain all principals (users, groups, group memberships)
/// and resources (with owner attributes) needed to evaluate policies. In production this
/// would be loaded from the Unity Catalog resource store or an external identity provider.
pub struct CedarPolicy {
    authorizer: Authorizer,
    policies: PolicySet,
    entities: Entities,
}

impl CedarPolicy {
    pub fn new(policies: PolicySet, entities: Entities) -> Self {
        Self { authorizer: Authorizer::new(), policies, entities }
    }
}

#[async_trait::async_trait]
impl Policy<RequestContext> for CedarPolicy {
    async fn authorize(
        &self,
        resource: &ResourceIdent,
        permission: &Permission,
        context: &RequestContext,
    ) -> Result<Decision> {
        let principal_eid: EntityUid = match &context.recipient {
            Principal::User(name) =>
                format!("UnityCatalog::User::\"{}\"", name).parse()
                    .map_err(|e| crate::Error::Internal(format!("{e}")))?,
            Principal::Anonymous =>
                "UnityCatalog::User::\"anonymous\"".parse()
                    .map_err(|e| crate::Error::Internal(format!("{e}")))?,
            Principal::Custom(_) => {
                // Custom principals are not supported by this Cedar implementation.
                // Use a typed Cx with group data instead.
                return Ok(Decision::Deny { reason: Some("unsupported principal type".into()) });
            }
        };

        let action_eid = derive_cedar_action(resource, permission);
        let resource_eid = derive_cedar_resource(resource);

        let request = Request::new(
            Some(principal_eid),
            Some(action_eid),
            Some(resource_eid),
            Context::empty(),
            None, // schema-based validation is optional but recommended
        ).map_err(|e| crate::Error::Internal(format!("{e}")))?;

        let response = self.authorizer.is_authorized(&request, &self.policies, &self.entities);

        // Log denial diagnostics without exposing them to the HTTP caller
        if response.decision() == CedarDecision::Deny {
            let diag = response.diagnostics();
            tracing::debug!(
                ?diag,
                "Cedar policy denied request"
            );
        }

        Ok(match response.decision() {
            CedarDecision::Allow => Decision::Allow,
            CedarDecision::Deny  => Decision::Deny { reason: None },
        })
    }

    /// Override to evaluate all resources in a single Cedar authorization pass.
    async fn authorize_many(
        &self,
        resources: &[ResourceIdent],
        permission: &Permission,
        context: &RequestContext,
    ) -> Result<Vec<Decision>> {
        // Cedar evaluates each request independently but shares compiled policy set —
        // this is efficient; no per-request policy compilation overhead.
        let mut decisions = Vec::with_capacity(resources.len());
        for resource in resources {
            decisions.push(self.authorize(resource, permission, context).await?);
        }
        Ok(decisions)
    }
}

fn derive_cedar_action(resource: &ResourceIdent, permission: &Permission) -> EntityUid {
    let resource_type = match resource {
        ResourceIdent::Catalog(_)          => "Catalog",
        ResourceIdent::Schema(_)           => "Schema",
        ResourceIdent::Table(_)            => "Table",
        ResourceIdent::Volume(_)           => "Volume",
        ResourceIdent::Function(_)         => "Function",
        ResourceIdent::Share(_)            => "Share",
        ResourceIdent::Credential(_)       => "Credential",
        ResourceIdent::ExternalLocation(_) => "ExternalLocation",
        ResourceIdent::Recipient(_)        => "Recipient",
        ResourceIdent::Column(_)           => "Column",
    };
    let perm = match permission {
        Permission::Read   => "Read",
        Permission::Write  => "Write",
        Permission::Manage => "Manage",
        Permission::Create => "Create",
        Permission::Use    => "Use",
        Permission::Browse => "Browse",
        Permission::Select => "Select",
    };
    format!("UnityCatalog::Action::\"{}{}\"", perm, resource_type)
        .parse()
        .expect("valid Cedar entity UID")
}

fn derive_cedar_resource(resource: &ResourceIdent) -> EntityUid {
    use unitycatalog_common::models::ResourceRef;
    let (type_name, id) = match resource {
        ResourceIdent::Catalog(r)          => ("UnityCatalog::Catalog", r),
        ResourceIdent::Schema(r)           => ("UnityCatalog::Schema", r),
        ResourceIdent::Table(r)            => ("UnityCatalog::Table", r),
        ResourceIdent::Volume(r)           => ("UnityCatalog::Volume", r),
        ResourceIdent::Function(r)         => ("UnityCatalog::Function", r),
        ResourceIdent::Share(r)            => ("UnityCatalog::Share", r),
        ResourceIdent::Credential(r)       => ("UnityCatalog::Credential", r),
        ResourceIdent::ExternalLocation(r) => ("UnityCatalog::ExternalLocation", r),
        ResourceIdent::Recipient(r)        => ("UnityCatalog::Recipient", r),
        ResourceIdent::Column(r)           => ("UnityCatalog::Column", r),
    };
    let id_str = match id {
        ResourceRef::Name(name) => name.to_string(),
        ResourceRef::Uuid(u)    => u.to_string(),
        ResourceRef::Undefined  => "*".to_string(),
    };
    format!("{}::\"{}\"", type_name, id_str)
        .parse()
        .expect("valid Cedar entity UID")
}
```

### ABAC with Custom Context

To enable group-based ABAC without changing handler code, define a custom context:

```rust
/// Custom context carrying ABAC attributes for Cedar policy evaluation.
#[derive(Debug, Clone)]
pub struct AbacContext {
    pub recipient: Principal,
    pub groups: Vec<String>,
    pub workspace_id: Option<String>,
    pub source_ip: Option<std::net::IpAddr>,
}

impl AsRef<Principal> for AbacContext {
    fn as_ref(&self) -> &Principal { &self.recipient }
}

impl<S: Send + Sync> axum::extract::FromRequestParts<S> for AbacContext {
    type Rejection = std::convert::Infallible;
    async fn from_request_parts(parts: &mut axum::http::request::Parts, _: &S) -> Result<Self, Self::Rejection> {
        let recipient = parts.extensions.get::<Principal>().cloned().unwrap_or_else(Principal::anonymous);
        // Extract groups from X-Forwarded-Groups header (set by upstream proxy)
        let groups = parts
            .headers
            .get("x-forwarded-groups")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.split(',').map(str::to_string).collect())
            .unwrap_or_default();
        Ok(AbacContext { recipient, groups, workspace_id: None, source_ip: None })
    }
}

// This works without any changes to handler code — IF blanket impls are generic over Cx:
impl<T, Cx> CatalogHandler<Cx> for T
where
    T: ResourceStore + Policy<Cx>,
    Cx: AsRef<Principal> + Send + Sync + 'static,
{ ... }
```

### Soundness Validation Summary

| Requirement | Status | Notes |
|---|---|---|
| `ResourceIdent` → Cedar entity UID | ✅ | Straightforward string formatting |
| `Permission` × `ResourceIdent` → Cedar action | ✅ | n×m mapping; works but loses action name fidelity |
| `RequestContext` → Cedar principal | ✅ | `User(name)` → `UnityCatalog::User::"name"` |
| Custom `Cx` with ABAC attributes | ⚠️ | Requires blanket impl fix (Issue 2) |
| `authorize_many` batch evaluation | ✅ | Can be overridden; Cedar reuses compiled policies |
| Denial diagnostics | ⚠️ | Cedar provides them; `Decision` needs `reason` field (Issue 4) |
| Action name fidelity | ⚠️ | Lost without `action_type()` on `SecuredAction` (Issue 3) |

---

## Recommended Changes (Prioritized)

### P0 — Required for ABAC Extensibility

**1. Make handler blanket impls generic over `Cx`**

All files in `crates/server/src/api/`:

```rust
// Before (all api/*.rs files)
impl<T: ResourceStore + Policy<RequestContext>> CatalogHandler<RequestContext> for T { ... }

// After
impl<T, Cx> CatalogHandler<Cx> for T
where
    T: ResourceStore + Policy<Cx>,
    Cx: AsRef<Principal> + Send + Sync + 'static,
{ ... }
```

This is the single highest-leverage change. It unblocks Cedar/ABAC integration without any
handler logic changes.

**2. Remove `'static` from `SecuredAction::permission()`** (`crates/server/src/api/mod.rs:78`)

```rust
fn permission(&self) -> &Permission;   // was: &'static Permission
```

All existing impls returning `&STATIC_CONST` still compile. The `'static` provides no benefit.

### P1 — Recommended Design Improvements

**3. Add `action_type()` to `SecuredAction`** with a default impl (non-breaking)

```rust
pub trait SecuredAction: Send + Sync {
    fn resource(&self) -> ResourceIdent;
    fn permission(&self) -> &Permission;
    fn action_type(&self) -> &'static str { "unknown" }
}
```

Implement in each request type: `"CreateCatalog"`, `"ListTables"`, `"DeleteSchema"`, etc.

**4. Add denial reason to `Decision`**

```rust
pub enum Decision {
    Allow,
    Deny { reason: Option<String> },
}
```

Log reason via `tracing::debug!` in `check_required()`. Never surface to HTTP response.

**5. Clarify or remove `ProvidesPolicy<Cx>`** (`policy/mod.rs:122`)

The trait is currently unused in dispatch. Either remove it or make it the canonical dispatch
path and remove the `T: Policy<Cx>` direct bound from handler impls.

**6. Document `Principal::Custom(Bytes)` vs `Cx`**

Add a doc comment clarifying that `Custom` is for opaque principal forwarding and that ABAC
attribute injection should use the `Cx` type parameter.

### P2 — Documentation

**7. Module-level doc comment in `crates/server/src/policy/mod.rs`** explaining:
- The PDP/PEP architectural split
- How to implement a custom `Policy<Cx>`
- How to define and use a custom `Cx` type
- That `authorize_many` should be overridden for production policy engines

**8. Cedar integration example** in `crates/server/src/policy/cedar.rs` (or `examples/`) as a
reference `Policy<Cx>` implementation, validated with `cargo check`.

---

## Summary

| Dimension | Rating | Finding |
|---|---|---|
| **PEP/PDP separation** | ✅ Sound | `SecuredAction` and `Policy<Cx>` are cleanly distinct; enforcement is consistent |
| **Cx extensibility (traits)** | ✅ Sound | Generated traits correctly parameterize over `Cx` |
| **Cx extensibility (impls)** | ⚠️ Blocked | Blanket impls hardcode `RequestContext` — one structural fix required |
| **Cedar/ABAC soundness** | ✅ Sound | `Policy<Cx>` maps directly to Cedar's request model |
| **Action fidelity** | ⚠️ Coarse | 7 `Permission` variants lose UC action semantics; need `action_type()` |
| **Principal design** | ⚠️ Overlapping | `Custom(Bytes)` and generic `Cx` serve the same purpose |
| **Batch authorization** | ✅ Present | `authorize_many` hook exists; default is serial (document override) |
| **Denial observability** | ❌ Opaque | `Decision::Deny` carries no reason; no audit hook |

**Verdict:** The model is architecturally sound. Cedar integration is feasible with one structural
change (generic blanket impls) and three additive improvements (`action_type()`, `Deny::reason`,
`authorize_many` override documentation). No breaking changes to external API consumers are required.
