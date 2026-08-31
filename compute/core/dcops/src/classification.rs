use data_boundary_kernel::{Classified, DataClass, OperationalDataClass};

pub(crate) fn public<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Public)
}

pub(crate) fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

pub(crate) fn audit<T>(value: T) -> Classified<T> {
    Classified::new(value, OperationalDataClass::Audit)
}
