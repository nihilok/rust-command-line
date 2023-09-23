use core::fmt;
use std::{io, process::Command, string::FromUtf8Error};

#[derive(Debug)]
pub enum Errors {
    FromUtf8(FromUtf8Error),
    IO(io::Error),
    Custom(String),
    STDERR(String),
}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Errors::FromUtf8(ref err) => err.fmt(f),
            Errors::IO(ref err) => err.fmt(f),
            Errors::Custom(ref err) => write!(f, "ERROR: {})", err),
            Errors::STDERR(ref err) => write!(f, "ERROR: {})", err),
        }
    }
}

impl From<FromUtf8Error> for Errors {
    fn from(err: FromUtf8Error) -> Errors {
        Errors::FromUtf8(err)
    }
}

impl From<io::Error> for Errors {
    fn from(err: io::Error) -> Errors {
        Errors::IO(err)
    }
}

impl From<String> for Errors {
    fn from(err: String) -> Errors {
        Errors::STDERR(err)
    }
}

impl std::error::Error for Errors {}

pub struct CommandOutput(String, u8);

impl CommandOutput {
    pub fn output(&self) -> &str {
        &self.0
    }
    pub fn exit_code(&self) -> &u8 {
        &self.1
    }
}

pub fn execute_command(command_line: &str) -> Result<CommandOutput, Errors> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("{command_line}"))
        .output();
    return match output {
        Ok(output) => {
            if !output.status.success() {
                return Ok(CommandOutput(String::from_utf8(output.stderr)?, 1));
            }
            Ok(CommandOutput(String::from_utf8(output.stdout)?, 0))
        }
        Err(err) => Err(Errors::IO(err)),
    };
}

pub fn sh(command_line: &str) -> Result<String, Errors> {
    let output = execute_command(command_line);
    return match output {
        Ok(output) => {
            let content = output.output().to_string();
            if output.1 == 1 {
                return Err(Errors::STDERR(content))
            }
            Ok(content)
        }
        Err(err) => Err(err),
    };
}

pub fn execute_command_silent(command_line: &str, log_stderr: bool) -> bool {
    let output = execute_command(command_line);
    return match output {
        Ok(output) => {
            if output.exit_code() > &0 {
                if log_stderr {
                    eprintln!("{}", output.output());
                }
                return false;
            }
            true
        }
        Err(err) => {
            eprintln!("{}", err.to_string());
            false
        }
    };
}

pub fn command_exists(command: &str) -> bool {
    execute_command_silent(&format!("command -v {command}"), false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = execute_command_silent("echo hello", false);
        assert_eq!(result, true);
    }

    #[test]
    fn it_fails() {
        let result = execute_command_silent("invalid-command-xxxxxxxxxxxx", false);
        // stderr should be logged to the console
        assert_eq!(result, false);
    }

    #[test]
    fn test_command_exists() {
        let result = command_exists("echo");
        assert_eq!(result, true);
    }

    #[test]
    fn test_command_not_exists() {
        let result = command_exists("invalid-command-xxxxxxxxxxxx");
        assert_eq!(result, false);
    }

    #[test]
    fn test_command_with_errors() {
        let result = sh("mv file-does-not-exist.txt /location/does/not/exist");
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn test_command_not_with_errors() {
        let result = sh("echo hello");
        assert_eq!(result.is_err(), false);
    }
}
