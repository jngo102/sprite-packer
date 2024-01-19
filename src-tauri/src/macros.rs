#[macro_export]
macro_rules! log_panic {
    ($($arg:tt)+) => ({
        error!($($arg)+);
        panic!($($arg)+);
    })
}