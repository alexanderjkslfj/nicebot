use std::{collections::HashMap, fmt::Debug};

/// Prefix trie that supports Wildcards and Exacts
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrefixTrie<T: Copy> {
    value: Option<T>,
    exact: Option<T>,
    wildcard: Option<Box<PrefixTrie<T>>>,
    children: HashMap<char, PrefixTrie<T>>,
}

impl<T: Copy> Default for PrefixTrie<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Copy> PrefixTrie<T> {
    /// Creates a new [`PrefixTrie`].
    pub fn new() -> Self {
        PrefixTrie {
            value: None,
            exact: None,
            wildcard: None,
            children: HashMap::new(),
        }
    }

    /// Shrinks the internal data structure, saving a few bytes.
    pub fn shrink(&mut self) {
        self.children.shrink_to_fit();
        for child in self.children.values_mut() {
            child.shrink();
        }
    }

    /// Inserts a value at a given prefix.
    /// `*` symbols are interpreted as wildcards.
    /// A `\0` byte allows only for exact matching. (Anything after a `\0` byte is ignored.)
    pub fn insert(&mut self, key: &str, value: T) -> Option<T> {
        self.insert_chars(key.chars(), value)
    }

    /// Inserts a value at a given prefix.
    /// `*` symbols are interpreted as wildcards.
    /// A `\0` byte allows only for exact matching. (Anything after a `\0` byte is ignored.)
    pub fn insert_chars(&mut self, mut key: impl Iterator<Item = char>, value: T) -> Option<T> {
        if let Some(letter) = key.next() {
            if letter == '*' {
                if let Some(child) = &mut self.wildcard {
                    child.insert_chars(key, value)
                } else {
                    let mut child = PrefixTrie::new();
                    child.insert_chars(key, value);
                    self.wildcard = Some(Box::new(child));
                    None
                }
            } else if letter == '\0' {
                self.exact.replace(value)
            } else if let Some(child) = self.children.get_mut(&letter) {
                child.insert_chars(key, value)
            } else {
                let mut child = PrefixTrie::new();
                child.insert_chars(key, value);
                self.children.insert(letter, child);
                None
            }
        } else {
            self.value.replace(value)
        }
    }

    /// Gets the value with the most precise matching prefix
    pub fn get(&self, key: &str) -> Option<T> {
        self.get_chars(key.chars())
    }

    /// Gets the value with the most precise matching prefix
    pub fn get_chars(&self, key: impl Iterator<Item = char> + Clone) -> Option<T> {
        self.get_chars_depth(key).map(|val| val.0)
    }

    fn get_chars_depth(&self, mut key: impl Iterator<Item = char> + Clone) -> Option<(T, usize)> {
        let mut biggest_result: Option<(T, usize)> = None;

        if let Some(wild) = &self.wildcard {
            let mut sub_key = key.clone();
            loop {
                if let Some(result) = wild.get_chars_depth(sub_key.clone()) {
                    if biggest_result.is_none() || biggest_result.unwrap().1 > result.1 {
                        biggest_result = Some(result);
                    }
                }
                if sub_key.next().is_none() {
                    break;
                }
            }
        }

        if let Some(letter) = key.next() {
            if let Some(child) = self.children.get(&letter) {
                if let Some(result) = child.get_chars_depth(key.clone()) {
                    if biggest_result.is_none() || biggest_result.unwrap().1 > result.1 {
                        biggest_result = Some(result);
                    }
                }
            }
        } else if let Some(exact) = self.exact {
            return Some((exact, 1000));
        }

        if let Some(result) = biggest_result {
            return Some((result.0, result.1 + 1));
        }

        self.value.map(|value| (value, 0))
    }

    /// Checks if key can be found
    pub fn has(&self, key: &str) -> bool {
        self.has_chars(key.chars())
    }

    /// Checks if key can be found
    pub fn has_chars(&self, key: impl Iterator<Item = char> + Clone) -> bool {
        self.get_chars(key).is_some()
    }
}
