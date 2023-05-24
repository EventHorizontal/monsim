
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
                debug_file.push_str(&format![
                    "debug_output.txt creation time: {}\n",
                    chrono::Local::now()
                ]);
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
#[cfg(feature = "debug")]
macro_rules! debug_to_file {
	($x: expr) => {
        #[cfg( feature = "debug" )]
		let debug_file_output_error = crate::debug::write_debug_to_file(format!["[{}:{}:{}]: {} = {:#?}", std::file!(), std::line!(), std::column!(), stringify!($x), $x.clone()]);
	};
}
