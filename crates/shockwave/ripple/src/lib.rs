pub mod epidemic_tree;
pub mod propagation;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RippleError {
    #[error("Node not found in the tree")]
    NodeNotFound,
    #[error("Invalid stake amount")]
    InvalidStake,
    // Add more error types as needed
}
