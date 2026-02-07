// sw: terminal stopwatch
// copyright (C) 2022-2023 Ula Shipman <ula.hello@mailbox.org>
// licensed under GPL-3.0-or-later

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
    Visuals,
    License,
    Quit,
    QuitAbrupt,
}

#[allow(clippy::enum_glob_use)]
use Command::*;

impl Command {
    pub const fn short_name_literal(self) -> &'static str {
        match self {
            Help => "h",
            Display => "",
            Toggle => "s",
            Reset => "r",
            Change => "c",
            Offset => "o",
            Name => "n",
            Precision => "p",
            Visuals => "v",
            License => "l",
            Quit | QuitAbrupt => "q",
        }
    }

    pub const fn short_name_display(self) -> &'static str {
        match self {
            Display => "<Enter>",
            _ => self.short_name_literal(),
        }
    }

    pub const fn long_name(self) -> &'static str {
        match self {
            Help => "help",
            Display => "display",
            Toggle => "toggle",
            Reset => "reset",
            Change => "change",
            Offset => "offset",
            Name => "name",
            Precision => "precision",
            Visuals => "visuals",
            License => "license",
            Quit | QuitAbrupt => "quit",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Help => "show help",
            Display => "display elapsed time",
            Toggle => "toggle stopwatch",
            Reset => "reset stopwatch",
            Change => "change elapsed time",
            Offset => "offset elapsed time",
            Name => "name stopwatch",
            Precision => "set display precision",
            Visuals => "toggle visual cues",
            License => "print license info",
            Quit | QuitAbrupt => "Abandon all Data",
        }
    }

    pub const fn iter() -> &'static [Self] {
        &[
            Help, Display, Toggle, Reset, Change, Offset, Name, Precision, Visuals, License, Quit,
        ]
    }
}

impl FromStr for Command {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        let s = s.trim().to_lowercase();
        for cmd in Self::iter() {
            if s == cmd.short_name_literal() || s == cmd.long_name() {
                return Ok(*cmd);
            }
        }
        Err(())
    }
}
