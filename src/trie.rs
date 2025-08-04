use std::{collections::HashMap, fmt::Debug};

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
    pub fn new() -> Self {
        PrefixTrie {
            value: None,
            exact: None,
            wildcard: None,
            children: HashMap::new(),
        }
    }

    pub fn shrink(&mut self) {
        self.children.shrink_to_fit();
        for child in self.children.values_mut() {
            child.shrink();
        }
    }

    pub fn insert(&mut self, key: &str, value: T) -> Option<T> {
        self.insert_chars(key.chars(), value)
    }

    pub fn insert_chars(&mut self, mut key: impl Iterator<Item = char>, value: T) -> Option<T> {
        if let Some(letter) = key.next() {
            if letter == '*' {
                if let Some(child) = &mut self.wildcard {
                    return child.insert_chars(key, value);
                } else {
                    let mut child = PrefixTrie::new();
                    child.insert_chars(key, value);
                    self.wildcard = Some(Box::new(child));
                    return None;
                }
            } else if letter == '\0' {
                return self.exact.replace(value);
            } else if let Some(child) = self.children.get_mut(&letter) {
                return child.insert_chars(key, value);
            } else {
                let mut child = PrefixTrie::new();
                child.insert_chars(key, value);
                self.children.insert(letter, child);
                return None;
            }
        } else {
            return self.value.replace(value);
        }
    }

    pub fn get(&self, key: &str) -> Option<T> {
        self.get_chars(key.chars())
    }

    pub fn get_chars(&self, key: impl Iterator<Item = char> + Clone) -> Option<T> {
        self.get_chars_depth(key).and_then(|val| Some(val.0))
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

        if let Some(value) = self.value {
            return Some((value, 0));
        } else {
            return None;
        };
    }

    pub fn has(&self, key: &str) -> bool {
        self.has_chars(key.chars())
    }

    pub fn has_chars(&self, key: impl Iterator<Item = char> + Clone) -> bool {
        self.get_chars(key).is_some()
    }
}
