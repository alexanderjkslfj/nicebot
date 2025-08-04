#[cfg(test)]
mod tests {
    use nicebot::PrefixTrie;

    #[test]
    fn trie_get_exact() {
        let mut trie = PrefixTrie::new();

        trie.insert("apple", 1);
        trie.insert("banana", 2);
        trie.insert("citrus", 3);

        let apple = trie.get("apple");
        let banana = trie.get("banana");
        let citrus = trie.get("citrus");

        assert_eq!(apple, Some(1));
        assert_eq!(banana, Some(2));
        assert_eq!(citrus, Some(3));

        let date = trie.get("date");

        assert_eq!(date, None);
    }

    #[test]
    fn trie_get_longer() {
        let mut trie = PrefixTrie::new();

        trie.insert("abc", 5);

        let value = trie.get("abcdefg");

        assert_eq!(value, Some(5));
    }

    #[test]
    fn trie_get_shorter() {
        let mut trie = PrefixTrie::new();

        trie.insert("abc", 5);

        let value = trie.get("a");

        assert_eq!(value, None);
    }

    #[test]
    fn trie_get_empty() {
        let mut trie = PrefixTrie::new();

        trie.insert("", 1);

        let value = trie.get("");

        assert_eq!(value, Some(1));
    }

    #[test]
    fn trie_has_exact() {
        let mut trie = PrefixTrie::new();

        trie.insert("abc", 5);

        assert!(trie.has("abc"));
    }

    #[test]
    fn trie_has_longer() {
        let mut trie = PrefixTrie::new();

        trie.insert("abc", 5);

        assert!(trie.has("abcdef"));
    }

    #[test]
    fn trie_has_shorter() {
        let mut trie = PrefixTrie::new();

        trie.insert("abc", 5);

        assert!(!trie.has("a"));
    }

    #[test]
    fn trie_prioritize_exact() {
        let mut trie = PrefixTrie::new();

        trie.insert("/aaa", 0);
        trie.insert("/aaa\0", 1);
        trie.insert("*/aaa", 2);
        trie.insert("*/aaa\0", 3);
        trie.insert("/aaa*", 4);
        trie.insert("*/aaa*", 5);

        assert_eq!(trie.get("/aaa"), Some(1));
        assert_eq!(trie.get("/aaa/xxx"), Some(4));
        assert_eq!(trie.get("/xxx/aaa"), Some(3));
        assert_eq!(trie.get("/xxx/aaa/xxx"), Some(5));
        assert_eq!(trie.get("/xxx"), None);
    }

    #[test]
    fn trie_double_wildcard() {
        let mut trie = PrefixTrie::new();

        trie.insert("*/abc*", 5);

        assert!(!trie.has("abc"));
        assert!(!trie.has("/axxxbc"));
        assert!(!trie.has("/xabc"));
        assert!(trie.has("/abc"));
        assert!(trie.has("/x/abc"));
        assert!(trie.has("/abc/x"));
        assert!(trie.has("/abcx"));
        assert!(trie.has("/x/abc/x"));
    }
}
