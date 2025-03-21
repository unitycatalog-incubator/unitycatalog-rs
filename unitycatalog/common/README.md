# unitycatalog-common

Common types and utilities for the Unity Catalog API.

Since we rely heavily on macros / code generation, the common crate contains
utilities for both server and client implementations. To handle the different
needs to downstream crates we introduce a number of feature flags that can be
used to enable or disable certain parts of the crate.