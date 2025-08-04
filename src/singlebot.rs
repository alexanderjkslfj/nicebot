use std::{cell::Cell, ops::AddAssign};

#[cfg(feature = "async")]
use futures_lite::stream::{Stream, StreamExt};
#[cfg(feature = "percent-decoding")]
use percent_encoding::percent_decode_str;

use crate::PrefixTrie;

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

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum Match {
    Star,
    Yes,
    No,
}

impl AddAssign<String> for SingleBot {
    fn add_assign(&mut self, rhs: String) {
        let captures = Self::capture_str(rhs.as_str());
        self.extend_prefixes(captures);
    }
}

impl From<String> for SingleBot {
    fn from(value: String) -> Self {
        let mut bot = Self::new(None);
        bot += value;
        bot
    }
}

impl AddAssign<&String> for SingleBot {
    fn add_assign(&mut self, rhs: &String) {
        let captures = Self::capture_str(rhs.as_str());
        self.extend_prefixes(captures);
    }
}

impl From<&String> for SingleBot {
    fn from(value: &String) -> Self {
        let mut bot = Self::new(None);
        bot += value;
        bot
    }
}

impl AddAssign<&str> for SingleBot {
    fn add_assign(&mut self, rhs: &str) {
        let captures = Self::capture_str(rhs);
        self.extend_prefixes(captures);
    }
}

impl From<&str> for SingleBot {
    fn from(value: &str) -> Self {
        let mut bot = Self::new(None);
        bot += value;
        bot
    }
}

impl AddAssign<std::fs::File> for SingleBot {
    fn add_assign(&mut self, rhs: std::fs::File) {
        let captures = Self::capture_file(rhs);
        self.extend_prefixes(captures);
    }
}

impl From<std::fs::File> for SingleBot {
    fn from(value: std::fs::File) -> Self {
        let mut bot = Self::new(None);
        bot += value;
        bot
    }
}

impl<T> AddAssign<std::io::BufReader<T>> for SingleBot
where
    T: std::io::Read,
{
    fn add_assign(&mut self, rhs: std::io::BufReader<T>) {
        let captures = Self::capture_reader(rhs);
        self.extend_prefixes(captures);
    }
}

impl<T> From<std::io::BufReader<T>> for SingleBot
where
    T: std::io::Read,
{
    fn from(value: std::io::BufReader<T>) -> Self {
        let mut bot = Self::new(None);
        bot += value;
        bot
    }
}

