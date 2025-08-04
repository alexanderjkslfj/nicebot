mod trie;
pub use trie::PrefixTrie;

mod singlebot;
pub use singlebot::SingleBot;

#[cfg(feature = "meta")]
mod metabot;
#[cfg(feature = "meta")]
pub use metabot::{AddRobots, CheckError, CheckURL, MetaBot, TryAddRobots};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Permission {
    Allowed,
    Denied,
    Unspecified,
}
impl Default for Permission {
    fn default() -> Self {
        Self::Unspecified
    }
}
