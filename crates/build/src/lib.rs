pub mod google {
    pub mod api {
        include!("./gen/google.api.rs");
    }
}

pub mod gnostic {
    pub mod openapi {
        pub mod v3 {
            include!("./gen/gnostic.openapi.v3.rs");
        }
    }
}
