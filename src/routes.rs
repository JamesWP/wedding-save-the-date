use warp::Filter;

use crate::db::Db;
use crate::model::DbInvite;
use sendmail::email;

//https://github.com/seanmonstar/&&warp/blob/master/examples/todos.rs

pub fn routes(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let content_root = std::env::var("CONTENT_ROOT").unwrap();
    let index_page = format!("{}/index.html", &content_root);

    let index_html = warp::get().and(warp::path("index.html")).and(warp::fs::file(index_page.clone()));
    let root = warp::get().and(warp::path::end()).and(warp::fs::file(index_page.clone()));
    let static_data = warp::path("static").and(warp::fs::dir(content_root));

    let invite_lookup = warp::path("invite").and(warp::path("lookup")).and(warp::path::end()).and(warp::post()).and(with_db(db.clone())).and(warp::body::json()).and_then(handlers::invite_lookup);
    let invite = warp::path("invite").and(warp::path::param()).and(warp::path::end()).and(warp::get()).and(with_db(db.clone())).and_then(handlers::get_invite);
    let rsvp = warp::path("rsvp").and(warp::path::param()).and(warp::path::end()).and(warp::put()).and(with_db(db.clone())).and(warp::body::json()).and_then(handlers::save_rsvp);
    let api = warp::path("api").and(invite.or(invite_lookup).or(rsvp));

    static_data.or(index_html).or(root).or(api)
}

fn with_db(db: Db) -> impl Filter<Extract = (Db,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

fn send_receipt(invite: &DbInvite) {
    let body = format!("
<html><body>
Thanks for sending your RSVP.<br/>
If you want to update the response you can go back to <a href='https://carolineandjames.peach.family'>the website</a> and click send again.<br/>
<br/>
Caroline and James. x<br/>
<!--{:#?}-->
</body></html>", invite);
    let result = email::send(
        // From Address
        "<Caroline and James>carolineandjamespeach@gmail.com",
        // To Address
        &[invite.email.as_str()],
        // Subject
        "Invite RSVP",
        // Body
        &body 
    );

    match result {
        Err(e) => {
            log::info!("Error sending mail {:?}", e);
        }, 
        Ok(_) => {
            log::info!("Sending mail");
        },
    };
}
                

mod handlers {
    use std::convert::Infallible;

    use serde::{Deserialize, Serialize};
    use warp::hyper::StatusCode;

    use crate::{
        db::{self, Db}, model::{DbAttendance, DbGuest},
    };

    use crate::routes::send_receipt; 

    pub async fn get_invite(
        invite_key: String,
        db: Db,
    ) -> Result<Box<dyn warp::Reply>, Infallible> {
        let invite = db::get_invite(&db, db::InviteIdentifier::InviteKey(invite_key)).await;

        if let Some(invite) = invite {
            Ok(Box::new(warp::reply::json(&invite)))
        } else {
            Ok(Box::new(StatusCode::NOT_FOUND))
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct ApiGuest {
        guestid: i32,
        attending: DbAttendance,
        diet_options: Vec<String>,
    }

    #[derive(Deserialize, Debug)]
    pub struct ApiRsvp {
        guests: Vec<ApiGuest>,
    }

    pub async fn save_rsvp(
        invite_key: String,
        db: Db,
        data: ApiRsvp,
    ) -> Result<Box<dyn warp::Reply>, Infallible> {
        let invite = db::get_invite(&db, db::InviteIdentifier::InviteKey(invite_key)).await;
       
        let invite = match invite {
            Some(invite) => invite,
            None => return Ok(Box::new(StatusCode::NOT_FOUND)),
        };

        let guests: Vec<_> = data.guests.iter().map(|guest| DbGuest {guestid:guest.guestid, name: "".to_owned(), attending: guest.attending, diet_options: guest.diet_options.clone() }).collect();
        let result = db::save_guests(&db, invite.uid, &guests).await;

        match result {
            Ok(updated_invite) => {
                log::info!("Updated invite {:?}", updated_invite);
                if let Some(updated_invite) = &updated_invite {
                    send_receipt(updated_invite);
                }
                Ok(Box::new(StatusCode::OK))
            },
            Err(e) => {
                log::error!("Unable to update invite {}. request={:?}", e, data);
                Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR))
            }
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct ApiLookup {
        email: String,
        password: String,
    }

    #[derive(Serialize)]
    pub struct ApiLookupResponse {
        invite_key: String
    }

    pub async fn invite_lookup(
        db: Db,
        data: ApiLookup,
    ) -> Result<Box<dyn warp::Reply>, Infallible> {

        let key = db::open_invite(&db, &data.email, &data.password).await;

        match key {
            Ok(Some(key)) => {
                log::info!("Invite opened. {} key={}", data.email, key);
                Ok(Box::new(warp::reply::json(&ApiLookupResponse{ invite_key: key })))
            },
            Ok(None) => {
                log::info!("Invite not found. email={} password={}", data.email, data.password);
                Ok(Box::new(StatusCode::NOT_FOUND))
            },
            Err(e) => {
                log::error!("Unable to open invite. {} email={} password={}", e,  data.email, data.password);
                Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR))
            }
        }
    }
}
