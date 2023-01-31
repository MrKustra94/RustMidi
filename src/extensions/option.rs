pub trait OptionExt<T> {
    fn when<F>(cond: bool, fa: F) -> Option<T>
    where
        F: Fn() -> T;

    fn to_result<E, F>(self, on_empty: F) -> Result<T, E>
    where
        F: Fn() -> E;
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

    fn to_result<E, F>(self, on_empty: F) -> Result<T, E>
    where
        F: Fn() -> E,
    {
        match self {
            Some(v) => Ok(v),
            None => Err(on_empty()),
        }
    }
}
