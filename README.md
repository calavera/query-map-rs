QueryMap is a generic wrapper around HashMap<String, Vec<V>>
to handle different transformations like URL query strings.

QueryMap can normalize HashMap structures with single value elements
into structures with value vector elements.

## Installation

```
cargo install query_map
```

## Examples

Create a QueryMap from a HashMap:

```
use std::collections::HashMap;
use query_map::QueryMap;

let mut data = HashMap::new();
data.insert("foo".into(), vec!["bar".into()]);

let map: QueryMap<String> = QueryMap::from(data);
assert_eq!("bar", map.first("foo").unwrap().as_str());
assert_eq!(None, map.first("bar"));
```

Create a QueryMap from a Serde Value (requires `serde` feature):

```ignore
use query_map::QueryMap;
#[derive(Deserialize)]
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
```

Create a QueryMap from a query string (requires `url-query` feature):

```
use query_map::QueryMap;

let data = "foo=bar&baz=quux&foo=qux";
let map = data.parse::<QueryMap<String>>().unwrap();
let got = map.all("foo").unwrap();
assert_eq!(vec!["bar", "qux"], got);
```