pub mod app;
#[cfg(feature = "debug")]
pub mod debug;
pub mod sim;
mod test;

#[macro_export]
macro_rules! not {
    ($x: expr) => {
        !$x
    };
}
