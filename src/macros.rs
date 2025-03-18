// just a silly little guy
macro_rules! break_if {
    ($x:expr) => {
        if $x {
            break;
        }
    };
}

pub(crate) use break_if;
