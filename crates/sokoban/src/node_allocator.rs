use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};
use num_derive::FromPrimitive;
use std::collections::BTreeSet;
use std::mem::{align_of, size_of};

/// Enum representing the fields of a tree node:
/// 0 - left pointer
/// 1 - right pointer
/// 2 - parent pointer
/// 3 - value pointer (index of leaf)
#[derive(Debug, Copy, Clone, PartialEq, Eq, FromPrimitive)]
pub enum TreeField {
    Left = 0,
    Right = 1,
    Parent = 2,
    Value = 3,
}

/// Enum representing the fields of a simple node (Linked List / Binary Tree):
/// 0 - left pointer
/// 1 - right pointer
#[derive(Debug, Copy, Clone, PartialEq, Eq, FromPrimitive)]
pub enum NodeField {
    Left = 0,
    Right = 1,
}

/// This is a convenience trait that exposes an interface to read a struct from an arbitrary byte array
pub trait FromSlice {
    fn new_from_slice(data: &mut [u8]) -> &mut Self;
}

/// This trait provides an API for map-like data structures that use the NodeAllocator
/// struct as the underlying container
pub trait NodeAllocatorMap<K, V> {
    fn insert(&mut self, key: K, value: V) -> Option<u32>;
    fn remove(&mut self, key: &K) -> Option<V>;
    fn contains(&self, key: &K) -> bool;
    fn get(&self, key: &K) -> Option<&V>;
    fn get_mut(&mut self, key: &K) -> Option<&mut V>;
    #[deprecated]
    fn size(&self) -> usize;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn capacity(&self) -> usize;
    fn iter(&self) -> Box<dyn DoubleEndedIterator<Item = (&K, &V)> + '_>;
    fn iter_mut(&mut self) -> Box<dyn DoubleEndedIterator<Item = (&K, &mut V)> + '_>;
}

/// This trait adds additional functions for sorted map data structures that use the NodeAllocator
pub trait OrderedNodeAllocatorMap<K, V>: NodeAllocatorMap<K, V> {
    fn get_min_index(&mut self) -> u32;
    fn get_max_index(&mut self) -> u32;
    fn get_min(&mut self) -> Option<(K, V)>;
    fn get_max(&mut self) -> Option<(K, V)>;
}

pub trait ZeroCopy: Pod {
    fn load_mut_bytes(data: &'_ mut [u8]) -> Option<&'_ mut Self> {
        let size = std::mem::size_of::<Self>();
        bytemuck::try_from_bytes_mut(&mut data[..size]).ok()
    }

    fn load_bytes(data: &'_ [u8]) -> Option<&'_ Self> {
        let size = std::mem::size_of::<Self>();
        bytemuck::try_from_bytes(&data[..size]).ok()
    }
}

pub const SENTINEL: u32 = 0;

#[repr(C)]
#[cfg_attr(
    feature = "native",
    derive(serde::Serialize, serde::Deserialize),
    derive(sov_modules_api::macros::UniversalWallet),
    serde(bound = "T: serde::Serialize + serde::de::DeserializeOwned")
)]
#[derive(Copy, Clone, BorshSerialize, BorshDeserialize)]
pub struct Node<T: Copy + Clone + Pod + Zeroable + Default, const NUM_REGISTERS: usize> {
    /// Arbitrary registers (generally used for pointers)
    /// Note: Register 0 is ALWAYS used for the free list
    #[cfg_attr(feature = "native", serde(with = "serde_arrays", bound = ""))]
    registers: [u32; NUM_REGISTERS],
    value: T,
}

#[cfg(feature = "native")]
impl<T, const NUM_REGISTERS: usize> schemars::JsonSchema for Node<T, NUM_REGISTERS>
where
    T: Copy + Clone + Pod + Zeroable + Default + schemars::JsonSchema,
{
    fn schema_name() -> String {
        format!("Node_{}_{}", std::any::type_name::<T>(), NUM_REGISTERS)
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        use schemars::schema::*;

        let mut required = BTreeSet::new();
        required.insert("registers".to_string());
        required.insert("value".to_string());

        let mut schema_obj = SchemaObject {
            instance_type: Some(InstanceType::Object.into()),
            object: Some(Box::new(ObjectValidation {
                properties: [
                    ("registers".to_string(), gen.subschema_for::<Vec<u32>>()),
                    ("value".to_string(), gen.subschema_for::<T>()),
                ]
                .iter()
                .cloned()
                .collect(),
                required,
                ..Default::default()
            })),
            ..Default::default()
        };

        Schema::Object(schema_obj)
    }
}

impl<T: Copy + Clone + Pod + Zeroable + Default, const NUM_REGISTERS: usize> Default
    for Node<T, NUM_REGISTERS>
{
    fn default() -> Self {
        assert!(NUM_REGISTERS >= 1);
        Self {
            registers: [SENTINEL; NUM_REGISTERS],
            value: T::default(),
        }
    }
}

