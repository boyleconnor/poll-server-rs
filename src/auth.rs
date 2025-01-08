use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use chrono::prelude::{DateTime, Utc};

const SALT_LENGTH: usize = 16;  // in bytes

pub(crate) type Username = String;

#[derive(Serialize, Deserialize, Clone)]
pub(crate) enum UserRole {
    Admin,
    General
}

pub(crate) type SessionId = String;

#[derive(Serialize, Deserialize)]
pub(crate) struct UserSession {
    #[serde()]
    pub expiration: DateTime<Utc>,
    pub username: Username
}

#[derive(Serialize, Deserialize)]
pub(crate) struct User {
    role: UserRole,
    password_salt: Vec<u8>,
    password_hash: Vec<u8>,
}

#[derive(Deserialize)]
pub(crate) struct LoginRequest {
    pub username: Username,
    pub password: String
}

impl User {
    pub(crate) fn new(role: UserRole, password: String) -> User {
        let mut password_salt = vec![0u8; SALT_LENGTH];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut password_salt);

        let password_hash = Self::get_salted_hash(password, &password_salt);

        User {
            role,
            password_salt,
            password_hash
        }
    }

    pub(crate) fn authenticate(&self, password: String) -> bool {
        self.password_hash == Self::get_salted_hash(password, &self.password_salt)
    }

    fn get_salted_hash(password: String, password_salt: &Vec<u8>) -> Vec<u8> {
        let mut salted_password = password_salt.clone();
        salted_password.extend(password.as_bytes().iter());
        let password_hash = Sha512::digest(salted_password).to_vec();
        password_hash
    }
}
