use std::{collections::HashMap, ops::AddAssign};

use url::{Host, ParseError, Url};

use crate::{Permission, SingleBot};

#[derive(Clone, Debug, PartialEq, Eq)]
/// Used to check the `robots.txt`s of multiple Hosts.
pub struct MetaBot {
    hosts: HashMap<Host<String>, SingleBot>,
    user_agent: Option<String>,
}

/// Allows for adding `robots.txt`s.
pub trait AddRobots<T> {
    /// Adds a `robots.txt` for the specified host.
    fn add_robots(&mut self, host: Host<String>, robots_txt: T);
}

impl<T> AddRobots<T> for MetaBot
where
    SingleBot: AddAssign<T>,
{
    fn add_robots(&mut self, host: Host<String>, robots_txt: T) {
        let bot = self
            .hosts
            .entry(host)
            .or_insert_with(|| SingleBot::new(self.user_agent.clone()));
        bot.add_assign(robots_txt);
    }
}

/// Allows for adding `robots.txt`s.
pub trait TryAddRobots<T, Q> {
    /// Adds a `robots.txt` for the specified host.
    /// Returns `false` if parsing the host fails.
    fn try_add_robots(&mut self, host: Q, robots_txt: T) -> bool;
}

impl<T, Q> TryAddRobots<T, Q> for MetaBot
where
    MetaBot: AddRobots<T>,
    Q: AsRef<str>,
{
    fn try_add_robots(&mut self, host: Q, robots_txt: T) -> bool {
        let Ok(parsed_host) = Host::parse(host.as_ref()) else {
            return false;
        };
        self.add_robots(parsed_host, robots_txt);
        true
    }
}

/// Allows for checking the permissions for a URL.
pub trait CheckURL<T> {
    /// Checks the permissions for a URL.
    /// # Errors
    /// Will return `Err` if URL parsing fails or URL doesn't contain host.
    fn check(&self, url: T) -> Result<Permission, CheckError>;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// Error if checking fails because of an invalid URL.
pub enum CheckError {
    /// Failed at parsing the input as a URL.
    ParseError(ParseError),
    /// The provided URL did not contain a host.
    MissingHost,
}

impl CheckURL<Url> for MetaBot {
    fn check(&self, url: Url) -> Result<Permission, CheckError> {
        if let Some(host) = url.host() {
            if let Some(bot) = self.hosts.get(&host.to_owned()) {
                Ok(bot.check(url.path()))
            } else {
                Ok(Permission::Unspecified)
            }
        } else {
            Err(CheckError::MissingHost)
        }
    }
}

impl CheckURL<&str> for MetaBot {
    fn check(&self, url: &str) -> Result<Permission, CheckError> {
        match Url::parse(url) {
            Ok(parsed) => self.check(parsed),
            Err(err) => Err(CheckError::ParseError(err)),
        }
    }
}

impl CheckURL<&String> for MetaBot {
    fn check(&self, url: &String) -> Result<Permission, CheckError> {
        self.check(url.as_str())
    }
}

impl CheckURL<String> for MetaBot {
    fn check(&self, url: String) -> Result<Permission, CheckError> {
        self.check(url.as_str())
    }
}

impl MetaBot {
    /// Creates a new [`MetaBot`].
    /// [`MetaBot`] is used to check multiple Hosts. If checking only on a single host, use [`crate::SingleBot`]
    pub fn new(user_agent: Option<String>) -> Self {
        let hosts = HashMap::new();
        Self { hosts, user_agent }
    }

    /// Shrinks the internal data structure, saving a few bytes.
    pub fn shrink(&mut self) {
        for bot in self.hosts.values_mut() {
            bot.shrink();
        }
    }
}
