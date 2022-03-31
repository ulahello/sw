use std::io::{self, Write};

use sw::stopwatch::Stopwatch;
use sw::Error;

fn main() {
    match control_stopwatch(Stopwatch::new()) {
        Ok(()) => (),
        Err(err) => {
            eprintln!("fatal: {}", err);
        }
    }
}

enum Command {
    Quit,
    Help,
    Toggle,
    Reset,
}

impl Command {
    pub fn from_stdin() -> Result<Result<Self, String>, Error> {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.to_lowercase().trim() {
            "q" => Ok(Ok(Self::Quit)),
            "h" => Ok(Ok(Self::Help)),
            "s" => Ok(Ok(Self::Toggle)),
            "r" => Ok(Ok(Self::Reset)),
            other => Ok(Err(other.into())),
        }
    }
}

fn control_stopwatch(mut stopwatch: Stopwatch) -> Result<(), Error> {
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    // splash text
    writeln!(
        stderr,
        "{} {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    )?;

    writeln!(stderr, "type \"h\" for help")?;
    writeln!(stderr)?;

    loop {
        // prompt for command
        write!(stdout, "> ")?;
        stdout.flush()?;

        // respond to command
        match Command::from_stdin()? {
            Ok(command) => match command {
                Command::Quit => return Ok(()),

                Command::Help => {
                    writeln!(stdout, "terminal stopwatch")?;
                    writeln!(stdout)?;
                    writeln!(stdout, "| command | description        |")?;
                    writeln!(stdout, "| ---     | ---                |")?;
                    writeln!(stdout, "| q       | quit               |")?;
                    writeln!(stdout, "| h       | print this message |")?;
                    writeln!(stdout, "| s       | toggle stopwatch   |")?;
                    writeln!(stdout, "| r       | reset stopwatch    |")?;
                    writeln!(stdout, "| <enter> | display stopwatch  |")?;
                }

                Command::Toggle => {
                    stopwatch.toggle();
                    if stopwatch.is_running() {
                        writeln!(stderr, "started stopwatch")?;
                    } else {
                        writeln!(stderr, "stopped stopwatch")?;
                    }
                }

                Command::Reset => {
                    stopwatch.reset();
                    writeln!(stderr, "reset stopwatch")?;
                }
            },
            Err(input) => {
                if input.is_empty() {
                    writeln!(stdout, "{}", stopwatch)?;
                } else {
                    writeln!(stderr, "unrecognized command `{}`", input)?;
                }
            }
        }
    }
}
