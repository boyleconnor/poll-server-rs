use serde::{Deserialize, Serialize};
use crate::models::VotingError::{InvalidVoteLengthError, OutsideScoreRangeError};

#[derive(Clone, Serialize, Deserialize)]
pub struct Poll {
    pub metadata: PollMetadata,
    votes: Vec<Vec<u8>>
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PollMetadata {
    pub id: usize,
    pub candidates: Vec<String>,
    pub min_score: u8,
    pub max_score: u8
}

impl Poll {
    pub fn new(id: usize, candidates: Vec<String>, min_score: u8, max_score: u8) -> Self {
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
}

pub enum VotingError {
    OutsideScoreRangeError,
    InvalidVoteLengthError,
}

impl Poll {

    pub fn add_vote(&mut self, vote: Vec<u8>) -> Result<(), VotingError> {
        if vote.len() != self.metadata.candidates.len() {
            Err(InvalidVoteLengthError)
        } else if vote.iter().any(|score| (score < &self.metadata.min_score) | (score > &self.metadata.max_score)) {
            Err(OutsideScoreRangeError)
        } else {
            self.votes.push(vote);
            Ok(())
        }
    }
}