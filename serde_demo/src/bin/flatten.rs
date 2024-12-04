use serde::Deserializer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug)]
struct Foo<T> {
    f1: T,
}

impl<T: Serialize> Serialize for Foo<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        self.f1.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for Foo<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Delegate deserialization to T
        let f1 = T::deserialize(deserializer)?;
        Ok(Foo { f1 })
    }
}

fn main() {
    // Example: HashMap<String, Vec<usize>> as the generic type
    let mut map: HashMap<String, Vec<usize>> = HashMap::new();
    map.insert("key1".to_string(), vec![1, 2, 3]);
    map.insert("key2".to_string(), vec![4, 5, 6]);

    // Serialize Foo2<HashMap<String, Vec<usize>>>
    let foo = Foo { f1: map };
    let json = serde_json::to_string(&foo).unwrap();
    println!("{}", json);
    // Output: {"key1":[1,2,3],"key2":[4,5,6]}

    // Deserialize JSON back into Foo2<HashMap<String, Vec<usize>>>
    let json = r#"{"key1":[1,2,3],"key2":[4,5,6]}"#;
    let foo: Foo<HashMap<String, Vec<usize>>> = serde_json::from_str(json).unwrap();
    println!("{:?}", foo);
    // Output: Foo2 { f1: {"key1": [1, 2, 3], "key2": [4, 5, 6]} }
}
