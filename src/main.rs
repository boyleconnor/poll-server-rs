use std::string::ToString;
use std::sync::{Arc, Mutex};
use axum::{routing::get, Json, Router};
use axum::extract::{Path, State};
use axum::routing::post;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
struct AppState {
    polls: Arc<Mutex<Vec<Poll>>>
}

#[derive(Clone, Serialize, Deserialize)]
struct ScoreRange(u8, u8);

#[derive(Clone, Serialize, Deserialize)]
struct Poll {
    candidates: Vec<String>,
    score_range: ScoreRange,
    votes: Vec<Vec<u8>>
}

impl Poll {
    // FIXME: Figure out how to run this at validation time
    fn is_valid(&self) -> bool {
        self.votes.iter().all(|vote| self.candidates.len() == vote.len())
    }
}

async fn list_polls (
    // access the state via the `State` extractor
    // extracting a state of the wrong type results in a compile error
    State(state): State<AppState>,
) -> Json<Vec<Poll>> {
    Json(state.polls.lock().unwrap().to_vec())
}

#[axum::debug_handler]
async fn create_poll (
    State(state): State<AppState>,
    Json(new_poll): Json<Poll>,
) {
    let mut polls = state.polls.lock().unwrap();
    polls.push(new_poll);
}

async fn add_vote (
    State(state): State<AppState>,
    Path(poll_id): Path<usize>,
    Json(vote): Json<Vec<u8>>
) {
    // FIXME: Add check for correct vec length
    let mut polls = state.polls.lock().unwrap();
    let poll = polls.get_mut(poll_id).unwrap();
    poll.votes.push(vote);

}

#[tokio::main]
async fn main() {
    let state = AppState {
        polls: Arc::new(Mutex::new(
            vec![Poll {
                score_range: ScoreRange(0, 5),
                candidates: vec![
                    "O'Brien".to_string(),
                    "Murphy".to_string()
                ],
                votes: vec![]
            }]
        ))
    };

    // create a `Router` that holds our state
    let app = Router::new()
        .route("/polls", get(list_polls))
        .route("/polls", post(create_poll))
        .route("/vote/:poll_id", post(add_vote))
        // provide the state so the router can access it
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}