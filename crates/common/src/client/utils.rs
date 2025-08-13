use std::collections::HashMap;

use crate::google::protobuf::{Struct, Value, value::Kind as ValueKind};

pub(super) fn hash_map_to_struct(map: HashMap<String, String>) -> Struct {
    Struct {
        fields: map
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    Value {
                        kind: Some(ValueKind::StringValue(v.clone())),
                    },
                )
            })
            .collect(),
    }
}
