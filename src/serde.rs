use serde_crate::{
    de::{MapAccess, Visitor},
    Deserialize, Deserializer,
};

use super::QueryMap;
use std::{collections::HashMap, fmt, marker::PhantomData, sync::Arc};

#[cfg_attr(feature = "serde", derive(Deserialize), serde(crate = "serde_crate"))]
#[serde(untagged)]
enum OneOrMany<V> {
    One(V),
    Many(Vec<V>),
}

impl<'de, V> Deserialize<'de> for QueryMap<V>
where
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<QueryMap<V>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct QueryMapVisitor<V> {
            d: PhantomData<V>,
        }

        impl<'de, V> Visitor<'de> for QueryMapVisitor<V>
        where
            V: Deserialize<'de>,
        {
            type Value = QueryMap<V>;

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
                // values may either be a single V or Vec<V>
                // to handle both single and multi value data
                while let Some((key, value)) = map.next_entry::<_, OneOrMany<V>>()? {
                    inner.insert(
                        key,
                        match value {
                            OneOrMany::One(one) => vec![one],
                            OneOrMany::Many(many) => many,
                        },
                    );
                }
                Ok(QueryMap(Arc::new(inner)))
            }
        }

        deserializer.deserialize_map(QueryMapVisitor { d: PhantomData })
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
            data: QueryMap<String>,
        }

        let json = serde_json::json!({
            "data": {
                "foo": "bar"
            }
        });

        let test: Test = serde_json::from_value(json).unwrap();
        assert_eq!("bar", test.data.first("foo").unwrap().as_str());

        let expected = serde_json::json!({
            "data": {
                "foo": ["bar"]
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
            data: QueryMap<String>,
        }

        let json = serde_json::json!({
            "data": {
                "foo": ["bar"]
            }
        });

        let test: Test = serde_json::from_value(json.clone()).unwrap();
        assert_eq!("bar", test.data.first("foo").unwrap().as_str());

        let reparsed = serde_json::to_value(test).unwrap();
        assert_eq!(json, reparsed);
    }
}
