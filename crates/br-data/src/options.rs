use gumdrop::Options as Opts;

#[derive(Debug, Opts)]
pub struct Options {
    #[options(help = "print current version")]
    pub version: bool,

    #[options(help = "print help message")]
    pub help: bool,

    #[options(no_short, help = "start without running branshrc.br")]
    pub norc: bool,

    #[options(no_long, help = "execute command and exit")]
    pub command: Option<String>
}

impl Options {
    pub fn parse() -> Self {
        Self::parse_args_default_or_exit()
    }
}
