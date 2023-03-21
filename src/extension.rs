pub trait OptionExt<T> {
    fn when<F>(cond: bool, fa: F) -> Option<T>
    where
        F: Fn() -> T;
}

impl<T> OptionExt<T> for Option<T> {
    fn when<F>(cond: bool, fa: F) -> Option<T>
    where
        F: Fn() -> T,
    {
        if cond {
            Some(fa())
        } else {
            None
        }
    }
}