impl<T: Copy + Clone + Pod + Zeroable + Default, const NUM_REGISTERS: usize>
    Node<T, NUM_REGISTERS>
{
    #[inline(always)]
    pub(crate) fn get_free_list_register(&self) -> u32 {
        self.registers[0]
    }

    #[inline(always)]
    pub fn get_register(&self, r: usize) -> u32 {
        self.registers[r]
    }

    #[inline(always)]
    pub(crate) fn set_free_list_register(&mut self, v: u32) {
        self.registers[0] = v;
    }

    #[inline(always)]
    pub fn set_register(&mut self, r: usize, v: u32) {
        self.registers[r] = v;
    }

    #[inline(always)]
    pub fn set_value(&mut self, v: T) {
        self.value = v;
    }

    #[inline(always)]
    pub fn get_value_mut(&mut self) -> &mut T {
        &mut self.value
    }

    #[inline(always)]
    pub fn get_value(&self) -> &T {
        &self.value
    }
}

#[repr(C)]
#[cfg_attr(
    feature = "native",
    derive(serde::Serialize, serde::Deserialize),
    derive(sov_modules_api::macros::UniversalWallet),
    serde(bound = "T: serde::Serialize + serde::de::DeserializeOwned")
)]
#[derive(Copy, Clone, BorshSerialize, BorshDeserialize)]
pub struct NodeAllocator<
    T: Default + Copy + Clone + Pod + Zeroable,
    const MAX_SIZE: usize,
    const NUM_REGISTERS: usize,
> {
    /// Size of the allocator. The max value this can take is `MAX_SIZE`
    pub size: u64,
    /// Index that represents the "boundary" of the allocator. When this value reaches `MAX_SIZE`
    /// this indicates that all of the nodes has been used at least once and all new allocated
    /// indicies must be pulled from the free list.
    bump_index: u32,
    /// Buffer index of the first element in the free list. The free list is a singly-linked list
    /// of unallocated nodes. The free list operates like a stack. When a node is removed from the
    /// allocator, the removed node becomes the new free list head. When new nodes are added,
    /// the new index to allocated is pulled from the `free_list_head`
    free_list_head: u32,
    /// Nodes containing data, with `NUM_REGISTERS` registers that store arbitrary data
    #[cfg_attr(feature = "native", serde(with = "serde_arrays", bound = ""))]
    pub nodes: [Node<T, NUM_REGISTERS>; MAX_SIZE],
}

#[cfg(feature = "native")]
impl<T, const MAX_SIZE: usize, const NUM_REGISTERS: usize> schemars::JsonSchema
    for NodeAllocator<T, MAX_SIZE, NUM_REGISTERS>
where
    T: Default + Copy + Clone + Pod + Zeroable + schemars::JsonSchema,
{
    fn schema_name() -> String {
        format!(
            "NodeAllocator_{}_{}_{}",
            std::any::type_name::<T>(),
            MAX_SIZE,
            NUM_REGISTERS
        )
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        use schemars::schema::*;
        let mut required = BTreeSet::new();
        required.insert("size".to_string());
        required.insert("bump_index".to_string());
        required.insert("free_list_head".to_string());
        required.insert("nodes".to_string());

        let mut schema_obj = SchemaObject {
            instance_type: Some(InstanceType::Object.into()),
            object: Some(Box::new(ObjectValidation {
                properties: [
                    ("size".to_string(), gen.subschema_for::<u64>()),
                    ("bump_index".to_string(), gen.subschema_for::<u32>()),
                    ("free_list_head".to_string(), gen.subschema_for::<u32>()),
                    (
                        "nodes".to_string(),
                        gen.subschema_for::<Vec<Node<T, NUM_REGISTERS>>>(),
                    ),
                ]
                .iter()
                .cloned()
                .collect(),
                required,
                ..Default::default()
            })),
            ..Default::default()
        };

        Schema::Object(schema_obj)
    }
}

unsafe impl<
        T: Default + Copy + Clone + Pod + Zeroable,
        const MAX_SIZE: usize,
        const NUM_REGISTERS: usize,
    > Zeroable for NodeAllocator<T, MAX_SIZE, NUM_REGISTERS>
{
}

unsafe impl<
        T: Default + Copy + Clone + Pod + Zeroable,
        const MAX_SIZE: usize,
        const NUM_REGISTERS: usize,
    > Pod for NodeAllocator<T, MAX_SIZE, NUM_REGISTERS>
{
}

impl<
        T: Default + Copy + Clone + Pod + Zeroable,
        const MAX_SIZE: usize,
        const NUM_REGISTERS: usize,
    > ZeroCopy for NodeAllocator<T, MAX_SIZE, NUM_REGISTERS>
{
}

impl<
        T: Default + Copy + Clone + Pod + Zeroable,
        const MAX_SIZE: usize,
        const NUM_REGISTERS: usize,
    > Default for NodeAllocator<T, MAX_SIZE, NUM_REGISTERS>
{
    fn default() -> Self {
        assert!(NUM_REGISTERS >= 1);
        let na = NodeAllocator {
            size: 0,
            bump_index: 1,
            free_list_head: 1,
            nodes: [Node::<T, NUM_REGISTERS>::default(); MAX_SIZE],
        };
        na.assert_proper_alignment();
        na
    }
}

