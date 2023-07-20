#[macro_export]
macro_rules! always {
    ($e:expr) => {
        if $e {
            true
        } else {
            assert!($e);
            false
        }
    };
}

#[macro_export]
macro_rules! never {
    ($e:expr) => {
        if $e {
            assert!(!($e));
            true
        } else {
            false
        }
    };
}
