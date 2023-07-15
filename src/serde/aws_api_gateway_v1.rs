use serde_crate::{ser::SerializeMap, Serializer};

use crate::QueryMap;
use std::collections::HashMap;

/// Serializes `QueryMap`, converting value from `Vec<String>` to `String`
pub fn serialize_query_string_parameters<S>(
    value: &QueryMap,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let query_string_parameters: HashMap<String, String> = (*value)
        .iter()
        .map(|(k, _)| (String::from(k), String::from((*value).first(k).unwrap())))
        .collect::<HashMap<String, String>>();

    let mut map = serializer.serialize_map(Some(query_string_parameters.len()))?;
    for (k, v) in &query_string_parameters {
        map.serialize_entry(k, v)?;
    }
    map.end()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serde::aws_api_gateway_v2::deserialize_empty;

    #[test]
    fn test_serialize_query_string_parameters() {
        #[cfg_attr(
            feature = "serde",
            derive(Deserialize, Serialize),
            serde(crate = "serde_crate")
        )]
        struct Test {
            #[serde(default, deserialize_with = "deserialize_empty")]
            #[serde(serialize_with = "serialize_query_string_parameters")]
            pub v: QueryMap,
        }

        let data = serde_json::json!({
            "v": {
                "key1": ["value1", "value2", "value3"]
            }
        });

        let decoded: Test = serde_json::from_value(data).unwrap();
        let key1_value = decoded.v.all("key1").unwrap();
        assert_eq!(3, key1_value.len());
        assert_eq!("value1", *(key1_value.first().unwrap()));

        let encoded = serde_json::to_string(&decoded).unwrap();
        assert_eq!(encoded, r#"{"v":{"key1":"value1"}}"#.to_string());
    }
}
