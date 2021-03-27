#[macro_use]
extern crate lazy_static;

mod builtins;

use std::{ffi::OsStr, path::PathBuf, str::FromStr};

use br_data::{command::TabCompletionType, context::Context};

pub fn load_builtins(ctx: &mut Context) {
    ctx.builtins = vec![
        builtins::alias::CMD.clone(),
        builtins::cd::CMD,
        builtins::exit::CMD,
        builtins::get::CMD,
        builtins::set::CMD,
    ];
}

pub fn get_tab_completion(cmd: String, args: Vec<String>, ctx: &mut Context) -> Vec<String> {
    for builtin in ctx.builtins.clone() {
        if builtin.name == cmd {
            return get_comp(builtin.tab_completion, args, ctx);
        }
    }

    let arg = args.into_iter().last().unwrap_or_default();
    get_file(FileType::Both, arg)
}

#[derive(Debug, PartialEq)]
enum FileType {
    File,
    Directory,
    Both,
}

impl From<&TabCompletionType> for FileType {
    fn from(tab: &TabCompletionType) -> Self {
        match tab {
            TabCompletionType::Directory(_) => FileType::Directory,
            TabCompletionType::File(_) => FileType::File,
            _ => FileType::Both,
        }
    }
}

fn get_comp(tc_type: TabCompletionType, args: Vec<String>, ctx: &mut Context) -> Vec<String> {
    match tc_type.clone() {
        TabCompletionType::None => Vec::new(),
        TabCompletionType::File(subargs)
        | TabCompletionType::Directory(subargs)
        | TabCompletionType::FileOrDirectory(subargs) => {
            ctx.cli.completion.delim = String::from("/");
            let mut itr = args.into_iter().peekable();
            let arg = itr.next().unwrap();

            // If is last arg
            if itr.peek().is_none() {
                return get_file(FileType::from(&tc_type), arg);
            } else {
                for sub in subargs {
                    if sub.arg == arg {
                        return get_comp(sub.subargs, itr.collect(), ctx);
                    }
                }
            }

            Vec::new()
        },
        TabCompletionType::Dynamic(fun) => (fun)(args, ctx),
        TabCompletionType::Static(comp) => {
            if args.is_empty() {
                return comp.into_iter().map(|arg| arg.arg).collect();
            }

            let mut itr = args.into_iter().peekable();
            let arg = itr.next().unwrap();

            // If is last arg
            if itr.peek().is_none() {
                return comp
                    .into_iter()
                    .map(|comp| comp.arg)
                    .filter(|comp| comp.starts_with(&arg))
                    .collect();
            }

            for comp_arg in comp {
                if comp_arg.arg == arg {
                    return get_comp(comp_arg.subargs, itr.collect(), ctx);
                }
            }
            Vec::new()
        },
    }
}

fn get_file(file_type: FileType, mut arg: String) -> Vec<String> {
    let mut trim_home_dir = None;
    if arg.starts_with('~') { 
        if arg.starts_with("~/") || arg == "~" {
            let home_dir = match home::home_dir() {
                Some(home_dir) => home_dir.to_string_lossy().to_string(),
                None => String::from("~")
            };

            trim_home_dir = Some(home_dir.clone());
            arg = home_dir + arg.strip_prefix("~").unwrap()
        } else {
            // TODO: Get other user home dir
        }
    }

    let mut print_cur_path = true;
    let (cur_path, cur_entry) = if let Some(path) = arg.rsplit_once("/") {
        path
    } else {
        print_cur_path = false;
        (".", arg.as_str())
    };

    let path = PathBuf::from_str(&cur_path).unwrap();
    let children_wrapped = path.read_dir();
    if !path.exists() || children_wrapped.is_err() {
        return Vec::new();
    }

    let mut output: Vec<String> = children_wrapped
        .unwrap()
        .filter(|child| child.is_ok())
        .map(|child| child
            .unwrap()
            .path())
        .filter(|child| 
            (file_type == FileType::Directory && child.is_dir()) ||
            (file_type == FileType::File && child.is_file())     ||
             file_type == FileType::Both)
        .map(|child| child
            .file_name()
            .unwrap_or_else(|| OsStr::new(""))
            .to_string_lossy()
            .to_string())
        .filter(|child| child
            .starts_with(cur_entry))
        .map(|child| {
            let mut path = cur_path.to_string();
            if let Some(home) = trim_home_dir.clone() {
                path = path.replace(&home, "~");
            }

            if print_cur_path {
                format!("{}/{}", path, child)
            } else {
                child
            }
        })
        .collect();

    output.sort();

    output
}

