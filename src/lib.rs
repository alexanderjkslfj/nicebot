mod trie;
pub use trie::PrefixTrie;

mod singlebot;
pub use singlebot::{Permission, SingleBot};

mod nicebot;
pub use nicebot::{AddRobots, CheckError, CheckURL, NiceBot, TryAddRobots};
