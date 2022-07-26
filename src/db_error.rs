use std::{fmt::Display, error::Error};

#[derive(Debug)]
pub enum DbError {
    DidntUpdateGuest{ guestid: i32, inviteid: i32}
}

impl Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbError::DidntUpdateGuest { guestid, inviteid } => {
                f.write_fmt(format_args!("Didn't update guest. guestid={}, inviteid={}", guestid, inviteid))
            }
        }
    }
}
impl Error for DbError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}
