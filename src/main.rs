mod models;
mod state;

use axum::{routing::get, Json, Router};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, post};
use models::{Poll, VotingError};
use state::AppState;
use crate::models::{PollCreationRequest, PollMetadata, Vote};

#[axum::debug_handler]
async fn save_state(
    State(app_state): State<AppState>,
) -> Result<(), (StatusCode, String)> {
    state::save_state_to_file(&app_state).map_err(|e| {
        println!("Failed to save state: {e:?}");
        (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string())
    })
}

#[axum::debug_handler]
async fn list_polls (
    // access the state via the `State` extractor
    // extracting a state of the wrong type results in a compile error
    State(app_state): State<AppState>,
) -> Json<Vec<PollMetadata>> {
    let polls = app_state.polls.lock().unwrap();
    Json(polls.values().map(|poll| poll.metadata.clone()).collect())
}

#[axum::debug_handler]
async fn create_poll (
    State(mut app_state): State<AppState>,
    Json(poll_creation_request): Json<PollCreationRequest>,
) {
    let poll_id = app_state.get_new_id();
    let mut polls = app_state.polls.lock().unwrap();
    polls.insert(poll_id, Poll::new(
        poll_id,
        poll_creation_request.candidates,
        poll_creation_request.min_score,
        poll_creation_request.max_score
    ));
}

#[axum::debug_handler]
async fn get_poll (
    State(app_state): State<AppState>,
    Path(poll_id): Path<usize>,
) -> Result<Json<PollMetadata>, (StatusCode, String)> {
    let polls = app_state.polls.lock().unwrap();
    let poll_option = polls.get(&poll_id);
    match poll_option {
        Some(poll) => { Ok(Json(poll.metadata.clone())) }
        None => { Err((StatusCode::NOT_FOUND, format!("poll with id {} not found", poll_id))) }
    }
}

#[axum::debug_handler]
async fn delete_poll (
    State(app_state): State<AppState>,
    Path(poll_id): Path<usize>,
) -> Result<(), (StatusCode, String)> {
    let mut polls = app_state.polls.lock().unwrap();
    polls.remove(&poll_id)
        .map(|_| ())
        .ok_or(
            (StatusCode::NOT_FOUND, format!("poll with id {} not found", poll_id))
        )
}

#[axum::debug_handler]
async fn add_vote (
    State(app_state): State<AppState>,
    Path(poll_id): Path<usize>,
    Json(vote): Json<Vote>
) -> Result<(), (StatusCode, String)>{
    // FIXME: Add check for correct vec length
    let mut polls = app_state.polls.lock().unwrap();
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
    State(app_state): State<AppState>,
    Path(poll_id): Path<usize>
) -> Result<Json<Vec<Vote>>, (StatusCode, String)>{
    // FIXME: Add check for correct vec length
    let mut polls = app_state.polls.lock().unwrap();
    let poll_option = polls.get_mut(&poll_id);
    if let Some(poll) = poll_option {
        Ok(Json(poll.list_votes()))
    } else {
        Err((StatusCode::NOT_FOUND, format!("poll not found: {}", poll_id)))
    }
}

#[tokio::main]
async fn main() {
    let app_state = state::initialize_state();

    // create a `Router` that holds our state
    let app = Router::new()
        .route("/polls", post(create_poll))
        .route("/polls", get(list_polls))
        .route("/polls/{poll_id}", get(get_poll))
        .route("/polls/{poll_id}", delete(delete_poll))
        .route("/polls/{poll_id}/votes", post(add_vote))
        .route("/polls/{poll_id}/votes", get(list_votes))
        .route("/save_state", post(save_state))
        // provide the state so the router can access it
        .with_state(app_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}