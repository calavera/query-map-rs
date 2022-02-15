#![crate_name = "query_map"]
#![deny(clippy::all, clippy::cargo)]
#![warn(missing_docs, nonstandard_style, rust_2018_idioms)]

//!
//!
//! QueryMap is a generic wrapper around HashMap<String, Vec<V>>
//! to handle different transformations like URL query strings.
//!
//! QueryMap can normalize HashMap structures with single value elements
//! into structures with value vector elements.
//!
//! # Examples
//!
//! Create a QueryMap from a HashMap:
//!
//! ```
//! use std::collections::HashMap;
//! use query_map::QueryMap;
//!
//! let mut data = HashMap::new();
//! data.insert("foo".into(), vec!["bar".into()]);
//!
//! let map: QueryMap<String> = QueryMap::from(data);
//! assert_eq!("bar", map.first("foo").unwrap().as_str());
//! assert_eq!(None, map.first("bar"));
//! ```
//!
//! Create a QueryMap from a Serde Value (requires `serde` feature):
//!
//! ```ignore
//! use query_map::QueryMap;
//!
//! #[derive(Deserialize)]
//! struct Test {
//!     data: QueryMap<String>,
//! }
//!
//! let json = serde_json::json!({
//!     "data": {
//!         "foo": "bar"
//!     }
//! });
//!
//! let test: Test = serde_json::from_value(json).unwrap();
//! assert_eq!("bar", test.data.first("foo").unwrap().as_str());
//! ```
//!
//! Create a QueryMap from a query string (requires `url-query` feature):
//!
//! ```
//! use query_map::QueryMap;
//!
//! let data = "foo=bar&baz=quux&foo=qux";
//! let map = data.parse::<QueryMap<String>>().unwrap();
//! let got = map.all("foo").unwrap();
//! assert_eq!(vec!["bar", "qux"], got);
//! ```
//!

use std::{
    collections::{hash_map::Keys, HashMap},
    sync::Arc,
};

#[cfg(feature = "serde")]
mod serde;

#[cfg(feature = "serde")]
pub use crate::serde::*;

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde_derive;

#[cfg(feature = "url-query")]
mod url_query;

#[cfg(feature = "url-query")]
pub use url_query::*;

/// A read-only view into a map of data which may contain multiple values
///
/// Internally data is always represented as many values
#[derive(Default, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(crate = "serde_crate"))]
pub struct QueryMap<V>(pub(crate) Arc<HashMap<String, Vec<V>>>);

impl<V> QueryMap<V> {
    /// Return the first elelemnt associated with a key
    pub fn first(&self, key: &str) -> Option<&V> {
        self.0.get(key).and_then(|values| values.first())
    }

    /// Return all elements associated with a key
    pub fn all(&self, key: &str) -> Option<Vec<&V>> {
        self.0
            .get(key)
            .map(|values| values.iter().collect::<Vec<_>>())
    }

    /// Return true if there are no elements in the map
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Return an iterator for this map
    pub fn iter(&self) -> QueryMapIter<'_, V> {
        QueryMapIter {
            data: self,
            keys: self.0.keys(),
            current: None,
            next_idx: 0,
        }
    }
}

impl<V> Clone for QueryMap<V> {
    fn clone(&self) -> Self {
        QueryMap(self.0.clone())
    }
}

impl<V> From<HashMap<String, Vec<V>>> for QueryMap<V> {
    fn from(inner: HashMap<String, Vec<V>>) -> Self {
        QueryMap(Arc::new(inner))
    }
}

/// A read only reference to the `QueryMap`'s data
pub struct QueryMapIter<'a, V> {
    data: &'a QueryMap<V>,
    keys: Keys<'a, String, Vec<V>>,
    current: Option<(&'a String, Vec<&'a V>)>,
    next_idx: usize,
}

impl<'a, V> Iterator for QueryMapIter<'a, V> {
    type Item = (&'a str, &'a V);

    #[inline]
    fn next(&mut self) -> Option<(&'a str, &'a V)> {
        if self.current.is_none() {
            self.current = self
                .keys
                .next()
                .map(|k| (k, self.data.all(k).unwrap_or_default()));
        };

        let mut reset = false;
        let ret = if let Some((key, values)) = &self.current {
            let value = values[self.next_idx];

            if self.next_idx + 1 < values.len() {
                self.next_idx += 1;
            } else {
                reset = true;
            }

            Some((key.as_str(), value))
        } else {
            None
        };

        if reset {
            self.current = None;
            self.next_idx = 0;
        }

        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn str_map_default_is_empty() {
        let d: QueryMap<String> = QueryMap::default();
        assert!(d.is_empty())
    }

    #[test]
    fn test_map_first() {
        let mut data = HashMap::new();
        data.insert("foo".into(), vec!["bar".into()]);
        let map: QueryMap<String> = QueryMap(data.into());
        assert_eq!("bar", map.first("foo").unwrap().as_str());
        assert_eq!(None, map.first("bar"));
    }

    #[test]
    fn test_map_all() {
        let mut data = HashMap::new();
        data.insert("foo".into(), vec!["bar".into(), "baz".into()]);
        let map: QueryMap<String> = QueryMap(data.into());
        let got = map.all("foo").unwrap();
        assert_eq!(vec!["bar", "baz"], got);
        assert_eq!(None, map.all("bar"));
    }

    #[test]
    fn test_map_iter() {
        let mut data = HashMap::new();
        data.insert("foo".into(), vec!["bar".into()]);
        data.insert("baz".into(), vec!["boom".into()]);
        let map: QueryMap<String> = QueryMap(data.into());
        let mut values = map.iter().map(|(_, v)| v).collect::<Vec<_>>();
        values.sort();
        assert_eq!(vec!["bar", "boom"], values);
    }
}
