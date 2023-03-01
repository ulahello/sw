// sw: terminal stopwatch
// copyright (C) 2022-2023 Ula Shipman <ula.hello@mailbox.org>
// licensed under GPL-3.0-or-later

use core::fmt;
use core::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Display,
    Toggle,
    Reset,
    Change,
    Offset,
    Name,
    Precision,
    License,
    Quit,
}

use Command::*;

impl Command {
    pub fn as_str(&self) -> &'static str {
        match self {
            Help => "h",
            Display => "",
            Toggle => "s",
            Reset => "r",
            Change => "c",
            Offset => "o",
            Name => "n",
            Precision => "p",
            License => "l",
            Quit => "q",
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Command {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        match s {
            "h" => Ok(Help),
            "" => Ok(Display),
            "s" => Ok(Toggle),
            "r" => Ok(Reset),
            "c" => Ok(Change),
            "o" => Ok(Offset),
            "n" => Ok(Name),
            "p" => Ok(Precision),
            "l" => Ok(License),
            "q" => Ok(Quit),
            _ => Err(()),
        }
    }
}
