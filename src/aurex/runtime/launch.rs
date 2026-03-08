#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LaunchDescriptor {
    pub title: &'static str,
    pub cartridge_id: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LaunchStage {
    Idle,
    Pending(LaunchDescriptor),
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

    pub fn request(&mut self, request: LaunchDescriptor) -> bool {
        if matches!(self.stage, LaunchStage::Pending(current) if current == request) {
            return false;
        }

        self.stage = LaunchStage::Pending(request);
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
    use super::{LaunchDescriptor, LaunchIntentController, LaunchStage};

    #[test]
    fn request_sets_pending_and_cancel_resets_idle() {
        let mut launch = LaunchIntentController::new();
        assert_eq!(launch.stage(), LaunchStage::Idle);

        let req = LaunchDescriptor {
            title: "NEON CIRCUIT",
            cartridge_id: "neon_circuit",
        };

        assert!(launch.request(req));
        assert_eq!(launch.stage(), LaunchStage::Pending(req));
        assert!(launch.is_pending());

        assert!(launch.cancel());
        assert_eq!(launch.stage(), LaunchStage::Idle);
        assert!(!launch.is_pending());
    }
}
