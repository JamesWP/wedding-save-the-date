use std::{error::Error, sync::Arc};

use tokio_postgres::{Client, NoTls};

use crate::{
    db_error::DbError,
    model::{DbGuest, DbInvite},
};

pub type Db = Arc<Client>;

pub async fn init_db() -> Result<Db, tokio_postgres::Error> {
    let connection_string = std::env::var("POSTGRES_CONN_STRING").expect("no connection string");

    let (client, connection) = loop {
        // Connect to the database.
        let result = tokio_postgres::connect(&connection_string, NoTls).await;

        match result {
            Ok(r) => break r,
            Err(e) => dbg!(e),
        };

        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    };

    dbg!("Got connection!");

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
            panic!("No db connection exiting!");
        }
    });

    Ok(Arc::new(client))
}

pub async fn open_invite(
    db: &Db,
    invite_email: &str,
    invite_password: &str,
) -> Result<Option<String>, Box<dyn Error>> {
    let result = db
        .query_opt(
            "select random from invite where lower(email) = lower($1::TEXT) and password = $2::TEXT",
            &[&invite_email, &invite_password],
        )
        .await?;

    Ok(result.map(|row| row.get("random")))
}

pub enum InviteIdentifier {
    InviteKey(String),
    InviteUid(i32),
}

pub async fn get_invite(db: &Db, invite_identifier: InviteIdentifier) -> Option<DbInvite> {
    
    let result = match invite_identifier {
        InviteIdentifier::InviteKey(invite_key) => {
            db
                .query_opt(
                    "select uid, email, names from invite where random = $1::TEXT",
                    &[&invite_key],
                ).await
        },
        InviteIdentifier::InviteUid(invite_uid) => 
        {
            db
                .query_opt(
                    "select uid, email, names from invite where uid = $1",
                    &[&invite_uid],
                ).await
        }
    };

    let mut invite = DbInvite::from(result.ok().flatten()?);

    let result = db
        .query(
            "select guestid, name, attending, diet from public.guest where invite_uid = $1",
            &[&invite.uid],
        )
        .await;

    let result = result.ok()?;

    invite.guests = result.into_iter().map(DbGuest::from).collect();

    Some(invite)
}

pub async fn save_guests(db: &Db, invite_uid: i32, guests: &[DbGuest]) -> Result<Option<DbInvite>, Box<dyn Error>> {
    db.execute("begin transaction", &[]).await?;

    let statement = db
        .prepare("update public.guest set attending = $3, diet = $4 where invite_uid = $1 and guestid = $2")
        .await?;
    for guest in guests {
        let attending = (guest.attending as u8) as i32;
        let affected = db
            .execute(&statement, &[&invite_uid, &guest.guestid, &attending, &guest.diet_options])
            .await?;

        if affected != 1 {
            db.execute("rollback", &[]).await?;
            return Err(Box::new(DbError::DidntUpdateGuest {
                guestid: guest.guestid,
                inviteid: invite_uid,
            }));
        }
    }

    let _result = db.query("commit", &[]).await?;

    let updated_invite = get_invite(db, InviteIdentifier::InviteUid(invite_uid)).await;

    Ok(updated_invite)
}
