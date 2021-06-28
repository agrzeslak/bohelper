use std::process;

fn main() {
    bohelper::run().unwrap_or_else(|err| {
        eprintln!("Error encountered: {}", err);
        process::exit(1);
    });
}
