pub trait AsStr: AsRef<str> + std::fmt::Debug {}

impl<T> AsStr for T where T: AsRef<str> + std::fmt::Debug {}
