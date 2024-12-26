use std::env::consts::OS;

use crate::{error::Error, json::version::meta::vanilla::{Action, Name, Rule}};

use super::TARGET_ARCH;

pub trait ParseRule {
    fn parse_rule(&self) -> bool;
}

impl ParseRule for [Rule] {
    fn parse_rule(&self) -> bool {
        let parsed_os: Name = match OS {
            "linux" => Name::Linux,
            "windows" => Name::Windows,
            "macos" => Name::Osx,
            _ => panic!("Unknown operating system!"),
        };

        if self.is_empty() {
            true
        } else {
            let mut should_push = false;
            for rule in self {
                if rule.action == Action::Disallow {
                    if let Some(os) = &rule.os {
                        if os.name.is_some()
                            && os.name != Some(parsed_os.clone())
                            && os.arch.is_some()
                            && os.arch != Some(TARGET_ARCH.to_string())
                        {
                            continue;
                        } else {
                            break;
                        }
                    } else {
                        continue;
                    }
                } else if rule.action == Action::Allow {
                    if let Some(os) = &rule.os {
                        if (os.name.is_some() && os.name != Some(parsed_os.clone()))
                            || (os.arch.is_some() && os.arch != Some(TARGET_ARCH.to_string()))
                        {
                            continue;
                        } else {
                            should_push = true;
                            break;
                        }
                    } else {
                        should_push = true;
                        continue;
                    }
                }
            }
            should_push
        }
    }
}

impl ParseRule for Option<Vec<Rule>> {
    fn parse_rule(&self) -> bool {
        match self {
            Some(rules) => {
                let parsed_os: Name = match OS {
                    "linux" => Name::Linux,
                    "windows" => Name::Windows,
                    "macos" => Name::Osx,
                    _ => panic!("Unknown operating system!"),
                };

                if rules.is_empty() {
                    true
                } else {
                    let mut should_push = false;
                    for rule in rules {
                        if rule.action == Action::Disallow {
                            if let Some(os) = &rule.os {
                                if os.name.is_some()
                                    && os.name != Some(parsed_os.clone())
                                    && os.arch.is_some()
                                    && os.arch != Some(TARGET_ARCH.to_string())
                                {
                                    continue;
                                } else {
                                    break;
                                }
                            } else {
                                continue;
                            }
                        } else if rule.action == Action::Allow {
                            if let Some(os) = &rule.os {
                                if (os.name.is_some() && os.name != Some(parsed_os.clone()))
                                    || (os.arch.is_some()
                                        && os.arch != Some(TARGET_ARCH.to_string()))
                                {
                                    continue;
                                } else {
                                    should_push = true;
                                    break;
                                }
                            } else {
                                should_push = true;
                                continue;
                            }
                        }
                    }
                    should_push
                }
            }
            None => true,
        }
    }
}

pub fn parse_lib_path(artifact: &str) -> crate::Result<String> {
    let name_items: Vec<&str> = artifact.split(':').collect();
    if name_items.len() < 3 {
        return Err(Error::Parse(format!("Invalid artifact format: {}", artifact)));
    }

    let package = name_items[0];
    let name = name_items[1];
    let version_ext: Vec<&str> = name_items[2].split('@').collect();
    let version = version_ext[0];
    let ext = version_ext.get(1).unwrap_or(&"jar");

    if name_items.len() == 3 {
        Ok(format!(
            "{}/{}/{}/{}-{}.{}",
            package.replace('.', "/"),
            name,
            version,
            name,
            version,
            ext
        ))
    } else {
        let data_ext: Vec<&str> = name_items[3].split('@').collect();
        let data = data_ext[0];
        let data_ext = data_ext.get(1).unwrap_or(&"jar");

        Ok(format!(
            "{}/{}/{}/{}-{}-{}.{}",
            package.replace('.', "/"),
            name,
            version,
            name,
            version,
            data,
            data_ext
        ))
    }
}