impl Default for SingleBot {
    fn default() -> Self {
        SingleBot::new(None)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SingleBot {
    prefixes: PrefixTrie<Permission>,
    user_agent: Option<String>,
}

impl SingleBot {
    pub fn new(user_agent: Option<String>) -> Self {
        let mut prefixes = PrefixTrie::new();
        prefixes.insert("", Permission::Unspecified);
        SingleBot {
            prefixes,
            user_agent,
        }
    }

    #[cfg(feature = "async-tokio")]
    pub async fn add_file_tokio(&mut self, robots_txt: tokio::fs::File) {
        let captures = Self::capture_file_tokio(robots_txt);
        self.extend_prefixes_async(captures).await;
    }

    #[cfg(feature = "async-async-std")]
    pub async fn add_file_asyncstd(&mut self, robots_txt: async_std::fs::File) {
        let captures = Self::capture_file_asyncstd(robots_txt);
        self.extend_prefixes_async(captures).await;
    }

    #[cfg(feature = "async-smol")]
    pub async fn add_file_smol(&mut self, robots_txt: smol::fs::File) {
        let captures = Self::capture_file_smol(robots_txt);
        self.extend_prefixes_async(captures).await;
    }

    #[cfg(feature = "async")]
    pub async fn from_reader_async(
        &mut self,
        robots_txt: futures_lite::io::BufReader<impl futures_lite::AsyncBufReadExt + Unpin>,
    ) {
        let captures = Self::capture_reader_async(robots_txt);
        self.extend_prefixes_async(captures).await;
    }

    fn decode((op, mut val): (String, String)) -> (String, String) {
        if val.ends_with('$') {
            val.pop();
            val.push('\0');
        }

        #[cfg(not(feature = "percent-decoding"))]
        return (op, val);

        #[cfg(feature = "percent-decoding")]
        if let Ok(decoded) = percent_decode_str(&val).decode_utf8() {
            return (op, decoded.into_owned());
        } else {
            return (op, val);
        }
    }

    fn conform(
        state: &mut Match,
        precise: &Cell<bool>,
        user_agent: Option<&String>,
        (op, val): (String, String),
    ) -> Option<Option<(Match, (String, String))>> {
        match op.as_str() {
            "user-agent" => {
                if val == "*" {
                    *state = Match::Star;
                } else if let Some(ua) = user_agent
                    && val.to_lowercase().contains(ua)
                {
                    *state = Match::Yes;
                    precise.set(true);
                } else {
                    *state = Match::No;
                }
                Some(None)
            }
            "allow" | "disallow" => {
                if matches!(state, Match::No) {
                    Some(None)
                } else {
                    Some(Some((state.clone(), (op, val))))
                }
            }
            _ => Some(None),
        }
    }

    fn filter_weak(
        result: Option<(Match, (String, String))>,
        precise: bool,
    ) -> Option<(String, String)> {
        match result {
            Some((Match::Yes, group)) => Some(group),
            Some((Match::Star, group)) if !precise => Some(group),
            _ => None,
        }
    }

    fn extend_prefix_trie(prefixes: &mut PrefixTrie<Permission>, (op, val): (String, String)) {
        match op.as_str() {
            "allow" => prefixes.insert(&val, Permission::Allowed),
            "disallow" => prefixes.insert(&val, Permission::Denied),
            _ => unreachable!(),
        };
    }

    #[cfg(feature = "async")]
    async fn extend_prefixes_async(&mut self, captures: impl Stream<Item = (String, String)>) {
        if self.user_agent.is_some() {
            self.extend_prefixes_with_user_agent_async(captures).await
        } else {
            self.extend_prefixes_without_user_agent_async(captures)
                .await
        }
    }

    async fn extend_prefixes_without_user_agent_async(
        &mut self,
        captures: impl Stream<Item = (String, String)>,
    ) {
        captures
            .map(Self::decode)
            .scan(false, |state, (op, val)| match op.as_str() {
                "user-agent" => {
                    *state = val == "*";
                    Some(None)
                }
                "allow" if *state => Some(Some((true, val))),
                "disallow" if *state => Some(Some((false, val))),
                _ => Some(None),
            })
            .filter_map(|opt| opt)
            .for_each(|(allow, val): (bool, String)| {
                self.prefixes.insert(
                    &val,
                    if allow {
                        Permission::Allowed
                    } else {
                        Permission::Denied
                    },
                );
            })
            .await;
    }

    async fn extend_prefixes_with_user_agent_async(
        &mut self,
        captures: impl Stream<Item = (String, String)>,
    ) {
        let precise = Cell::new(false);

        captures
            .map(Self::decode)
            .scan(Match::No, |state, (op, val)| {
                Self::conform(state, &precise, self.user_agent.as_ref(), (op, val))
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .filter_map(|result| Self::filter_weak(result, precise.get()))
            .for_each(|pair| Self::extend_prefix_trie(&mut self.prefixes, pair));
    }

    fn extend_prefixes(&mut self, captures: impl Iterator<Item = (String, String)>) {
        if self.user_agent.is_some() {
            self.extend_prefixes_with_user_agent(captures);
        } else {
            self.extend_prefixes_without_user_agent(captures);
        }
    }

    fn extend_prefixes_without_user_agent(
        &mut self,
        captures: impl Iterator<Item = (String, String)>,
    ) {
        captures
            .map(Self::decode)
            .scan(false, |state, (op, val)| match op.as_str() {
                "user-agent" => {
                    *state = val == "*";
                    Some(None)
                }
                "allow" if *state => Some(Some((true, val))),
                "disallow" if *state => Some(Some((false, val))),
                _ => Some(None),
            })
            .filter_map(|opt| opt)
            .for_each(|(allow, val): (bool, String)| {
                self.prefixes.insert(
                    &val,
                    if allow {
                        Permission::Allowed
                    } else {
                        Permission::Denied
                    },
                );
            });
    }

    fn extend_prefixes_with_user_agent(
        &mut self,
        captures: impl Iterator<Item = (String, String)>,
    ) {
        let precise = Cell::new(false);

        captures
            .map(Self::decode)
            .scan(Match::No, |state, (op, val)| {
                Self::conform(state, &precise, self.user_agent.as_ref(), (op, val))
            })
            .collect::<Vec<_>>()
            .into_iter()
            .filter_map(|result| Self::filter_weak(result, precise.get()))
            .for_each(|pair| Self::extend_prefix_trie(&mut self.prefixes, pair));
    }

    /// Trims the internal data structure, saving a few bytes.
    pub fn trim(&mut self) {
        self.prefixes.shrink();
    }

    /// Checks the permission defined for a specific URL.
    pub fn check(&self, url: &str) -> Permission {
        self.prefixes.get(url).unwrap()
    }

    #[cfg(feature = "async-smol")]
    fn capture_file_smol(file: smol::fs::File) -> impl Stream<Item = (String, String)> {
        let reader = smol::io::BufReader::new(file);
        Self::capture_reader_async(reader)
    }

    #[cfg(feature = "async-async-std")]
    fn capture_file_asyncstd(file: async_std::fs::File) -> impl Stream<Item = (String, String)> {
        let reader = async_std::io::BufReader::new(file);
        Self::capture_reader_async(reader)
    }

    #[cfg(feature = "async-tokio")]
    fn capture_file_tokio(file: tokio::fs::File) -> impl Stream<Item = (String, String)> {
        use tokio_util::compat::TokioAsyncReadCompatExt;

        let reader = tokio::io::BufReader::new(file);
        let compat = reader.compat();
        Self::capture_reader_async(compat)
    }

    #[cfg(feature = "async")]
    fn capture_reader_async(
        reader: impl futures_lite::AsyncBufReadExt + Unpin,
    ) -> impl Stream<Item = (String, String)> {
        let lines = reader.lines().filter_map(|line| line.ok());
        Self::capture_lines_async(lines)
    }

    #[cfg(feature = "async")]
    fn capture_lines_async(
        lines: impl Stream<Item = String>,
    ) -> impl Stream<Item = (String, String)> {
        lines
            .map(strip_comment)
            .filter_map(|line| parse_pair(line))
            .filter_map(|pair| filter_and_normalize(pair))
    }

    fn capture_file(file: std::fs::File) -> impl Iterator<Item = (String, String)> {
        let reader = std::io::BufReader::new(file);
        Self::capture_reader(reader)
    }

    fn capture_reader(reader: impl std::io::BufRead) -> impl Iterator<Item = (String, String)> {
        let lines = reader.lines().filter_map(|line| line.ok());
        Self::capture_lines(lines)
    }

    fn capture_str(input: &str) -> impl Iterator<Item = (String, String)> {
        let lines = input.lines();
        Self::capture_lines_str(lines)
    }

    fn capture_lines<'a>(
        lines: impl IntoIterator<Item = String>,
    ) -> impl Iterator<Item = (String, String)> {
        lines
            .into_iter()
            .map(strip_comment)
            .filter_map(parse_pair)
            .filter_map(filter_and_normalize)
    }

    fn capture_lines_str<'a>(
        lines: impl IntoIterator<Item = &'a str>,
    ) -> impl Iterator<Item = (String, String)> {
        lines
            .into_iter()
            .map(strip_comment_str)
            .filter_map(parse_pair_str)
            .map(|(op, val)| (op.trim().to_lowercase(), val.trim()))
            .filter(|(_, val)| !val.contains(' '))
            .filter(|(op, _)| op == "allow" || op == "disallow" || op == "user-agent")
            .map(|(op, val)| (op, val.to_owned()))
    }
}

fn strip_comment_str(line: &str) -> &str {
    match line.find('#') {
        Some(idx) => &line[..idx],
        None => line,
    }
}

fn strip_comment(mut line: String) -> String {
    if let Some(idx) = line.find('#') {
        line.truncate(idx);
    }
    line
}

fn parse_pair_str(line: &str) -> Option<(&str, &str)> {
    if line.is_empty() {
        None
    } else {
        let mut parts = line.splitn(2, ':');
        let op = parts.next()?;
        let val = parts.next()?;
        Some((op, val))
    }
}

fn parse_pair(mut line: String) -> Option<(String, String)> {
    if line.is_empty() {
        return None;
    }
    let idx = line.find(':')?;
    let mut val = line.split_off(idx);
    val.remove(0);
    let op = line;
    Some((op, val))
}

fn filter_and_normalize((op, val): (String, String)) -> Option<(String, String)> {
    let op_trim = op.trim().to_lowercase();
    if !(op_trim == "allow" || op_trim == "disallow" || op_trim == "user-agent") {
        return None;
    }
    let val_trim = val.trim();
    if val_trim.contains(' ') {
        return None;
    }
    Some((op_trim, val_trim.to_owned()))
}
