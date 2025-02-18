
use warp::{Filter, Rejection, Reply};
use warp::reply::{html,json};
use rusqlite::Result;

mod database;

async fn get_tweaks(block_hash: String, db_path: String) -> Result<impl Reply, Rejection> {
    match database::fetch_tweaks(block_hash, &db_path) {
        Ok(tweaks) => Ok(json(&tweaks)),
        Err(err) => Ok(json(&err.to_string())),
    }
}

async fn get_tweak_metrics(db_path: String) -> Result<impl Reply, Rejection> {
    match database::get_tweak_metrics(&db_path) {
        Ok(tweaks) => {
            let mut response = String::from("<html><body><table border='1'><tr><th>Block Hash</th><th>Tweak Count</th></tr>");
            
            for tweak in tweaks {
                response.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td></tr>",
                    tweak.block_hash, tweak.tweak_count
                ));
            }
            response.push_str("</table></body></html>");
            Ok(html(response))
        },
        Err(err) => Ok(html(err.to_string())),
    }
}

async fn get_status(db_path: String) -> Result<impl Reply, Rejection> {
    match database::get_highest_block(&db_path) {
        Ok(height) => Ok(json(&height)),
        Err(err) => Ok(json(&err.to_string())),
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
    let tweak_metrics = warp::path!("block_stats")
    .and(with_db_path(db_path.clone()))
    .and_then(get_tweak_metrics);
    let status_route = warp::path!("status")
    .and(with_db_path(db_path.clone()))
    .and_then(get_status);

    let routes = tweaks_route
    .or(status_route)
    .or(tweak_metrics);

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}