use far::find_and_replace;
use input::{ArgsError, FarMode, parse_cmdline};

mod far;
mod file;
mod input;
mod iter;
mod replace;
mod testdir;

fn handle_argserror(e: ArgsError) {
    eprintln!("Failed to parse command-line arguments: {}", e)
}

fn main() {
    let args = match parse_cmdline() {
        Ok(v) => v,
        Err(e) => return handle_argserror(e)
    };

    find_and_replace(args.paths, &args.pattern, &args.replacement, args.mode)
}
