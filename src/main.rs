mod models;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::sync::{Arc, Mutex};
use axum::{routing::get, Json, Router};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, post};
use serde::{Deserialize, Serialize};
use models::{Poll, VotingError};
use crate::models::{PollCreationRequest, PollMetadata, Vote};

static STATE_FILENAME: &str = "polls.json";

#[derive(Serialize, Deserialize, Clone)]
struct AppState {
    polls: Arc<Mutex<HashMap<usize, Poll>>>,
    poll_counter: Arc<Mutex<usize>>
}

fn load_state_from_file() -> Result<AppState, std::io::Error> {
    let state_file = File::open(STATE_FILENAME)?;
    let reader = BufReader::new(state_file);
    let state = serde_json::from_reader(reader)?;
    Ok(state)
}

fn initialize_state() -> AppState {
    load_state_from_file().unwrap_or_else(|err| {
        println!("failed to load state from file: {err}");
        AppState {
            poll_counter: Arc::new(Mutex::new(0)),
            polls: Arc::new(Mutex::new(
                HashMap::<usize, Poll>::new()
            ))
        }
    })
}

fn get_new_id(counter: &mut usize) -> usize {
    *counter = *counter + 1;
    counter.clone()
}

#[axum::debug_handler]
async fn save_state(
    State(state): State<AppState>,
) -> Result<(), (StatusCode, String)> {
    let state_file: File = File::create(STATE_FILENAME).map_err(|_| {
        println!("state File already exists");
        (StatusCode::INTERNAL_SERVER_ERROR, "could not open state file for writing".to_string())
    })?;
    let writer = BufWriter::new(state_file);
    serde_json::to_writer(writer, &state).map_err(
        |_| { (StatusCode::INTERNAL_SERVER_ERROR, "could not write to file".to_string()) }
    )
}

#[axum::debug_handler]
async fn list_polls (
    // access the state via the `State` extractor
    // extracting a state of the wrong type results in a compile error
    State(state): State<AppState>,
) -> Json<Vec<PollMetadata>> {
    let polls = state.polls.lock().unwrap();
    Json(polls.values().map(|poll| poll.metadata.clone()).collect())
}

#[axum::debug_handler]
async fn create_poll (
    State(state): State<AppState>,
    Json(poll_creation_request): Json<PollCreationRequest>,
) {
    let poll_id = get_new_id(&mut state.poll_counter.lock().unwrap());
    let mut polls = state.polls.lock().unwrap();
    polls.insert(poll_id, Poll::new(
        poll_id,
        poll_creation_request.candidates,
        poll_creation_request.min_score,
        poll_creation_request.max_score
    ));
}

#[axum::debug_handler]
async fn get_poll (
    State(state): State<AppState>,
    Path(poll_id): Path<usize>,
) -> Result<Json<PollMetadata>, (StatusCode, String)> {
    let polls = state.polls.lock().unwrap();
    let poll_option = polls.get(&poll_id);
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
    polls.remove(&poll_id)
        .map(|_| ())
        .ok_or(
            (StatusCode::NOT_FOUND, format!("poll with id {} not found", poll_id))
        )
}

#[axum::debug_handler]
async fn add_vote (
    State(state): State<AppState>,
    Path(poll_id): Path<usize>,
    Json(vote): Json<Vote>
) -> Result<(), (StatusCode, String)>{
    // FIXME: Add check for correct vec length
    let mut polls = state.polls.lock().unwrap();
    let poll_option = polls.get_mut(&poll_id);
    if let Some(poll) = poll_option {
        match poll.add_vote(vote.clone()) {
            Ok(_) => { Ok(()) }
            Err(VotingError::InvalidVoteLengthError) => {
                Err((StatusCode::UNPROCESSABLE_ENTITY, format!("vote was incorrect length (should be {})", poll.metadata.candidates.len())))
            }
            Err(VotingError::OutsideScoreRangeError) => {
                Err((StatusCode::UNPROCESSABLE_ENTITY, format!("vote contained score outside accepted range: [{}, {}]", poll.metadata.min_score, poll.metadata.max_score)))
            }
        }
    } else {
        Err((StatusCode::NOT_FOUND, format!("poll not found: {}", poll_id)))
    }
}

#[axum::debug_handler]
async fn list_votes (
    State(state): State<AppState>,
    Path(poll_id): Path<usize>
) -> Result<Json<Vec<Vote>>, (StatusCode, String)>{
    // FIXME: Add check for correct vec length
    let mut polls = state.polls.lock().unwrap();
    let poll_option = polls.get_mut(&poll_id);
    if let Some(poll) = poll_option {
        Ok(Json(poll.list_votes()))
    } else {
        Err((StatusCode::NOT_FOUND, format!("poll not found: {}", poll_id)))
    }
}

#[tokio::main]
async fn main() {
    let state = initialize_state();

    // create a `Router` that holds our state
    let app = Router::new()
        .route("/polls", post(create_poll))
        .route("/polls", get(list_polls))
        .route("/polls/:poll_id", get(get_poll))
        .route("/polls/:poll_id", delete(delete_poll))
        .route("/polls/:poll_id/votes", post(add_vote))
        .route("/polls/:poll_id/votes", get(list_votes))
        .route("/save_state", post(save_state))
        // provide the state so the router can access it
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}