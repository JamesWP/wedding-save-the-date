use serde::{Deserialize, Serialize};
use tokio_postgres::Row;

use serde_repr::*;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[repr(u8)]
pub enum DbAttendance {
    NotResponded = 0,
    Attending = 1,
    NotAttending = 2,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DbInvite {
    pub uid: i32,
    pub email: String,
    pub names: String,
    pub guests: Vec<DbGuest>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DbGuest {
    pub guestid: i32,
    pub name: String,
    pub attending: DbAttendance,
    pub diet_options: Vec<String>,
}

impl From<Row> for DbInvite {
    fn from(value: Row) -> Self {
        let uid: i32 = value.get("uid");
        let email: String = value.get("email");
        let names: String = value.get("names");

        DbInvite {
            uid,
            email,
            names,
            guests: Default::default(),
        }
    }
}

impl From<Row> for DbGuest {
    fn from(value: Row) -> Self {
        let guestid: i32 = value.get("guestid");
        let name: String = value.get("name");
        let attending: i32 = value.get("attending");
        let diet_options: Vec<String> = value.get("diet");

        DbGuest {
            guestid,
            name,
            attending: match attending {
                0 => DbAttendance::NotResponded,
                1 => DbAttendance::Attending,
                2 => DbAttendance::NotAttending,
                _ => DbAttendance::NotResponded,
            },
            diet_options: diet_options,
        }
    }
}
