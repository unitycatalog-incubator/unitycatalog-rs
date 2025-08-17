# unitycatalog-rs

> [!WARNING]
> This is a thus far unofficial and experimental implementation of the Unity Catalog APIs.
> This project is in early development and is not yet ready for production use.

An experimental implementation of [Unity Catalog] in Rust.

The reference implementation of unity catalog aims to be a fully self-contained
implememntaion of the Unity Catalog API. While this implementation also
provides a standalone server the primary goal is to provide a toolset that
allows integrators to customize the implementation to their needs or partially
adopt specific API surfaces.

The design focuses on abstracting the unity catalog specific behaviors as much
as possible, while allowing cosutomization of all deployment specific
aspects.

[Unity Catalog]: unitycatalog.io

## What's in the repository

The bulk of the implementation is within the rust crates inside the `crates` directory.

There are two crates used for internal purposes and not meant for external use
* [`unitycatalog-build`](crates/build/) - generate rust code from protobuf definitions.
* [`unitycatalog-derive`](crates/derive/) - derive macros for internal use.

The core logic is implemented in the [`unitycatalog-common`](crates/common/) crate.
There are further extension crates to build more advanced serves.
* [`unitycatalog-postgres`](crates/postgres/) - use postgres as a backend for the unity catalog.

The simples way to get started with `unitycatalog-rs` is to use the [`unitycatalog-cli`](crates/cli/).
This exposes a commandline interface to run a unoty catalog server or query a service.

### Python bindings

Two python wheels are build in the project.
* [python bindings](python/client/) for the unity catalog client
* `uc` cli is a binary only distribution to install the unitycatalog cli via pip

### Node bindings

* [node bindings](node/client/) for the unity catalog client

## Development

We use [`just`](https://just.systems/man/en/) as the command runner for development tasks.
Have a look at the [`justfile`](justfile) to see all available tasks.