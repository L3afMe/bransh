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

pub enum TokenizationError {
    UnterminateQuote(usize, char),
    InvalidEscape(usize, char),
    UnexpectedValue(usize, String, String),
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

    let mut args: Vec<String> = Vec::new();
    let mut arg = String::new();

    let mut last_buf_char = char::default();
    let mut last_escaped = false;

    for (idx, buf_char) in buffer.chars().enumerate() {
        if last_buf_char == '\\' && !last_escaped {
            match buf_char {
                '#' | ';' | '|' | '&' | '>' | '\\' | '"' | '\'' => arg.push(buf_char),
                'n' => arg.push('\n'),
                'r' => arg.push('\r'),
                't' => arg.push('\t'),
                _ => return Err(TokenizationError::InvalidEscape(idx + 1 + left_trim_size, buf_char)),
            }

            last_escaped = true;
            last_buf_char = buf_char;
            continue;
        } else if buf_char == '\'' || buf_char == '"' {
            if in_quote == char::default() {
                in_quote = buf_char;
                quote_pos = idx + 1 + left_trim_size;
            } else if in_quote == buf_char {
                in_quote = char::default();
            } else {
                arg.push(buf_char)
            }
        } else if buf_char == ' ' && in_quote == char::default() {
            if !arg.is_empty() {
                args.push(arg);
                arg = String::new();
            }
        } else if (buf_char == ';' || buf_char == '|') && in_quote == char::default() {
            if !arg.is_empty() {
                args.push(arg);
                arg = String::new();
            }

            args.push(buf_char.into());

            match args_to_command(args) {
                Some(command) => commands.push(command),
                None => {
                    return Err(TokenizationError::UnexpectedValue(
                        buffer.len() + left_trim_size,
                        String::from("command"),
                        String::from("null"),
                    ));
                },
            }

            args = Vec::new();
        } else if buf_char != '\\' {
            arg.push(buf_char)
        }

        last_escaped = false;
        last_buf_char = buf_char;
    }

    if !arg.is_empty() {
        args.push(arg);
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
        return Err(TokenizationError::InvalidEscape(
            buffer.len() + left_trim_size,
            char::default(),
        ));
    }

    Ok(commands)
}

fn args_to_command(mut args: Vec<String>) -> Option<Command> {
    if args.is_empty() {
        return None;
    }

    if args.contains(&String::from("Testlol")) {
        println!("{:?}", args);
    }

    let cmd_name = args[0].clone();
    args.remove(0);
    let output_type;
    let background;

    let mut should_pop = false;
    if let Some(last) = args.last() {
        should_pop = OutputType::is_valid(last);

        output_type = OutputType::from(last);
    } else {
        output_type = OutputType::default()
    }

    if should_pop {
        args.pop();
    }

    if let Some(last) = args.last() {
        background = last == "&";
    } else {
        background = false;
    }

    let command = Command::new(cmd_name, args, background, output_type);

    Some(command)
}