impl<
        T: Default + Copy + Clone + Pod + Zeroable,
        const MAX_SIZE: usize,
        const NUM_REGISTERS: usize,
    > NodeAllocator<T, MAX_SIZE, NUM_REGISTERS>
{
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    fn assert_proper_alignment(&self) {
        let reg_size = size_of::<u32>() * NUM_REGISTERS;
        let self_ptr = std::slice::from_ref(self).as_ptr() as usize;
        let node_ptr = std::slice::from_ref(&self.nodes).as_ptr() as usize;
        let self_align = align_of::<Self>();
        let t_index = node_ptr + reg_size;
        let t_align = align_of::<T>();
        let t_size = size_of::<T>();
        assert!(
            self_ptr % self_align as usize == 0,
            "NodeAllocator alignment mismatch, address is {} which is not a multiple of the struct alignment ({})",
            self_ptr,
            self_align,
        );
        assert!(
            t_size % t_align == 0,
            "Size of T ({}) is not a multiple of the alignment of T ({})",
            t_size,
            t_align,
        );
        assert!(
            t_size == 0 || t_size >= self_align,
            "Size of T ({}) must be >= than the alignment of NodeAllocator ({})",
            t_size,
            self_align,
        );
        assert!(node_ptr == self_ptr + 16, "Nodes are misaligned");
        assert!(t_index % t_align == 0, "First index of T is misaligned");
        assert!(
            (t_index + t_size + reg_size) % t_align == 0,
            "Subsequent indices of T are misaligned"
        );
    }

    pub fn initialize(&mut self) {
        assert!(NUM_REGISTERS >= 1);
        self.assert_proper_alignment();
        if self.size == 0 && self.bump_index == 0 && self.free_list_head == 0 {
            self.bump_index = 1;
            self.free_list_head = 1;
        } else {
            panic!("Cannot reinitialize NodeAllocator");
        }
    }

    #[inline(always)]
    pub fn get(&self, i: u32) -> &Node<T, NUM_REGISTERS> {
        &self.nodes[(i - 1) as usize]
    }

    #[inline(always)]
    pub fn get_mut(&mut self, i: u32) -> &mut Node<T, NUM_REGISTERS> {
        &mut self.nodes[(i - 1) as usize]
    }

    /// Adds a new node to the allocator. The function returns the current pointer
    /// to the free list, where the new node is inserted
    pub fn add_node(&mut self, node: T) -> u32 {
        let i = self.free_list_head;
        if self.free_list_head == self.bump_index {
            if self.bump_index == (MAX_SIZE + 1) as u32 {
                panic!("Buffer is full, size {}", self.size);
            }
            self.bump_index += 1;
            self.free_list_head = self.bump_index;
        } else {
            self.free_list_head = self.get(i).get_free_list_register();
            self.get_mut(i).set_free_list_register(SENTINEL);
        }
        self.get_mut(i).set_value(node);
        self.size += 1;
        i
    }

    /// Removes the node at index `i` from the allocator and adds the index to the free list
    /// When deleting nodes, you MUST clear all registers prior to calling `remove_node`
    pub fn remove_node(&mut self, i: u32) -> Option<&T> {
        if i == SENTINEL {
            return None;
        }
        let free_list_head = self.free_list_head;
        self.get_mut(i).set_free_list_register(free_list_head);
        self.free_list_head = i;
        self.size -= 1;
        Some(self.get(i).get_value())
    }

    #[inline(always)]
    pub fn disconnect(&mut self, i: u32, j: u32, r_i: u32, r_j: u32) {
        if i != SENTINEL {
            // assert!(j == self.get_register(i, r_i), "Nodes are not connected");
            self.clear_register(i, r_i);
        }
        if j != SENTINEL {
            // assert!(i == self.get_register(j, r_j), "Nodes are not connected");
            self.clear_register(j, r_j);
        }
    }

    #[inline(always)]
    pub fn clear_register(&mut self, i: u32, r_i: u32) {
        if i != SENTINEL {
            self.get_mut(i).set_register(r_i as usize, SENTINEL);
        }
    }

    #[inline(always)]
    pub fn connect(&mut self, i: u32, j: u32, r_i: u32, r_j: u32) {
        if i != SENTINEL {
            self.get_mut(i).set_register(r_i as usize, j);
        }
        if j != SENTINEL {
            self.get_mut(j).set_register(r_j as usize, i);
        }
    }

    #[inline(always)]
    pub fn set_register(&mut self, i: u32, value: u32, r_i: u32) {
        if i != SENTINEL {
            self.get_mut(i).set_register(r_i as usize, value);
        }
    }

    #[inline(always)]
    pub fn get_register(&self, i: u32, r_i: u32) -> u32 {
        if i != SENTINEL {
            self.get(i).get_register(r_i as usize)
        } else {
            SENTINEL
        }
    }
}
