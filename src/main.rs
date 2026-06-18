use std::process;

use mpct::error::AppError;

fn main() {
    if let Err(err) = mpct::run() {
        if let Some(app_error) = err.downcast_ref::<AppError>()
            && let Some(code) = app_error.silent_exit_code()
        {
            process::exit(code);
        }

        eprintln!("{err}");
        process::exit(1);
    }
}
