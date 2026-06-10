use crate::message::{AppendEntriesArgs, AppendEntriesReply, RequestVoteArgs, RequestVoteReply};
use crate::log::ReplicatedLog;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaftRole {
    Follower,
    Candidate,
    Leader,
}

pub struct RaftNode {
    pub id: String,
    
    // Persistent state
    pub current_term: RwLock<u64>,
    pub voted_for: RwLock<Option<String>>,
    pub log: RwLock<ReplicatedLog>,

    // Volatile state
    pub commit_index: RwLock<usize>,
    pub last_applied: RwLock<usize>,

    // Role state
    pub role: RwLock<RaftRole>,
}

impl RaftNode {
    pub fn new(id: String) -> Self {
        Self {
            id,
            current_term: RwLock::new(0),
            voted_for: RwLock::new(None),
            log: RwLock::new(ReplicatedLog::new()),
            commit_index: RwLock::new(0),
            last_applied: RwLock::new(0),
            role: RwLock::new(RaftRole::Follower),
        }
    }

    pub fn handle_request_vote(&self, args: RequestVoteArgs) -> RequestVoteReply {
        let mut current_term = self.current_term.write();
        let mut voted_for = self.voted_for.write();
        let log = self.log.read();

        if args.term > *current_term {
            *current_term = args.term;
            *self.role.write() = RaftRole::Follower;
            *voted_for = None;
        }

        let mut vote_granted = false;
        
        if args.term == *current_term {
            let can_vote = voted_for.is_none() || voted_for.as_deref() == Some(&args.candidate_id);
            
            // Raft log completeness check
            let my_last_log_term = log.last_log_term();
            let my_last_log_index = log.last_log_index();
            
            let log_ok = args.last_log_term > my_last_log_term || 
                (args.last_log_term == my_last_log_term && args.last_log_index >= my_last_log_index);

            if can_vote && log_ok {
                vote_granted = true;
                *voted_for = Some(args.candidate_id);
            }
        }

        RequestVoteReply {
            term: *current_term,
            vote_granted,
        }
    }

    pub fn handle_append_entries(&self, args: AppendEntriesArgs) -> AppendEntriesReply {
        let mut current_term = self.current_term.write();
        let mut log = self.log.write();

        if args.term > *current_term {
            *current_term = args.term;
            *self.role.write() = RaftRole::Follower;
            *self.voted_for.write() = None;
        }

        if args.term < *current_term {
            return AppendEntriesReply {
                term: *current_term,
                success: false,
                conflict_index: None,
                conflict_term: None,
            };
        }

        // We acknowledge the leader
        *self.role.write() = RaftRole::Follower;

        // Check if log contains an entry at prevLogIndex whose term matches prevLogTerm
        let my_prev_term = log.get_term_at(args.prev_log_index);
        
        if args.prev_log_index > 0 && my_prev_term != Some(args.prev_log_term) {
            // Conflict! Tell leader to back up
            return AppendEntriesReply {
                term: *current_term,
                success: false,
                conflict_index: Some(log.len() + 1), // Simplification: just tell them our log end
                conflict_term: None,
            };
        }

        // Truncate and Append
        if !args.entries.is_empty() {
            log.truncate_from(args.prev_log_index + 1);
            log.append(args.entries);
        }

        // Update commit index
        if args.leader_commit > *self.commit_index.read() {
            let new_commit = std::cmp::min(args.leader_commit, log.last_log_index());
            *self.commit_index.write() = new_commit;
        }

        AppendEntriesReply {
            term: *current_term,
            success: true,
            conflict_index: None,
            conflict_term: None,
        }
    }
}
