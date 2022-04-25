use core::fmt;

use heapless::LinearMap;
use serde::de::{Error, MapAccess, Visitor};

pub struct LinearMapVisitor<K, V, const N: usize> {
    k: core::marker::PhantomData<K>,
    v: core::marker::PhantomData<V>,
}

impl<K, V, const N: usize> LinearMapVisitor<K, V, N> {
    pub fn new() -> Self {
        Self { k: core::marker::PhantomData, v: core::marker::PhantomData }
    }
}

impl<'de, K, V, const N: usize> Visitor<'de> for LinearMapVisitor<K, V, N>
where
    K: serde::Deserialize<'de> + Eq,
    V: serde::Deserialize<'de>,
{
    type Value = LinearMap<K, V, N>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("LinearMap")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
        let mut map = LinearMap::new();
        while let Some((key, value)) = access.next_entry::<K, V>()? {
            map.insert(key, value).map_err(|_| A::Error::custom("Out of capacity"))?;
        }
        Ok(map)
    }
}
