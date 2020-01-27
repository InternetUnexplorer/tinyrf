use crate::common::render_task::{FileExt, Frame};
use std::collections::{HashSet, VecDeque};
use std::fmt;
use uuid::Uuid;

/// A project submitted to the server
#[derive(Debug)]
pub(super) struct Project {
    pub uuid: Uuid,
    pub name: String,
    pub output_ext: FileExt,
    pub waiting_frames: VecDeque<Frame>,
    pub assigned_frames: HashSet<Frame>,
    pub completed_frames: VecDeque<Frame>,
    pub failed_frames: VecDeque<Frame>,
}

impl Project {
    /// Create a new project with the specified name
    pub(super) fn new(
        name: String,
        output_ext: FileExt,
        start_frame: Frame,
        end_frame: Frame,
    ) -> Project {
        assert!(start_frame <= end_frame);

        let waiting_frames = (start_frame..=end_frame).collect();

        Project {
            uuid: Uuid::new_v4(),
            name,
            output_ext,
            waiting_frames,
            assigned_frames: HashSet::new(),
            completed_frames: VecDeque::new(),
            failed_frames: VecDeque::new(),
        }
    }

    /// Check whether all of the frames have been rendered
    pub(super) fn complete(&self) -> bool {
        self.num_completed() == self.num_frames()
    }

    /// Get the ratio of frames that have been completed
    pub(super) fn progress(&self) -> f32 {
        (self.num_completed() as f32) / (self.num_frames() as f32)
    }

    /// Get the total number of frames
    pub(super) fn num_frames(&self) -> Frame {
        self.num_waiting() + self.num_assigned() + self.num_completed() + self.num_failed()
    }

    /// Get the number of waiting frames
    pub(super) fn num_waiting(&self) -> Frame {
        self.waiting_frames.len() as Frame
    }

    /// Get the number of assigned frames
    pub(super) fn num_assigned(&self) -> Frame {
        self.assigned_frames.len() as Frame
    }

    /// Get the number of completed frames
    pub(super) fn num_completed(&self) -> Frame {
        self.completed_frames.len() as Frame
    }

    /// Get the number of failed frames
    pub(super) fn num_failed(&self) -> Frame {
        self.failed_frames.len() as Frame
    }

    /// Move all of the failed frames back to the waiting queue
    pub(super) fn retry_failed(&mut self) {
        self.waiting_frames.append(&mut self.failed_frames);
    }
}

impl fmt::Display for Project {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
