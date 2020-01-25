use std::collections::VecDeque;
use uuid::Uuid;

/// A project submitted to the server
#[derive(Debug)]
pub struct Project {
    pub uuid: Uuid,
    pub name: String,
    pub waiting_frames: VecDeque<u32>,
    pub assigned_frames: VecDeque<u32>,
    pub completed_frames: VecDeque<u32>,
}

impl Project {
    /// Create a new project with the specified name
    pub fn new(name: String, start_frame: u32, end_frame: u32) -> Project {
        let waiting_frames = (start_frame..=end_frame).collect();

        Project {
            uuid: Uuid::new_v4(),
            name,
            waiting_frames,
            assigned_frames: VecDeque::new(),
            completed_frames: VecDeque::new(),
        }
    }

    /// Check whether all of the frames have been rendered
    pub fn complete(&self) -> bool {
        self.num_completed_frames() == self.num_frames()
    }

    /// Get the ratio of frames that have been completed
    pub fn progress(&self) -> f32 {
        (self.num_completed_frames() as f32) / (self.num_frames() as f32)
    }

    /// Get the total number of frames
    pub fn num_frames(&self) -> u32 {
        self.num_waiting_frames() + self.get_assigned_frames() + self.num_completed_frames()
    }

    /// Get the number of waiting frames
    pub fn num_waiting_frames(&self) -> u32 {
        self.waiting_frames.len() as u32
    }

    /// Get the number of assigned frames
    pub fn get_assigned_frames(&self) -> u32 {
        self.assigned_frames.len() as u32
    }

    /// Get the number of completed frames
    pub fn num_completed_frames(&self) -> u32 {
        self.completed_frames.len() as u32
    }
}
