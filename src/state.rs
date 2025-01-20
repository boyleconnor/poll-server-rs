use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::ops::Add;
use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use chrono::{TimeDelta, Utc};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use crate::models::Poll;
use crate::auth::{SessionId, UserAuth, UserRole, UserSession, Username};

pub static STATE_FILENAME: &str = "polls.json";

#[derive(Serialize, Deserialize, Clone)]
pub struct AppState {
    pub(crate) polls: Arc<Mutex<HashMap<usize, Poll>>>,
    pub(crate) users: Arc<Mutex<HashMap<Username, UserAuth>>>,
    pub(crate) user_sessions: Arc<Mutex<HashMap<SessionId, UserSession>>>,
    key: Arc<Vec<u8>>,
    poll_counter: Arc<Mutex<usize>>
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        Key::from(&state.key)
    }
}

impl AppState {
    fn load_from_file() -> Result<AppState, std::io::Error> {
        let state_file = File::open(STATE_FILENAME)?;
        let reader = BufReader::new(state_file);
        let state = serde_json::from_reader(reader)?;
        Ok(state)
    }

    /// Attempt to load app state from file; otherwise make a new one with no polls & an admin user
    /// with a random password.
    pub fn initialize() -> AppState {
        Self::load_from_file().unwrap_or_else(|err| {
            println!("failed to load state from file: {err}");
            const ADMIN_USERNAME: &str = "admin";
            const ADMIN_PASSWORD_LENGTH: usize = 16;
            let admin_password: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(ADMIN_PASSWORD_LENGTH)
                .map(char::from)
                .collect();
            println!("admin password: {admin_password} (write this down; it will not be shown again)");

            AppState {
                users: Arc::new(Mutex::new(HashMap::from([
                    (ADMIN_USERNAME.to_string(), UserAuth::new(
                        UserRole::Admin, admin_password.to_string()
                    ))
                ]))),
                user_sessions: Arc::new(Mutex::new(HashMap::new())),
                poll_counter: Arc::new(Mutex::new(0)),
                polls: Arc::new(Mutex::new(
                    HashMap::<usize, Poll>::new()
                )),
                key: Arc::new(Key::generate().master().to_vec())
            }
        })
    }

    pub fn get_new_id(&mut self) -> usize {
        let mut counter = self.poll_counter.lock().unwrap();
        *counter = *counter + 1;
        counter.clone()
    }

    pub fn new_user_session(&mut self, username: Username) -> SessionId {
        const SESSION_LENGTH: TimeDelta = chrono::Duration::days(7);
        const SESSION_ID_LENGTH: usize = 7;

        let expiration = Utc::now().add(SESSION_LENGTH);
        let user_session = UserSession {
            expiration,
            username: username.clone()
        };
        let session_id: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(SESSION_ID_LENGTH)
            .map(char::from)
            .collect();
        self.user_sessions.lock().unwrap().insert(session_id.clone(), user_session);
        session_id
    }
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