/// Expands to the path of the function within which it was invoked e.g.
/// `"crate::module::function::"`
#[macro_export]
macro_rules! function {
    () => {{
        // This works by defining a function within the call site and using
        // (abusing) `std::any::type_name` to retrieve it's full path.
        //
        // Note: it is possible that this may break as the docs for
        // `std::any::type_name`` state:
        //
        // > This is intended for diagnostic use. The exact contents and
        // > format of the string are not specified, other than being a
        // > best-effort description of the type
        //
        // and
        // > ... output may change between versions of the compiler.
        //
        // I'm ok with this because `q` is a low-risk crate (e.g. it's a "cool"
        // debugging tool but if it breaks it won't be the end of the world),
        // it works for now and I can't (currently) think of a better way to
        // do this without more "heavy-weight" methods e.g. parsing backtraces
        // or proc macros.
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f); // `"crate::module::function::f"`
        &name[..name.len() - 3] // `"crate::module::function::"`
    }};
}

#[macro_export]
macro_rules! q {
    () => {
        // FIXME:
        $crate::q_log(file!(), function!(), line!());
    };
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_function() {
        assert_eq!(function!(), "q::macros::tests::test_function");

        struct Foo {
            bar: String,
        }

        impl Foo {
            pub fn new() -> Self {
                Foo {
                    bar: function!().to_string(),
                }
            }
        }

        assert_eq!(Foo::new().bar, "q::macros::tests::test_function::Foo::new");
    }
}
