// 可能可以使用编译时替换，使用过程宏，优化运行时开销，但需要额外crates
#[macro_export]
macro_rules! func_return_string {
    ($name:ident) => {
        fn $name() -> String {
            stringify!($name).replace('_', "-")
        }
    };
}

#[macro_export]
macro_rules! func_return_string_ {
    ($name:ident) => {
        fn $name() -> String {
            stringify!($name).to_string()
        }
    };
}
