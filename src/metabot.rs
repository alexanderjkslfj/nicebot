use std::{collections::HashMap, ops::AddAssign};

use url::{Host, ParseError, Url};

use crate::{Permission, SingleBot};

pub struct MetaBot {
    hosts: HashMap<Host<String>, SingleBot>,
    user_agent: Option<String>,
}

pub trait AddRobots<T> {
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

pub trait TryAddRobots<T, Q> {
    fn try_add_robots(&mut self, host: Q, robots_txt: T) -> bool;
}

impl<T> TryAddRobots<T, &str> for MetaBot
where
    MetaBot: AddRobots<T>,
{
    fn try_add_robots(&mut self, host: &str, robots_txt: T) -> bool {
        let Ok(parsed_host) = Host::parse(host) else {
            return false;
        };
        self.add_robots(parsed_host, robots_txt);
        return true;
    }
}

impl<T> TryAddRobots<T, &String> for MetaBot
where
    MetaBot: AddRobots<T>,
{
    fn try_add_robots(&mut self, host: &String, robots_txt: T) -> bool {
        let Ok(parsed_host) = Host::parse(host.as_str()) else {
            return false;
        };
        self.add_robots(parsed_host, robots_txt);
        return true;
    }
}

impl<T> TryAddRobots<T, String> for MetaBot
where
    MetaBot: AddRobots<T>,
{
    fn try_add_robots(&mut self, host: String, robots_txt: T) -> bool {
        let Ok(parsed_host) = Host::parse(host.as_str()) else {
            return false;
        };
        self.add_robots(parsed_host, robots_txt);
        return true;
    }
}

pub trait CheckURL<T> {
    fn check(&self, url: T) -> Result<Permission, CheckError>;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CheckError {
    ParseError(ParseError),
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
    pub fn new(user_agent: Option<String>) -> Self {
        let hosts = HashMap::new();
        Self { hosts, user_agent }
    }

    pub fn trim(&mut self) {
        for bot in self.hosts.values_mut() {
            bot.trim();
        }
    }
}
