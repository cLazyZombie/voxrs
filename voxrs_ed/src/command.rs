use std::ffi::OsString;

#[derive(Debug)]
pub enum Command {
    Save(OsString),
    Load(OsString),
    ChangeMaterial(u8),
}

pub enum CommandParseError {
    InvalidCommand,
    UnknownCommand(String),
}

impl std::str::FromStr for Command {
    type Err = CommandParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let args = s.split_whitespace().map(|s| s.to_lowercase()).collect::<Vec<_>>();
        if args.is_empty() {
            return Err(CommandParseError::InvalidCommand);
        }

        match args[0].as_str() {
            "save" => {
                if args.len() != 2 {
                    Err(CommandParseError::InvalidCommand)
                } else {
                    Ok(Command::Save(args[1].clone().into()))
                }
            }
            "load" => {
                if args.len() != 2 {
                    Err(CommandParseError::InvalidCommand)
                } else {
                    Ok(Command::Load(args[1].clone().into()))
                }
            }
            "change_mat" => {
                if args.len() != 2 {
                    Err(CommandParseError::InvalidCommand)
                } else {
                    let mat_id = args[1].parse::<u8>();
                    if let Ok(mat_id) = mat_id {
                        Ok(Command::ChangeMaterial(mat_id))
                    } else {
                        Err(CommandParseError::InvalidCommand)
                    }
                }
            }
            _ => Err(CommandParseError::UnknownCommand(args[0].to_string())),
        }
    }
}
