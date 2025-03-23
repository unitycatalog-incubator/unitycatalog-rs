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
