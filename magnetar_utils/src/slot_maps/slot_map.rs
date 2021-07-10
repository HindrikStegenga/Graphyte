use super::slots::*;
use crate::handles::{Handle, HandleType};

pub struct SlotMap<T, K = u32>
where
    K: HandleType,
{
    slots: Vec<Slot<K>>,
    data: Vec<T>,
    free_list_head: Option<K>,
    free_list_tail: Option<K>,
}

impl<T, K> SlotMap<T, K>
where
    K: HandleType,
{
    pub fn new() -> SlotMap<T, K> {
        SlotMap {
            slots: vec![],
            data: vec![],
            free_list_head: None,
            free_list_tail: None,
        }
    }

    pub fn with_capacity(capacity: usize) -> SlotMap<T, K> {
        SlotMap {
            slots: Vec::with_capacity(capacity),
            data: Vec::with_capacity(capacity),
            free_list_head: None,
            free_list_tail: None,
        }
    }

    pub fn add(&mut self, item: T) -> Handle<T, K> {
        //Push the item into the data array
        let index_in_data = self.data.len();
        self.data.push(item);

        //Obtain the new slotmapkey
        return if let Some(index_in_slots) = self.free_list_head {
            let slot = unsafe {
                self.slots
                    .get_unchecked_mut(index_in_slots.to_usize().unwrap())
            };
            if self.free_list_tail == self.free_list_head {
                self.free_list_tail = None;
                self.free_list_head = None;
            } else {
                self.free_list_head = Some(slot.index);
            }

            slot.index = index_in_slots;

            let last_slot = unsafe { self.slots.get_unchecked_mut(index_in_data) };
            last_slot.reverse_slot = index_in_slots;

            Handle::from(index_in_slots)
        } else {
            assert_eq!(self.slots.len(), self.data.len() - 1);
            //This means our slots are compressed, so no gaps. Therefore append to the end as well.
            let index_in_slots = self.slots.len();
            self.slots.push(Slot {
                index: K::from_usize(index_in_data).unwrap(),
                reverse_slot: K::from_usize(index_in_slots).unwrap(),
            });
            Handle::from(K::from_usize(index_in_slots).unwrap())
        };
    }

    pub fn get(&self, key: Handle<T, K>) -> Option<&T> {
        if !self.validate_key(key) {
            return None;
        }
        let slot = unsafe { self.slots.get_unchecked(key.value.to_usize().unwrap()) };
        return self.data.get(slot.index.to_usize().unwrap());
    }

    pub fn get_mut(&mut self, key: Handle<T, K>) -> Option<&mut T> {
        if !self.validate_key(key) {
            return None;
        }
        let slot = unsafe { self.slots.get_unchecked(key.value.to_usize().unwrap()) };
        return self.data.get_mut(slot.index.to_usize().unwrap());
    }

    pub fn remove(&mut self, key: Handle<T, K>) -> Option<T> {
        if !self.validate_key(key) {
            return None;
        }

        let last_idx_in_data = self.data.len() - 1;
        let removed_item_slot_index = key.value.to_usize().unwrap();

        //get references to the slots
        //let removed_item_slot = unsafe { self.slots.get_unchecked_mut(removed_item_slot_index) };

        let removed_item_data_index =
            unsafe { self.slots.get_unchecked_mut(removed_item_slot_index) }.index;
        let item = self
            .data
            .swap_remove(removed_item_data_index.to_usize().unwrap());

        //Update the free list
        if let Some(list_head) = self.free_list_head {
            //Free list present
            unsafe { self.slots.get_unchecked_mut(removed_item_slot_index) }.index = list_head;
            self.free_list_head = Some(key.value);
        } else {
            //No free list present, so we set the idx to 0, and set (head, tail)
            unsafe { self.slots.get_unchecked_mut(removed_item_slot_index) }.index = K::zero();
            self.free_list_head = Some(key.value);
            self.free_list_tail = Some(key.value);
        }

        //Update last slot to point to the moved item in the array
        let last_elem_slot = unsafe { self.slots.get_unchecked_mut(last_idx_in_data) };
        last_elem_slot.index = removed_item_data_index;
        last_elem_slot.reverse_slot = K::zero();

        //Update reverse index in removed slot to point to the last slot index
        unsafe { self.slots.get_unchecked_mut(removed_item_slot_index) }.reverse_slot =
            K::from_usize(last_idx_in_data).unwrap();

        return Some(item);
    }

    pub fn clear(&mut self) {
        //Erase all contents, invalidates all the slots
        self.slots.clear();
        self.data.clear();
        self.free_list_head = None;
        self.free_list_tail = None;
    }

    pub fn len(&self) -> usize {
        return self.data.len();
    }

    pub fn capacity(&self) -> usize {
        return self.data.capacity();
    }

    pub fn data(&self) -> &[T] {
        self.data.as_slice()
    }

    pub fn data_mut(&mut self) -> &mut [T] {
        self.data.as_mut_slice()
    }

    pub fn iter(&self) -> std::slice::Iter<T> {
        return self.data.iter();
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<T> {
        return self.data.iter_mut();
    }

    pub fn iter_keys<'a>(&'a self) -> impl Iterator<Item = Handle<T, K>> + 'a {
        return SlotMapKeyIterator::from(self);
    }

    pub fn iter_key_values(&self) -> impl Iterator<Item = (Handle<T, K>, &T)> {
        return SlotMapKeyValueIterator::from(self);
    }
}

