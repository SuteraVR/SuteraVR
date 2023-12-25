#[cfg(test)]
mod tests {
    use suteravr_lib::Foo;

    /// whether clocking-server can load the suteravr-lib crate or not
    #[test]
    fn test_loading_suteravrlib() {
        let _foo: Foo = Foo::default();
    }
}
