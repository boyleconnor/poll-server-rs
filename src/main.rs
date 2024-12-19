use std::string::ToString;
use axum::{routing::get, Json, Router};
use axum::extract::State;
use serde::Serialize;

#[derive(Clone)]
struct AppState {
    polls: Vec<Poll>
}

#[derive(Clone, Serialize)]
struct Poll {
    candidates: Vec<String>
}

async fn handler (
    // access the state via the `State` extractor
    // extracting a state of the wrong type results in a compile error
    State(state): State<AppState>,
) -> Json<Vec<Poll>> {
    Json(state.polls)
}

#[tokio::main]
async fn main() {
    let state = AppState {polls: vec![Poll {candidates: vec![
        "O'Brien".to_string(),
        "Murphy".to_string()
    ]}]};

    // create a `Router` that holds our state
    let app = Router::new()
        .route("/", get(handler))
        // provide the state so the router can access it
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}