mod models;

use std::string::ToString;
use std::sync::{Arc, Mutex};
use axum::{routing::get, Json, Router};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, post};
use models::{Poll, VotingError};
use crate::models::PollMetadata;

#[derive(Clone)]
struct AppState {
    polls: Arc<Mutex<Vec<Poll>>>
}

async fn list_polls (
    // access the state via the `State` extractor
    // extracting a state of the wrong type results in a compile error
    State(state): State<AppState>,
) -> Json<Vec<PollMetadata>> {
    let polls = state.polls.lock().unwrap();
    Json(polls.iter().map(|poll| poll.metadata.clone()).collect())
}

#[axum::debug_handler]
async fn create_poll (
    State(state): State<AppState>,
    Json(poll_metadata): Json<PollMetadata>,
) {
    let mut polls = state.polls.lock().unwrap();
    polls.push(Poll::new(
        poll_metadata.candidates,
        poll_metadata.min_score,
        poll_metadata.max_score
    ));
}

#[axum::debug_handler]
async fn get_poll (
    State(state): State<AppState>,
    Path(poll_id): Path<usize>,
) -> Result<Json<PollMetadata>, (StatusCode, String)> {
    let polls = state.polls.lock().unwrap();
    let poll_option = polls.get(poll_id);
    match poll_option {
        Some(poll) => { Ok(Json(poll.metadata.clone())) }
        None => { Err((StatusCode::NOT_FOUND, format!("poll with id {} not found", poll_id))) }
    }
}

#[axum::debug_handler]
async fn delete_poll (
    State(state): State<AppState>,
    Path(poll_id): Path<usize>,
) -> Result<(), (StatusCode, String)> {
    let mut polls = state.polls.lock().unwrap();
    if poll_id < 0 || poll_id >= polls.len() {
        return Err((StatusCode::NOT_FOUND, format!("poll with id {} not found", poll_id)));
    }
    polls.remove(poll_id);
    Ok(())
}

async fn add_vote (
    State(state): State<AppState>,
    Path(poll_id): Path<usize>,
    Json(vote): Json<Vec<u8>>
) -> Result<(), (StatusCode, String)>{
    // FIXME: Add check for correct vec length
    let mut polls = state.polls.lock().unwrap();
    let poll_option = polls.get_mut(poll_id);
    if let Some(poll) = poll_option {
        match poll.add_vote(vote.clone()) {
            Ok(_) => { Ok(()) }
            Err(VotingError::InvalidVoteLengthError) => {
                Err((StatusCode::UNPROCESSABLE_ENTITY, format!("vote was incorrect length: {} (should be {})", vote.len(), poll.metadata.candidates.len())))
            }
            Err(VotingError::OutsideScoreRangeError) => {
                Err((StatusCode::UNPROCESSABLE_ENTITY, format!("vote contained score outside accepted range: [{}, {}]", poll.metadata.min_score, poll.metadata.max_score)))
            }
        }
    } else {
        Err((StatusCode::NOT_FOUND, format!("poll not found: {}", poll_id)))
    }
}

#[tokio::main]
async fn main() {
    let state = AppState {
        polls: Arc::new(Mutex::new(
            vec![Poll::new(
                vec![
                    "O'Brien".to_string(),
                    "Murphy".to_string(),
                    "Walsh".to_string()
                ],
                0,
                5
            )]
        ))
    };

    // create a `Router` that holds our state
    let app = Router::new()
        .route("/polls", post(create_poll))
        .route("/polls", get(list_polls))
        .route("/polls/:poll_id", get(get_poll))
        .route("/polls/:poll_id", delete(delete_poll))
        .route("/polls/:poll_id/votes", post(add_vote))
        // provide the state so the router can access it
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}