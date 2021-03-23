#[derive(Debug, Clone, PartialEq)]
pub enum OutputType {
    Depend,
    DependNot,
    Pipe,
    Ignore,
}

impl Default for OutputType {
    fn default() -> Self {
        Self::Ignore
    }
}

#[allow(dead_code)]
impl OutputType {
    fn is_valid(chr: &str) -> bool {
        chr == "&&" || chr == "||" || chr == "|" || chr == ";"
    }
}

impl From<&String> for OutputType {
    fn from(chr: &String) -> Self {
        match chr.as_ref() {
            "&&" => OutputType::Depend,
            "||" => OutputType::DependNot,
            "|" => OutputType::Pipe,
            _ => OutputType::Ignore,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Command {
    pub command:     String,
    pub args:        Vec<String>,
    pub background:  bool,
    pub output_type: OutputType,
}

impl Command {
    fn new(command: String, args: Vec<String>, background: bool, output_type: OutputType) -> Self {
        Self {
            command,
            args,
            background,
            output_type,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenizationError {
    UnterminateQuote(usize, char),
    InvalidEscape(usize, char),
    UnexpectedValue(usize, String, String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArgType {
    Quoted(String),
    Raw(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenizeType {
    Arg(Vec<ArgType>),
    End(OutputType),
}

pub type TokenizedCommands = Vec<Command>;

// TODO: Actual and efficient tokenizing when I'm not dumb
pub fn tokenize_command(command_buffer: String) -> Result<TokenizedCommands, TokenizationError> {
    let mut buffer = command_buffer.trim_start();
    let mut commands: TokenizedCommands = TokenizedCommands::new();

    if command_buffer.is_empty() {
        return Ok(commands);
    }

    // Store trim size so syntax positions are correct
    let left_trim_size = command_buffer.len() - buffer.len();
    buffer = buffer.trim_end();

    let mut in_quote = char::default();
    let mut quote_pos = 0;

    let mut args: Vec<TokenizeType> = Vec::new();
    let mut arg: Vec<ArgType> = Vec::new();
    let mut arg_part = String::new();

    let mut last_buf_char = char::default();
    let mut last_escaped = false;

    for (idx, buf_char) in buffer.chars().enumerate() {
        if last_buf_char == '\\' && !last_escaped {
            match buf_char {
                '#' | ';' | '|' | '&' | '>' | '\\' | '"' | '\'' => {
                    if !arg_part.is_empty() && in_quote == char::default() {
                        arg.push(ArgType::Raw(arg_part));
                        arg_part = buf_char.into();
                    }
                },
                'n' => arg_part.push('\n'),
                'r' => arg_part.push('\r'),
                't' => arg_part.push('\t'),
                _ => return Err(TokenizationError::InvalidEscape(idx + 1 + left_trim_size, buf_char)),
            }

            last_escaped = true;
            last_buf_char = buf_char;
            continue;
        } else if buf_char == '\'' || buf_char == '"' {
            if in_quote == char::default() {
                if !arg_part.is_empty() {
                    arg.push(ArgType::Raw(arg_part));
                    arg_part = String::new();
                }

                in_quote = buf_char;
                quote_pos = idx + 1 + left_trim_size;
            } else if in_quote == buf_char {
                if !arg_part.is_empty() {
                    arg.push(ArgType::Quoted(arg_part));
                    arg_part = String::new();
                }

                in_quote = char::default();
            } else {
                arg_part.push(buf_char)
            }
        } else if buf_char == ' ' && in_quote == char::default() {
            if !arg_part.is_empty() {
                arg.push(ArgType::Raw(arg_part));
                arg_part = String::new();
            }

            if !arg.is_empty() {
                args.push(TokenizeType::Arg(arg));
                arg = Vec::new();
            }
        } else if (buf_char == ';' || buf_char == '|') && in_quote == char::default() {
            if !arg_part.is_empty() {
                arg.push(ArgType::Raw(arg_part));
                args.push(TokenizeType::Arg(arg));
                arg = Vec::new();
                arg_part = String::new();
            }

            args.push(TokenizeType::End(OutputType::from(&buf_char.to_string())));

            match args_to_command(args) {
                Some(command) => commands.push(command),
                None => {
                    return Err(TokenizationError::UnexpectedValue(
                        idx + left_trim_size,
                        String::from("command"),
                        String::from("null"),
                    ));
                },
            }

            args = Vec::new();
        } else if buf_char != '\\' {
            arg_part.push(buf_char)
        }

        last_escaped = false;
        last_buf_char = buf_char;
    }

    if !arg_part.is_empty() {
        arg.push(ArgType::Raw(arg_part));
    }

    if !arg.is_empty() {
        args.push(TokenizeType::Arg(arg));
    }

    match args_to_command(args) {
        Some(command) => commands.push(command),
        None => {
            if let Some(cmd) = commands.last() {
                if cmd.output_type != OutputType::Ignore {
                    return Err(TokenizationError::UnexpectedValue(
                        buffer.len() + left_trim_size,
                        String::from("command"),
                        String::from("null"),
                    ));
                }
            }
        },
    }

    if in_quote != char::default() {
        return Err(TokenizationError::UnterminateQuote(quote_pos, in_quote));
    }

    if last_buf_char == '\\' && !last_escaped {
        return Err(TokenizationError::InvalidEscape(buffer.len() + left_trim_size, char::default()));
    }

    Ok(commands)
}

fn args_to_command(mut args: Vec<TokenizeType>) -> Option<Command> {
    if args.is_empty() {
        return None;
    }

    let mut cmd_name = String::new();
    if let Some(TokenizeType::Arg(tokens)) = args.first() {
        for val in tokens {
            match val {
                ArgType::Raw(val) => cmd_name += val,
                ArgType::Quoted(val) => cmd_name += val,
            }
        }
    }

    if cmd_name.is_empty() {
        return None;
    }

    // Remove cmd name from args
    args.remove(0);
    let mut output_type = OutputType::Ignore;
    let mut background = false;

    if let Some(TokenizeType::End(out)) = args.last() {
        output_type = out.clone();
        args.pop();
    }

    if let Some(TokenizeType::Arg(val)) = args.last() {
        if val.len() == 1 {
            if let ArgType::Raw(raw_val) = &val[0] {
                if raw_val == "&" {
                    background = true;
                    args.pop();
                }
            }
        }
    }

    let mut output_args = Vec::new();
    for arg in args {
        match arg {
            TokenizeType::Arg(val) => {
                let mut temp_arg = String::new();
                for arg_part in val {
                    match arg_part {
                        ArgType::Raw(val) => temp_arg += &val,
                        ArgType::Quoted(val) => temp_arg += &val,
                    }
                }

                if !temp_arg.is_empty() {
                    output_args.push(temp_arg);
                }
            },
            _ => break,
        }
    }

    let command = Command::new(cmd_name, output_args, background, output_type);

    Some(command)
}
