use std::env;

use warp::Filter;

mod routes;
mod termination;
mod db;
mod db_error;
mod model;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "wedding=debug");
    }

    pretty_env_logger::init(); 
    
    let db = db::init_db().await?;

    let api = routes::routes(db);
    let api = api.with(warp::log("wedding"));
    let (_, server) = warp::serve(api)
         .bind_with_graceful_shutdown(([0, 0, 0, 0], 3030), termination::termination_future());

    let (r,) = tokio::join!(tokio::task::spawn(server));
    
    r?;

    println!("terminating");
    Ok(())
}
