# Contribution Guide

## Getting Started

### Prerequisites

- Rust toolchain ([install instructions](https://www.rust-lang.org/tools/install))
- buf ([install instructions](https://buf.build/docs/installation))
- just ([install instructions](https://just.systems/man/en/))

## Generated Code

We heavily rely on code generation to ensure consistency with the API spec and to reduce the maintenance burden.
The most important components involved in our code generation are:

- The `protobuf` definitions which define the API surface.
- [`buf.gen.yaml`](buf.gen.yaml) which defines the code we generate using `buf`
- the [`build`](crates/build) crate which holds custom generation logic
- the [`derive`](crates/derive) crate which holds custom derive macros

THe unitycatalog API is specified as a REST API, however we maintain the API definitions in
protobuf to make use of the (IMO :)) more flexible and powerful code generation capabilities
and better ease of maintenance (legibility). To maintain a proper mapping of how the protobuf
messages / services should be mapped to REST API endpoints, we annotate the definitions
with [`google.api.http`](https://github.com/googleapis/googleapis/blob/master/google/api/http.proto)
and [`gnostic`](https://github.com/google/gnostic) options.

These annotations are used by both the `buf` compiler to generate e.g. OpenAPI specifications
and by our custom code to provide boilerplate implementations for server and client implementations.

have a look at

```sh
just generate
```

to see the exact sequence of steps performed to generate our code.
