macro_rules! hash_set {
    () => {
        std::collections::HashSet::new()
    };
    ( $( $x:expr ),* ) => {{
        let mut set = std::collections::HashSet::new();
        $(
            set.insert($x);
        )*
        set
    }};
}
