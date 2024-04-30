pub trait Component: 'static + Clone {}
impl<T: 'static + Clone> Component for T {}