pub(super) struct SlotMapKeyIterator<'map, T, K>
where
    K: HandleType,
{
    map: &'map SlotMap<T, K>,
    current_index: usize,
}

impl<'map, T, K> From<&'map SlotMap<T, K>> for SlotMapKeyIterator<'map, T, K>
where
    K: HandleType,
{
    fn from(map: &'map SlotMap<T, K>) -> Self {
        return Self {
            map,
            current_index: 0,
        };
    }
}

impl<'map, T, K> Iterator for SlotMapKeyIterator<'map, T, K>
where
    K: HandleType,
{
    type Item = Handle<T, K>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_index < self.map.slots.len() {
            let current_index = self.current_index;
            let item = self.map.slots.get(current_index).unwrap();
            self.current_index += 1;
            return Some(Handle::from(item.index));
        }
        return None;
    }
}

pub(super) struct SlotMapKeyValueIterator<'map, T, K>
where
    K: HandleType,
{
    map: &'map SlotMap<T, K>,
    current_index: usize,
}

impl<'map, T, K> From<&'map SlotMap<T, K>> for SlotMapKeyValueIterator<'map, T, K>
where
    K: HandleType,
{
    fn from(map: &'map SlotMap<T, K>) -> Self {
        return Self {
            map,
            current_index: 0,
        };
    }
}

impl<'map, T, K> Iterator for SlotMapKeyValueIterator<'map, T, K>
where
    K: HandleType,
{
    type Item = (Handle<T, K>, &'map T);

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_index < self.map.data.len() {
            let current_index = self.current_index;
            let item = self.map.data.get(current_index).unwrap();
            let reverse_slot = self.map.slots.get(current_index).unwrap();
            self.current_index += 1;
            return Some((Handle::from(reverse_slot.reverse_slot), item));
        }
        return None;
    }
}

impl<T, K> SlotMap<T, K>
where
    K: HandleType,
{
    fn validate_key(&self, key: Handle<T, K>) -> bool {
        let usize_idx = key.value.to_usize().unwrap();
        if usize_idx > self.slots.len() {
            return false;
        }

        return true;
    }
}

impl<T, K> Clone for SlotMap<T, K>
where
    T: Clone,
    K: HandleType,
{
    fn clone(&self) -> Self {
        return Self {
            slots: self.slots.clone(),
            data: self.data.clone(),
            free_list_head: self.free_list_head.clone(),
            free_list_tail: self.free_list_tail.clone(),
        };
    }
}

unsafe impl<T, K> Send for SlotMap<T, K>
where
    T: Send,
    K: HandleType,
{
}

impl<T, K> Default for SlotMap<T, K>
where
    K: HandleType,
{
    fn default() -> Self {
        SlotMap {
            slots: vec![],
            data: vec![],
            free_list_head: None,
            free_list_tail: None,
        }
    }
}

impl<'a, T, K> IntoIterator for &'a SlotMap<T, K>
where
    K: HandleType,
{
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        return self.data.iter();
    }
}

impl<'a, T, K> IntoIterator for &'a mut SlotMap<T, K>
where
    K: HandleType,
{
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        return self.data.iter_mut();
    }
}
