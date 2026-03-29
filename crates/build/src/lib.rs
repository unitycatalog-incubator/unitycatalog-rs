pub use error::*;

mod analysis;
pub mod codegen;
pub mod error;
pub mod openapi_enrich;
pub mod output;
pub mod parsing;
mod utils;

pub use codegen::{CodeGenConfig, CodeGenOutput};

mod google {
    #[allow(unused)]
    pub mod api {
        include!("./gen/google.api.rs");
    }
}

mod gnostic {
    pub mod openapi {
        #[allow(unused)]
        pub mod v3 {
            include!("./gen/gnostic.openapi.v3.rs");
        }
    }
}
