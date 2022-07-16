/// Error code returned when Unit could not be initialized.
#[derive(Debug, Clone, Copy)]
pub struct UnitInitError;

/// Error code returned by the Unit library.
pub struct UnitError(pub(crate) i32);

/// Result type returned from methods that have a [`UnitError`](UnitError)
/// error.
pub type UnitResult<T> = Result<T, UnitError>;

pub(crate) trait IntoUnitResult {
    fn into_unit_result(self) -> UnitResult<()>;
}

impl IntoUnitResult for i32 {
    fn into_unit_result(self) -> UnitResult<()> {
        if self == 0 {
            Ok(())
        } else {
            Err(UnitError(self))
        }
    }
}
