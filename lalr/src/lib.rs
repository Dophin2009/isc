#[macro_use]
mod comparator {
    macro_rules! item {
        ($x: item) => {
            $x
        };
    }

    macro_rules! comparators {
        ($t: ident($($p: tt)+), ($($s: ident),+), ($($f: ident),+)) => {
            item!(impl<$($p)+> PartialEq for $t<$($p)+> where $($s: PartialEq),+ {
                fn eq(&self, other: &Self) -> bool {
                    ($(self.$f),+) == ($(other.$f),+)
                }
            });
            item!(impl<$($p)+> Eq for $t<$($p)+> where $($s: Eq),+ {
            });
            item!(impl<$($p)+> PartialOrd for $t<$($p)+> where $($s: PartialOrd),+ {
                fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                    ($(self.$f),+).partial_cmp(&($(other.$f),+))
                }
            });
            item!(impl<$($p)+> Ord for $t<$($p)+> where $($s: Ord),+ {
                fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                    ($(self.$f),+).cmp(&($(other.$f),+))
                }
            });
        };
    }
}

mod error;
mod grammar;
mod lalr;

pub use error::Error;
pub use grammar::*;
