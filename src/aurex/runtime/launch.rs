#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LaunchStage {
    Idle,
    Pending(&'static str),
}

pub struct LaunchIntentController {
    stage: LaunchStage,
}

impl LaunchIntentController {
    pub fn new() -> Self {
        Self {
            stage: LaunchStage::Idle,
        }
    }

    pub fn stage(&self) -> LaunchStage {
        self.stage
    }

    pub fn is_pending(&self) -> bool {
        matches!(self.stage, LaunchStage::Pending(_))
    }

    pub fn request(&mut self, title: &'static str) -> bool {
        if matches!(self.stage, LaunchStage::Pending(current) if current == title) {
            return false;
        }

        self.stage = LaunchStage::Pending(title);
        true
    }

    pub fn cancel(&mut self) -> bool {
        if matches!(self.stage, LaunchStage::Idle) {
            return false;
        }

        self.stage = LaunchStage::Idle;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::{LaunchIntentController, LaunchStage};

    #[test]
    fn request_sets_pending_and_cancel_resets_idle() {
        let mut launch = LaunchIntentController::new();
        assert_eq!(launch.stage(), LaunchStage::Idle);

        assert!(launch.request("NEON CIRCUIT"));
        assert_eq!(launch.stage(), LaunchStage::Pending("NEON CIRCUIT"));
        assert!(launch.is_pending());

        assert!(launch.cancel());
        assert_eq!(launch.stage(), LaunchStage::Idle);
        assert!(!launch.is_pending());
    }
}
