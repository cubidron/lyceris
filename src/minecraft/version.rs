use std::env::consts::OS;

use crate::json::version::meta::vanilla::{Action, Name, Os, Rule};

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
                    if !should_push {
                        println!("Not allowed: {:?}", rules);
                    }
                    should_push
                }
            }
            None => true,
        }
    }
}
