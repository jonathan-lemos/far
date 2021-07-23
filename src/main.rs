mod dir_iter;
mod far;
mod file;
mod input;
mod iter;
mod replace;

use input::{ArgsError, parse_args};
use far::find_and_replace;

fn handle_argserror(e: ArgsError) {
    eprintln!("Failed to parse command-line arguments: {}", e)
}

fn main() {
    let args = match parse_args() {
        Ok(v) => v,
        Err(e) => return handle_argserror(e)
    };

    find_and_replace(args.paths.iter(), &args.pattern, &args.replacement)
}
