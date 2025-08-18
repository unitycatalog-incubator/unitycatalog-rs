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

### Addind new resources

UC manages resources at the root level. To add a new resource / API surface,
the following steps are required:

1. Define the resource in the protobuf definitions.
2. Annotate the resource with `google.api.http` and `gnostic` options.
3. Generate the common models using `just generate-proto`
4. Extend the `unitycatalog_common::models` module to include the newly generated files.
5. Run `just generate-code` to generate the server and client implementations.

Once the boilerplate code is generated, you can start implementing the resource logic.

For the client:
- Implement the higher level client in the `unitycatalog_client` crate.
- Implement the new client in the python bindings.
  - Don't forget to update the python type definitions.