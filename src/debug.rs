pub fn write_debug_to_file(message: String) -> std::io::Result<()> {
    use std::fs::{read_to_string, write};
    let debug_file_result = read_to_string("debug_output.txt");
    match debug_file_result {
        Ok(mut debug_file) => {
            debug_file.push_str(&message);
            debug_file.push_str("\n");
            write("debug_output.txt", debug_file)?;
        }
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => {
                let mut debug_file = String::new();
                debug_file.push_str(&format!["debug_output.txt creation time: {}\n", chrono::Local::now()]);
                debug_file.push_str(&message);
                debug_file.push_str("\n");
                write("debug_output.txt", debug_file)?;
            }
            _ => return Err(e),
        },
    }
    Ok(())
}

#[macro_export]
/// Only produces the file if the `debug` feature flag is enabled.
macro_rules! debug_to_file {
    ($x: expr) => {
        #[cfg(feature = "debug")]
        let debug_file_output_error = crate::debug::write_debug_to_file(format![
            "{location}: {name} = {value:#?}",
            location: debug_location!(),
            name = stringify!($x),
            value = $x.clone()
        ]);
    };
}

#[macro_export]
#[cfg(feature = "debug")]
macro_rules! source_code_location {
    () => {
        const_format::formatcp![
            "[ {file}:{line}:{col} ]",
            file = std::file!(),
            line = std::line!(),
            col = std::column!(),
        ]
    };
}

use monsim_utils::{Nothing, NOTHING};
use std::error::Error;
#[cfg(feature="debug")]
pub fn remove_debug_log_file() -> Result<Nothing, Box<dyn Error>> {

    #[cfg(feature = "debug")]
    if let Err(e) = std::fs::remove_file("debug_output.txt") {
        if std::io::ErrorKind::NotFound != e.kind() {
            Err(Box::new(e) as Box<dyn Error>)
        } else {
            Ok(NOTHING)
        }
    } else {
        Ok(NOTHING)
    }
}