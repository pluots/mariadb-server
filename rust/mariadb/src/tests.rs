macro_rules! assert_layouts_eq {
    ($a:ty, $b:ty) => {
        assert_eq!(
            core::alloc::Layout::new::<$a>(),
            core::alloc::Layout::new::<$b>(),
        );
    };
}

pub(crate) use assert_layouts_eq;
