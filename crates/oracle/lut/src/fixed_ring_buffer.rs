#[cfg(feature = "native")]
use schemars::{
    schema::{ArrayValidation, InstanceType, Schema, SchemaObject},
    JsonSchema,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::marker::PhantomData;

#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone, Copy)]
pub struct FixedRingBuffer<T, const N: usize> {
    pub ring_buffer: [T; N],
    pub current_index: usize,
}

#[cfg(feature = "native")]
impl<T, const N: usize> Serialize for FixedRingBuffer<T, N>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeTuple;
        let mut tup = serializer.serialize_tuple(N + 1)?;
        for item in &self.ring_buffer {
            tup.serialize_element(item)?;
        }
        tup.serialize_element(&self.current_index)?;
        tup.end()
    }
}

#[cfg(feature = "native")]
impl<'de, T, const N: usize> Deserialize<'de> for FixedRingBuffer<T, N>
where
    T: Deserialize<'de> + Copy + Default,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FixedRingBufferVisitor<T, const N: usize>(PhantomData<T>);

        impl<'de, T, const N: usize> serde::de::Visitor<'de> for FixedRingBufferVisitor<T, N>
        where
            T: Deserialize<'de> + Copy + Default,
        {
            type Value = FixedRingBuffer<T, N>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a FixedRingBuffer")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut ring_buffer = [T::default(); N];
                for item in ring_buffer.iter_mut() {
                    *item = seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(N, &self))?;
                }
                let current_index = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(N + 1, &self))?;

                Ok(FixedRingBuffer {
                    ring_buffer,
                    current_index,
                })
            }
        }

        deserializer.deserialize_tuple(N + 1, FixedRingBufferVisitor::<T, N>(PhantomData))
    }
}

#[cfg(feature = "native")]
impl<T, const N: usize> JsonSchema for FixedRingBuffer<T, N>
where
    T: JsonSchema,
{
    fn schema_name() -> String {
        format!("FixedRingBuffer<{}, {}>", T::schema_name(), N)
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> Schema {
        let mut obj = SchemaObject {
            instance_type: Some(InstanceType::Object.into()),
            ..Default::default()
        };

        let ring_buffer_schema = Schema::Object(SchemaObject {
            instance_type: Some(InstanceType::Array.into()),
            array: Some(Box::new(ArrayValidation {
                items: Some(gen.subschema_for::<T>().into()),
                max_items: Some(N as u32),
                min_items: Some(N as u32),
                ..Default::default()
            })),
            ..Default::default()
        });

        let current_index_schema = gen.subschema_for::<usize>();

        obj.metadata().title = Some(Self::schema_name());
        obj.object()
            .properties
            .insert("ring_buffer".to_string(), ring_buffer_schema);
        obj.object()
            .properties
            .insert("current_index".to_string(), current_index_schema);

        Schema::Object(obj)
    }
}

impl<T, const N: usize> FixedRingBuffer<T, N> {
    pub const fn len(&self) -> usize {
        N
    }

    pub fn push_or_overwrite(&mut self, value: T) {
        let index = self.current_index;
        self.ring_buffer[index] = value;
        self.current_index = (index + 1) % N;
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        let index = (self.current_index + index) % N;
        self.ring_buffer.get(index)
    }
}

impl<T: Copy, const N: usize> FixedRingBuffer<T, N> {
    pub fn to_vec(&self) -> Vec<T> {
        self.ring_buffer.to_vec()
    }
}

impl<T: Copy + Default, const N: usize> Default for FixedRingBuffer<T, N> {
    fn default() -> Self {
        FixedRingBuffer {
            ring_buffer: [T::default(); N],
            current_index: 0,
        }
    }
}

impl<T: IntoIterator, const N: usize> FixedRingBuffer<T, N> {
    pub fn to_nested_vec(self) -> Vec<Vec<<T as IntoIterator>::Item>> {
        self.ring_buffer
            .into_iter()
            .map(|e| e.into_iter().collect())
            .collect()
    }
}
