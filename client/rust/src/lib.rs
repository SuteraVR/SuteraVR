use godot::prelude::*;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

#[cfg(test)]
mod test {
    use suteravr_lib::Foo;

    /// whether this rust crate can load the suteravr-lib crate or not.
    #[test]
    fn test_loading_suteravrlib() {
        let _foo: Foo = Foo::default();
    }
}
