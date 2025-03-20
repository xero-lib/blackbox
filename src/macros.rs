// just a silly little guy
pub mod macros {
    #[macro_export]
    macro_rules! break_if {
        ($x:expr) => {
            if $x {
                break;
            }
        };
    }

    #[macro_export]
    macro_rules! debug_print {
        ($x:expr) => {
            #[cfg(debug_assertions)]
            println!("{:?}", $x)
        };
    }
}
