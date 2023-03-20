// sw: terminal stopwatch
// copyright (C) 2022-2023 Ula Shipman <ula.hello@mailbox.org>
// licensed under GPL-3.0-or-later

use termcolor::{Buffer, BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

use core::fmt;
use core::panic::Location;
use std::io::{self, stdin, BufRead, Read, Stdin, Write};

use crate::command::Command;

pub const INFO_CHANGE: Color = Color::Magenta;
pub const INFO_IDLE: Color = Color::Cyan;
pub const WARN: Color = Color::Yellow;
pub const ERROR: Color = Color::Red;

#[derive(Clone, Debug, PartialEq, Eq)]
enum IoKind {
    Out(ColorSpec),
    In,
}

pub struct Shell {
    stdout: BufferWriter,
    out_buf: Buffer,
    stdin: Stdin,
    read_limit: u64,
    last_op: Option<IoKind>,

    visual_cues: bool,

    splash_text_written: bool,

    ctor_loc: Location<'static>,
    finished: bool,
}

impl Shell {
    #[track_caller]
    pub fn new(choice: ColorChoice, read_limit: u64, visual_cues: bool) -> Self {
        let stdout = BufferWriter::stdout(choice);
        Self {
            out_buf: stdout.buffer(),
            stdout,
            stdin: stdin(),
            read_limit,
            last_op: None,
            visual_cues,
            splash_text_written: false,
            ctor_loc: *Location::caller(),
            finished: false,
        }
    }

    pub fn splash_text(&mut self) -> io::Result<()> {
        assert!(
            !self.splash_text_written,
            "splash text can only be written once"
        );
        self.splash_text_written = true;

        self.writeln(
            &ColorSpec::new(),
            format_args!(
                "{} {}: {}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
                env!("CARGO_PKG_DESCRIPTION")
            ),
        )?;
        self.writeln(
            &ColorSpec::new(),
            format_args!(r#"enter "h" for help, "l" for license."#),
        )?;
        self.writeln(
            &ColorSpec::new(),
            format_args!(
                "visual cues {}.",
                if self.visual_cues {
                    "enabled (unless --no-visual-cues)"
                } else {
                    "disabled"
                }
            ),
        )?;

        Ok(())
    }

    #[track_caller]
    pub fn create_cmd_buf(&mut self) -> CmdBuf<'_> {
        CmdBuf::new(self)
    }

    pub fn writeln(&mut self, color: &ColorSpec, fmt: fmt::Arguments) -> io::Result<()> {
        self.write(color, format_args!("{fmt}\n"))
    }

    pub fn write(&mut self, color: &ColorSpec, fmt: fmt::Arguments) -> io::Result<()> {
        let mut color = color.clone();
        color.set_reset(false);
        let this_op = IoKind::Out(color.clone());
        self.flush(Some(this_op))?;
        self.out_buf.set_color(&color)?;
        self.out_buf.write_fmt(fmt)?;
        Ok(())
    }

    pub fn read(&mut self) -> io::Result<String> {
        let this_op = IoKind::In;
        self.flush(Some(this_op))?;
        let mut input = String::new();
        self.stdin
            .lock()
            .take(self.read_limit)
            .read_line(&mut input)?;
        Ok(input.trim().to_string())
    }

    pub fn finish(&mut self) -> io::Result<()> {
        if !self.finished {
            self.finished = true;
            self.flush(None)?;
        }
        Ok(())
    }
}

impl Shell {
    fn flush(&mut self, anticipate: Option<IoKind>) -> io::Result<()> {
        if self.last_op != anticipate {
            match self.last_op {
                /* NOTE: flushing manually as workaround for
                 * https://github.com/BurntSushi/termcolor/issues/69 */
                Some(IoKind::Out(ref color)) => {
                    if !color.is_none() {
                        // don't reset color unless we have to
                        self.out_buf.reset()?;
                    }
                    self.stdout.print(&self.out_buf)?;
                    io::stdout().flush()?;
                    self.out_buf.clear();
                }
                Some(IoKind::In) | None => (),
            }
        }
        self.last_op = anticipate;
        Ok(())
    }
}

impl Drop for Shell {
    fn drop(&mut self) {
        _ = self.finish();
    }
}

pub struct CmdBuf<'shell> {
    shell: &'shell mut Shell,
    pad_above: bool,
}

impl CmdBuf<'_> {
    pub const fn visual_cues(&self) -> bool {
        self.shell.visual_cues
    }

    pub fn set_visual_cues(&mut self, new: bool) {
        self.shell.visual_cues = new;
    }

    pub fn read_cmd(
        &mut self,
        name: &str,
        is_running: bool,
    ) -> io::Result<Result<Command, String>> {
        let input = if self.shell.visual_cues {
            self.read(format_args!(
                "{name} {} ",
                if is_running { "*" } else { ";" }
            ))?
        } else {
            self.read(format_args!("{name}. "))?
        };
        match input.parse() {
            Ok(cmd) => Ok(Ok(cmd)),
            Err(()) => Ok(Err(input)),
        }
    }

    pub fn write_color(&mut self, color: &ColorSpec, fmt: fmt::Arguments) -> io::Result<()> {
        self.pad_above_once()?;
        self.shell.write(color, fmt)?;
        Ok(())
    }

    pub fn writeln_color(&mut self, color: &ColorSpec, fmt: fmt::Arguments) -> io::Result<()> {
        self.write_color(color, format_args!("{fmt}\n"))
    }

    pub fn write(&mut self, fmt: fmt::Arguments) -> io::Result<()> {
        self.write_color(&ColorSpec::new(), fmt)
    }

    pub fn writeln(&mut self, fmt: fmt::Arguments) -> io::Result<()> {
        self.writeln_color(&ColorSpec::new(), fmt)
    }

    pub fn info_change(&mut self, fmt: fmt::Arguments) -> io::Result<()> {
        self.writeln_color(ColorSpec::new().set_fg(Some(INFO_CHANGE)), fmt)
    }

    pub fn info_idle(&mut self, fmt: fmt::Arguments) -> io::Result<()> {
        self.writeln_color(ColorSpec::new().set_fg(Some(INFO_IDLE)), fmt)
    }

    pub fn warn(&mut self, fmt: fmt::Arguments) -> io::Result<()> {
        self.writeln_color(ColorSpec::new().set_fg(Some(WARN)), fmt)
    }

    pub fn error(&mut self, fmt: fmt::Arguments) -> io::Result<()> {
        self.writeln_color(
            ColorSpec::new().set_fg(Some(ERROR)),
            format_args!("error: {fmt}"),
        )
    }

    pub fn read(&mut self, prompt: fmt::Arguments) -> io::Result<String> {
        self.write(prompt)?;
        self.shell.read()
    }
}

impl<'shell> CmdBuf<'shell> {
    fn new(shell: &'shell mut Shell) -> Self {
        Self {
            pad_above: shell.splash_text_written,
            shell,
        }
    }

    fn pad_above_once(&mut self) -> io::Result<()> {
        if self.pad_above {
            self.vertical_pad()?;
            self.pad_above = false;
        }
        Ok(())
    }

    fn vertical_pad(&mut self) -> io::Result<()> {
        self.shell.writeln(&ColorSpec::new(), format_args!(""))?;
        Ok(())
    }
}
