// @generated
impl serde::Serialize for Resource {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.resource.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.internal.Resource", len)?;
        if let Some(v) = self.resource.as_ref() {
            match v {
                resource::Resource::Share(v) => {
                    struct_ser.serialize_field("share", v)?;
                }
                resource::Resource::Credential(v) => {
                    struct_ser.serialize_field("credential", v)?;
                }
                resource::Resource::Catalog(v) => {
                    struct_ser.serialize_field("catalog", v)?;
                }
                resource::Resource::Schema(v) => {
                    struct_ser.serialize_field("schema", v)?;
                }
                resource::Resource::Table(v) => {
                    struct_ser.serialize_field("table", v)?;
                }
                resource::Resource::Column(v) => {
                    struct_ser.serialize_field("column", v)?;
                }
                resource::Resource::ExternalLocation(v) => {
                    struct_ser.serialize_field("external_location", v)?;
                }
                resource::Resource::Recipient(v) => {
                    struct_ser.serialize_field("recipient", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Resource {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "share",
            "credential",
            "catalog",
            "schema",
            "table",
            "column",
            "external_location",
            "externalLocation",
            "recipient",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Share,
            Credential,
            Catalog,
            Schema,
            Table,
            Column,
            ExternalLocation,
            Recipient,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(
                        &self,
                        formatter: &mut std::fmt::Formatter<'_>,
                    ) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "share" => Ok(GeneratedField::Share),
                            "credential" => Ok(GeneratedField::Credential),
                            "catalog" => Ok(GeneratedField::Catalog),
                            "schema" => Ok(GeneratedField::Schema),
                            "table" => Ok(GeneratedField::Table),
                            "column" => Ok(GeneratedField::Column),
                            "externalLocation" | "external_location" => {
                                Ok(GeneratedField::ExternalLocation)
                            }
                            "recipient" => Ok(GeneratedField::Recipient),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Resource;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.internal.Resource")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Resource, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut resource__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Share => {
                            if resource__.is_some() {
                                return Err(serde::de::Error::duplicate_field("share"));
                            }
                            resource__ = map_
                                .next_value::<::std::option::Option<_>>()?
                                .map(resource::Resource::Share);
                        }
                        GeneratedField::Credential => {
                            if resource__.is_some() {
                                return Err(serde::de::Error::duplicate_field("credential"));
                            }
                            resource__ = map_
                                .next_value::<::std::option::Option<_>>()?
                                .map(resource::Resource::Credential);
                        }
                        GeneratedField::Catalog => {
                            if resource__.is_some() {
                                return Err(serde::de::Error::duplicate_field("catalog"));
                            }
                            resource__ = map_
                                .next_value::<::std::option::Option<_>>()?
                                .map(resource::Resource::Catalog);
                        }
                        GeneratedField::Schema => {
                            if resource__.is_some() {
                                return Err(serde::de::Error::duplicate_field("schema"));
                            }
                            resource__ = map_
                                .next_value::<::std::option::Option<_>>()?
                                .map(resource::Resource::Schema);
                        }
                        GeneratedField::Table => {
                            if resource__.is_some() {
                                return Err(serde::de::Error::duplicate_field("table"));
                            }
                            resource__ = map_
                                .next_value::<::std::option::Option<_>>()?
                                .map(resource::Resource::Table);
                        }
                        GeneratedField::Column => {
                            if resource__.is_some() {
                                return Err(serde::de::Error::duplicate_field("column"));
                            }
                            resource__ = map_
                                .next_value::<::std::option::Option<_>>()?
                                .map(resource::Resource::Column);
                        }
                        GeneratedField::ExternalLocation => {
                            if resource__.is_some() {
                                return Err(serde::de::Error::duplicate_field("externalLocation"));
                            }
                            resource__ = map_
                                .next_value::<::std::option::Option<_>>()?
                                .map(resource::Resource::ExternalLocation);
                        }
                        GeneratedField::Recipient => {
                            if resource__.is_some() {
                                return Err(serde::de::Error::duplicate_field("recipient"));
                            }
                            resource__ = map_
                                .next_value::<::std::option::Option<_>>()?
                                .map(resource::Resource::Recipient);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(Resource {
                    resource: resource__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.internal.Resource", FIELDS, GeneratedVisitor)
    }
}
