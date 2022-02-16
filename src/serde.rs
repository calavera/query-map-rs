use serde_crate::{
    de::{MapAccess, Visitor},
    Deserialize, Deserializer,
};

use super::QueryMap;
use std::{collections::HashMap, fmt, sync::Arc};

#[cfg_attr(feature = "serde", derive(Deserialize), serde(crate = "serde_crate"))]
#[serde(untagged)]
enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

impl<'de> Deserialize<'de> for QueryMap {
    fn deserialize<D>(deserializer: D) -> Result<QueryMap, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct QueryMapVisitor;

        impl<'de> Visitor<'de> for QueryMapVisitor {
            type Value = QueryMap;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "a QueryMap")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut inner = map
                    .size_hint()
                    .map(HashMap::with_capacity)
                    .unwrap_or_else(HashMap::new);
                // values may either be a single String or Vec<String>
                // to handle both single and multi value data
                while let Some((key, value)) = map.next_entry::<_, OneOrMany>()? {
                    inner.insert(
                        key,
                        match value {
                            OneOrMany::One(one) => {
                                one.split(',').map(String::from).collect::<Vec<_>>()
                            }
                            OneOrMany::Many(many) => many,
                        },
                    );
                }
                Ok(QueryMap(Arc::new(inner)))
            }
        }

        deserializer.deserialize_map(QueryMapVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_single() {
        #[cfg_attr(
            feature = "serde",
            derive(Deserialize, Serialize),
            serde(crate = "serde_crate")
        )]
        struct Test {
            data: QueryMap,
        }

        let json = serde_json::json!({
            "data": {
                "foo": "bar"
            }
        });

        let test: Test = serde_json::from_value(json).unwrap();
        assert_eq!("bar", test.data.first("foo").unwrap());

        let expected = serde_json::json!({
            "data": {
                "foo": ["bar"]
            }
        });

        let reparsed = serde_json::to_value(test).unwrap();
        assert_eq!(expected, reparsed);
    }

    #[test]
    fn test_deserialize_single_with_comma_separated_values() {
        #[cfg_attr(
            feature = "serde",
            derive(Deserialize, Serialize),
            serde(crate = "serde_crate")
        )]
        struct Test {
            data: QueryMap,
        }

        let json = serde_json::json!({
            "data": {
                "foo": "bar,baz"
            }
        });

        let test: Test = serde_json::from_value(json).unwrap();
        assert_eq!("bar", test.data.first("foo").unwrap());

        let expected = serde_json::json!({
            "data": {
                "foo": ["bar", "baz"]
            }
        });

        let reparsed = serde_json::to_value(test).unwrap();
        assert_eq!(expected, reparsed);
    }

    #[test]
    fn test_deserialize_vector() {
        #[cfg_attr(
            feature = "serde",
            derive(Deserialize, Serialize),
            serde(crate = "serde_crate")
        )]
        struct Test {
            data: QueryMap,
        }

        let json = serde_json::json!({
            "data": {
                "foo": ["bar"]
            }
        });

        let test: Test = serde_json::from_value(json.clone()).unwrap();
        assert_eq!("bar", test.data.first("foo").unwrap());

        let reparsed = serde_json::to_value(test).unwrap();
        assert_eq!(json, reparsed);
    }
}
