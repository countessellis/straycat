use std::{
  iter::{Enumerate, FilterMap},
  slice,
  vec,
};

use super::Sequence;

#[derive(Debug,Clone)]
pub struct Table<T> {
  sequence: Sequence,
  active:   u64,
  items:    Vec<Option<T>>,
}

pub type Iter<'a, T> = FilterMap<Enumerate<slice::Iter<'a, Option<T>>>,fn((usize, &'a Option<T>)) -> Option<(u64, &'a T)>>;
pub type IterMut<'a, T> = FilterMap<Enumerate<slice::IterMut<'a, Option<T>>>,fn((usize, &'a mut Option<T>)) -> Option<(u64, &'a mut T)>>;
pub type IntoIter<T> = FilterMap<Enumerate<vec::IntoIter<Option<T>>>,fn((usize, Option<T>)) -> Option<(u64, T)>>;

impl<T> IntoIterator for Table<T> {
  type Item = (u64, T);
  type IntoIter = IntoIter<T>;
  fn into_iter(self) -> Self::IntoIter {
    self.items.into_iter().enumerate().filter_map(|(id, item)| item.map(|v| (id as u64, v)))
  }
}

impl<'a, T> IntoIterator for &'a Table<T> {
  type Item = (u64, &'a T);
  type IntoIter = Iter<'a, T>;
  fn into_iter(self) -> Self::IntoIter {
    self.items.iter().enumerate().filter_map(|(id, item)| item.as_ref().map(|v| (id as u64, v)))
  }
}

impl<'a, T> IntoIterator for &'a mut Table<T> {
  type Item = (u64, &'a mut T);
  type IntoIter = IterMut<'a, T>;
  fn into_iter(self) -> Self::IntoIter {
    self.items.iter_mut().enumerate().filter_map(|(id, item)| item.as_mut().map(|v| (id as u64, v)))
  }
}

impl<T> FromIterator<(u64,T)> for Table<T> {
  fn from_iter<I: IntoIterator<Item = (u64,T)>>(iter: I) -> Self {
    let mut table = Table::new();
    for (id, item) in iter {
      table.insert(id,item);
    }
    table
  }
}

impl<T, const N: usize> From<[(u64,T); N]> for Table<T> {
  fn from(arr: [(u64,T); N]) -> Self {
    Self::from_iter(arr)
  }
}

pub enum Entry<'a, T> {
  Occupied(OccupiedEntry<'a, T>),
  Vacant(VacantEntry<'a, T>),
}

pub struct OccupiedEntry<'a, T> {
  id: u64,
  slot: &'a mut Option<T>,
}

pub struct VacantEntry<'a, T> {
  id: u64,
  table: &'a mut Table<T>,
}

impl<'a, T> Entry<'a, T> {
  pub fn key(&self) -> u64 {
    match self {
      Entry::Occupied(e) => e.key(),
      Entry::Vacant(e) => e.key(),
    }
  }

  pub fn or_insert(self, default: T) -> &'a mut T {
    match self {
      Entry::Occupied(e) => e.into_mut(),
      Entry::Vacant(e) => e.insert(default),
    }
  }

  pub fn or_insert_with<F: FnOnce() -> T>(self, default: F) -> &'a mut T {
    match self {
      Entry::Occupied(e) => e.into_mut(),
      Entry::Vacant(e) => e.insert(default()),
    }
  }

  pub fn or_insert_with_key<F: FnOnce(u64) -> T>(self, default: F) -> &'a mut T {
    match self {
      Entry::Occupied(e) => e.into_mut(),
      Entry::Vacant(e) => {
        let val = default(e.key());
        e.insert(val)
      }
    }
  }

  pub fn and_modify<F: FnOnce(&mut T)>(mut self, f: F) -> Self {
    if let Entry::Occupied(ref mut e) = self {
      f(e.get_mut());
    }
    self
  }
}

impl<'a, T: Default> Entry<'a, T> {
  pub fn or_default(self) -> &'a mut T {
    self.or_insert_with(T::default)
  }
}

impl<'a, T> OccupiedEntry<'a, T> {
  pub fn key(&self) -> u64 {
    self.id
  }

  pub fn get(&self) -> &T {
    self.slot.as_ref().expect("OccupiedEntry must contain Some")
  }

  pub fn get_mut(&mut self) -> &mut T {
    self.slot.as_mut().expect("OccupiedEntry must contain Some")
  }

  pub fn into_mut(self) -> &'a mut T {
    self.slot.as_mut().expect("OccupiedEntry must contain Some")
  }

  pub fn insert(&mut self, value: T) -> T {
    self.slot.replace(value).expect("OccupiedEntry must contain Some")
  }

  pub fn remove(self) -> T {
    self.slot.take().expect("OccupiedEntry must contain Some")
  }
}

impl<'a, T> VacantEntry<'a, T> {
  pub fn key(&self) -> u64 {
    self.id
  }

  pub fn insert(self, value: T) -> &'a mut T {
    let idx = self.id as usize;
    if idx >= self.table.items.len() {
      self.table.items.resize_with(idx + 1, || None);
    }
    self.table.items[idx].insert(value)
  }
}

impl<T> Table<T> {
  pub fn new() -> Self {
    Self {
      sequence: Default::default(),
      active:   0,
      items:    Vec::new(),
    }
  }

  pub fn active(&mut self) -> Option<&mut T> {
    self.get_mut(self.active)
  }

  pub fn set_active(&mut self, active: u64) -> bool {
    if active < self.items.len() as u64 {
      self.active = active;
      true
    } else { false }
  }

  pub fn insert(&mut self, id: u64, item: T) {
    let idx = id as usize;
    if idx >= self.items.len() {
      self.items.resize_with(idx + 1, || None);
    }
    self.items[idx] = Some(item);
  }

  pub fn get(&self, id: u64) -> Option<&T> {
    self.items.get(id as usize)?.as_ref()
  }

  pub fn get_mut(&mut self, id: u64) -> Option<&mut T> {
    self.items.get_mut(id as usize)?.as_mut()
  }

  pub fn entry(&mut self, id: u64) -> Entry<'_,T> {
    let idx = id as usize;
    if self.items.get(idx).is_some_and(|item| item.is_some()) {
      Entry::Occupied(OccupiedEntry { id: id, slot: &mut self.items[idx] })
    } else {
      Entry::Vacant(VacantEntry { id: id, table: self })
    }
  }

  pub fn remove(&mut self, id: u64) -> Option<T> {
    self.items.get_mut(id as usize)?.take()
  }

  pub fn values(&self) -> impl Iterator<Item = &T> {
    self.items.iter().flatten()
  }

  pub fn values_mut(&mut self) -> impl Iterator<Item = &mut T> {
    self.items.iter_mut().flatten()
  }

  pub fn next(&mut self) -> u64 {
    self.sequence.next()
  }

  pub fn len(&self) -> usize {
    self.items.iter().flatten().count()
  }

  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  pub fn keys(&self) -> impl Iterator<Item = u64> + '_ {
    self.items.iter().enumerate().filter_map(|(id, item)| item.as_ref().map(|_| id as u64))
  }

  pub fn iter(&self) -> Iter<'_,T> {
    self.into_iter()
  }

  pub fn iter_mut(&mut self) -> IterMut<'_,T> {
    self.into_iter()
  }
}

