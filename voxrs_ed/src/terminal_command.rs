use std::ffi::OsString;

#[derive(Debug)]
pub enum TerminalCommand {
    Save(OsString),
    Load(OsString),
    ChangeMaterial(u8),
}

pub enum ParseError {
    InvalidCommand,
    UnknownCommand(String),
}

impl std::str::FromStr for TerminalCommand {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let args = s.split_whitespace().map(|s| s.to_lowercase()).collect::<Vec<_>>();
        if args.is_empty() {
            return Err(ParseError::InvalidCommand);
        }

        match args[0].as_str() {
            "save" => {
                if args.len() != 2 {
                    Err(ParseError::InvalidCommand)
                } else {
                    Ok(TerminalCommand::Save(args[1].clone().into()))
                }
            }
            "load" => {
                if args.len() != 2 {
                    Err(ParseError::InvalidCommand)
                } else {
                    Ok(TerminalCommand::Load(args[1].clone().into()))
                }
            }
            "change_mat" => {
                if args.len() != 2 {
                    Err(ParseError::InvalidCommand)
                } else {
                    let mat_id = args[1].parse::<u8>();
                    if let Ok(mat_id) = mat_id {
                        Ok(TerminalCommand::ChangeMaterial(mat_id))
                    } else {
                        Err(ParseError::InvalidCommand)
                    }
                }
            }
            _ => Err(ParseError::UnknownCommand(args[0].to_string())),
        }
    }
}
