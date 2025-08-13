/// Nicebot is a Rust crate for parsing `robots.txt` files. It is performant and lean; all dependencies are optional.
///
/// ```rust
/// use nicebot::{SingleBot, Permission};
///
/// let robots_txt = r"
///     User-Agent: *
///     Allow: /aaa
///     Disallow: /bbb
/// ";
///
/// let bot = SingleBot::from(robots_txt);
///
/// assert_eq!(bot.check("/aaa"), Permission::Allowed);
/// assert_eq!(bot.check("/bbb"), Permission::Denied);
/// assert_eq!(bot.check("/ccc"), Permission::Unspecified);
/// ```
///
/// Filtering several sites (requires feature `meta`, enabled by default):
/// ```rust
/// use nicebot::{MetaBot, Permission, AddRobots, TryAddRobots, CheckURL};
///
/// # fn main() -> Result<(), nicebot::CheckError> {
/// let mut meta = MetaBot::new(None);
///
/// let robots_gmail = r#"
///     User-agent: *
///     Allow: /
///     Disallow: /a/*
///     Disallow: /mail?hl=*
///     Disallow: /tasks/*
///
///     Sitemap: https://www.google.com/gmail/sitemap.xml
/// "#;
///
/// let robots_reddit = r#"
///     User-agent: *
///     Disallow: /
/// "#;
///
/// meta.try_add_robots("gmail.com", robots_gmail);
/// meta.try_add_robots("www.reddit.com", robots_reddit);
///
/// assert_eq!(meta.check("https://gmail.com/abc")?, Permission::Allowed);
/// assert_eq!(meta.check("https://www.reddit.com/abc")?, Permission::Denied);
/// assert_eq!(meta.check("https://old.reddit.com/abc")?, Permission::Unspecified);
/// # Ok(())
/// # }
/// ```
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
