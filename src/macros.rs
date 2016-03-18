

#[doc(hidden)]
#[macro_export]
macro_rules! create_ids {
    ($t:ty, ) => ();
    ($t:ty, prev($prev:ident), $name:ident) => (
        #[allow(non_upper_case_globals)]
        pub const $name: $t = $prev + 1;
    );
    ($t:ty, prev($prev:ident), $name:ident, $($n:ident),+) => (
        #[allow(non_upper_case_globals)]
        pub const $name: $t = $prev + 1;
        create_ids!($t, prev($name), $($n),+);
    );
    ($t:ty, $name:ident, $($n:ident),+) => (
        #[allow(non_upper_case_globals)]
        pub const $name: $t = 0;
        create_ids!($t, prev($name), $($n),+);
    );
    ($t:ty, $name:ident) => (
        #[allow(non_upper_case_globals)]
        pub const $name: $t = 0;
    );
}
