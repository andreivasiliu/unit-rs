use crate::nxt_unit;

/// Error code returned when Unit could not be initialized.
#[derive(Debug, Clone, Copy)]
pub struct UnitInitError;

/// Error code returned by the Unit library.
pub struct UnitError(pub(crate) i32);

impl UnitError {
    pub fn error() -> Self {
        Self(nxt_unit::NXT_UNIT_ERROR as i32)
    }
}

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

impl std::error::Error for UnitError {}

impl std::fmt::Debug for UnitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug_tuple = f.debug_tuple("UnitError");

        debug_tuple.field(&match self.0 as u32 {
            nxt_unit::NXT_UNIT_OK => "Successful",
            nxt_unit::NXT_UNIT_AGAIN => "Again",
            nxt_unit::NXT_UNIT_CANCELLED => "Cancelled",
            nxt_unit::NXT_UNIT_ERROR => "Error",
            _ => "Unknown",
        });
        debug_tuple.finish()
    }
}

impl std::fmt::Display for UnitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 as u32 {
            nxt_unit::NXT_UNIT_OK => "Unit error code: Successful.".fmt(f),
            nxt_unit::NXT_UNIT_AGAIN => "Unit error code: Not yet available, try again.".fmt(f),
            nxt_unit::NXT_UNIT_CANCELLED => "Unit error code: Cancelled.".fmt(f),
            nxt_unit::NXT_UNIT_ERROR => "Unit error code: General error.".fmt(f),
            _ => write!(f, "Unknown Unit error code: {}.", self.0),
        }
    }
}
