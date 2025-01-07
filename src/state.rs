use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use crate::models::Poll;
use crate::auth::{User, UserRole};

pub static STATE_FILENAME: &str = "polls.json";

#[derive(Serialize, Deserialize, Clone)]
pub struct AppState {
    pub polls: Arc<Mutex<HashMap<usize, Poll>>>,
    pub users: Arc<Mutex<HashMap<String, User>>>,
    poll_counter: Arc<Mutex<usize>>
}

impl AppState {
    pub fn get_new_id(&mut self) -> usize {
        let mut counter = self.poll_counter.lock().unwrap();
        *counter = *counter + 1;
        counter.clone()
    }
}

fn load_state_from_file() -> Result<AppState, std::io::Error> {
    let state_file = File::open(STATE_FILENAME)?;
    let reader = BufReader::new(state_file);
    let state = serde_json::from_reader(reader)?;
    Ok(state)
}

pub fn initialize_state() -> AppState {
    load_state_from_file().unwrap_or_else(|err| {
        println!("failed to load state from file: {err}");
        const ADMIN_USERNAME: &str = "admin";
        const ADMIN_PASSWORD: &str = "password"; // FIXME: change this
        AppState {
            users: Arc::new(Mutex::new(HashMap::from([
                (ADMIN_USERNAME.to_string(), User::new(
                    ADMIN_USERNAME.to_string(), UserRole::Admin, ADMIN_PASSWORD.to_string()
                ))
            ]))),
            poll_counter: Arc::new(Mutex::new(0)),
            polls: Arc::new(Mutex::new(
                HashMap::<usize, Poll>::new()
            ))
        }
    })
}

#[derive(Debug)]
pub enum SaveStateError {
    CreateFileError,
    WriteFileError,
}

pub fn save_state_to_file(state: &AppState) -> Result<(), SaveStateError> {
    let state_file: File = File::create(STATE_FILENAME).map_err(|_| {SaveStateError::CreateFileError})?;
    let writer = BufWriter::new(state_file);
    serde_json::to_writer(writer, &state).map_err(|_| {SaveStateError::WriteFileError})
}