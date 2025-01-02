use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::models::VotingError::{InvalidVoteLengthError, OutsideScoreRangeError};

#[derive(Clone, Serialize, Deserialize)]
pub struct Vote(Arc<[u8]>);

#[derive(Clone, Serialize, Deserialize)]
pub struct Poll {
    pub metadata: PollMetadata,
    votes: Vec<Vote>
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PollMetadata {
    pub id: usize,
    pub candidates: Arc<[Arc<str>]>,
    pub min_score: u8,
    pub max_score: u8
}

#[derive(Clone, Deserialize)]
pub struct PollCreationRequest {
    pub candidates: Arc<[Arc<str>]>,
    pub min_score: u8,
    pub max_score: u8
}

impl Poll {
    pub fn new(id: usize, candidates: Arc<[Arc<str>]>, min_score: u8, max_score: u8) -> Self {
        Self {
            metadata: PollMetadata {
                id,
                candidates,
                min_score,
                max_score
            },
            votes: Vec::new(),
        }
    }

    pub fn list_votes(&self) -> Vec<Vote> {
        self.votes.clone()
    }
}

pub enum VotingError {
    OutsideScoreRangeError,
    InvalidVoteLengthError,
}

impl Poll {

    pub fn add_vote(&mut self, vote: Vote) -> Result<(), VotingError> {
        if vote.0.len() != self.metadata.candidates.len() {
            Err(InvalidVoteLengthError)
        } else if vote.0.iter().any(|score| (score < &self.metadata.min_score) | (score > &self.metadata.max_score)) {
            Err(OutsideScoreRangeError)
        } else {
            self.votes.push(vote);
            Ok(())
        }
    }
}