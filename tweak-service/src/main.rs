
use warp::{Filter, Rejection, Reply};
use rusqlite::Result;

mod database;

async fn get_tweaks(block_hash: String, db_path: String) -> Result<impl Reply, Rejection> {
    match database::fetch_tweaks(block_hash, &db_path) {
        Ok(tweaks) => Ok(warp::reply::json(&tweaks)),
        Err(err) => Ok(warp::reply::json(&err.to_string())),
    }
}

async fn get_status(db_path: String) -> Result<impl Reply, Rejection> {
    match database::get_highest_block(&db_path) {
        Ok(height) => Ok(warp::reply::json(&height)),
        Err(err) => Ok(warp::reply::json(&err.to_string())),
    }
}

// Middleware to inject `db_path` into handler
fn with_db_path(db_path: String) -> impl Filter<Extract = (String,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db_path.clone())
}

#[tokio::main]
async fn main() {
    let db_path = String::from("blocks.db");
    let tweaks_route = warp::path!("tweaks" / String)
    .and(with_db_path(db_path.clone()))
    .and_then(get_tweaks);
    let status_route = warp::path!("status")
    .and(with_db_path(db_path.clone()))
    .and_then(get_status);

    let routes = tweaks_route
    .or(status_route);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}