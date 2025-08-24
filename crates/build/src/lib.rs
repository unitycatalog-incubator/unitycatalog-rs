pub use error::*;

mod analysis;
pub mod codegen;
pub mod error;
pub mod parsing;
mod utils;

mod google {
    pub mod api {
        include!("./gen/google.api.rs");
    }
}

mod gnostic {
    pub mod openapi {
        pub mod v3 {
            include!("./gen/gnostic.openapi.v3.rs");
        }
    }
}
