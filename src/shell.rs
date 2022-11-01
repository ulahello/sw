// sw: terminal stopwatch
// copyright (C) 2022 Ula Shipman <ula.hello@mailbox.org>
// licensed under GPL-3.0-or-later

use crate::parse::ReadDur;

use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

use core::fmt;
use std::io::{self, stderr, stdin, stdout, BufRead, BufWriter, Read, Write};

const READ_LIMIT: u64 = 64;

pub fn write(s: impl fmt::Display) -> io::Result<()> {
    let mut stdout = BufWriter::new(stdout());
    writeln!(stdout, "{s}")?;
    stdout.flush()?;
    Ok(())
}

pub fn read(prompt: impl fmt::Display) -> io::Result<String> {
    let mut stdout = BufWriter::new(stdout());
    write!(stdout, "{prompt}")?;
    stdout.flush()?;

    let stdin = stdin();
    let mut input = String::new();
    stdin.lock().take(READ_LIMIT).read_line(&mut input)?;
    Ok(input.trim().into())
}

pub fn log(color: impl Into<Option<Color>>, body: impl fmt::Display) -> io::Result<()> {
    let bufwtr = BufferWriter::stderr(ColorChoice::Auto);
    let mut buffer = bufwtr.buffer();
    buffer.set_color(ColorSpec::new().set_fg(color.into()))?;
    writeln!(&mut buffer, "{body}")?;
    buffer.reset()?;
    bufwtr.print(&buffer)?;
    Ok(())
}

pub fn splash_text() -> io::Result<()> {
    let mut stderr = BufWriter::new(stderr());
    writeln!(
        stderr,
        "{} {}: {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_DESCRIPTION")
    )?;
    writeln!(stderr, r#"type "h" for help, "l" for license"#)?;
    stderr.flush()?;
    Ok(())
}

pub fn read_dur(prompt: impl fmt::Display) -> io::Result<Option<ReadDur>> {
    let prompt = prompt.to_string();
    let prompt_len = prompt.chars().count();
    let input = read(prompt)?;
    if input.is_empty() {
        return Ok(None);
    }

    let parsed = ReadDur::parse(&input);
    Ok(match parsed {
        Ok(dur) => Some(dur),
        Err(err) => {
            err.log(prompt_len)?;
            None
        }
    })
}
