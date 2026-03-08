#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LaunchDescriptor {
    pub title: &'static str,
    pub cartridge_id: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LaunchValidationError {
    EmptyCartridgeId,
    InvalidCartridgeId,
    CartridgeMissing,
    CartridgeManifestInvalid,
}

pub fn validate_launch_descriptor(desc: LaunchDescriptor) -> Result<(), LaunchValidationError> {
    if desc.cartridge_id.is_empty() {
        return Err(LaunchValidationError::EmptyCartridgeId);
    }

    let valid = desc
        .cartridge_id
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');

    if !valid {
        return Err(LaunchValidationError::InvalidCartridgeId);
    }

    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LaunchStage {
    Idle,
    Pending(LaunchDescriptor),
    Validating(LaunchDescriptor),
    Ready(LaunchDescriptor),
    Rejected(LaunchValidationError),
}

pub struct LaunchIntentController {
    stage: LaunchStage,
    validate_frames_left: u8,
}

impl LaunchIntentController {
    pub const VALIDATION_FRAMES: u8 = 18;

    pub fn new() -> Self {
        Self {
            stage: LaunchStage::Idle,
            validate_frames_left: 0,
        }
    }

    pub fn stage(&self) -> LaunchStage {
        self.stage
    }

    pub fn is_active(&self) -> bool {
        !matches!(self.stage, LaunchStage::Idle)
    }

    pub fn request(&mut self, request: LaunchDescriptor) -> bool {
        if matches!(self.stage, LaunchStage::Ready(current) if current == request) {
            return false;
        }

        self.stage = LaunchStage::Pending(request);
        self.validate_frames_left = Self::VALIDATION_FRAMES;
        true
    }

    pub fn reject(&mut self, reason: LaunchValidationError) -> bool {
        self.stage = LaunchStage::Rejected(reason);
        self.validate_frames_left = 0;
        true
    }

    pub fn cancel(&mut self) -> bool {
        if matches!(self.stage, LaunchStage::Idle) {
            return false;
        }

        self.stage = LaunchStage::Idle;
        self.validate_frames_left = 0;
        true
    }

    pub fn tick(&mut self) -> Option<LaunchStage> {
        match self.stage {
            LaunchStage::Pending(desc) => {
                self.stage = LaunchStage::Validating(desc);
                Some(self.stage)
            }
            LaunchStage::Validating(desc) => {
                if self.validate_frames_left > 0 {
                    self.validate_frames_left -= 1;
                }

                if self.validate_frames_left == 0 {
                    self.stage = LaunchStage::Ready(desc);
                    Some(self.stage)
                } else {
                    None
                }
            }
            LaunchStage::Idle | LaunchStage::Ready(_) | LaunchStage::Rejected(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LaunchDescriptor, LaunchIntentController, LaunchStage, LaunchValidationError,
        validate_launch_descriptor,
    };

    #[test]
    fn request_progresses_to_ready_after_validation_ticks() {
        let mut launch = LaunchIntentController::new();
        let req = LaunchDescriptor {
            title: "NEON CIRCUIT",
            cartridge_id: "neon_circuit",
        };

        assert!(launch.request(req));
        assert_eq!(launch.stage(), LaunchStage::Pending(req));

        assert_eq!(launch.tick(), Some(LaunchStage::Validating(req)));

        for _ in 0..(LaunchIntentController::VALIDATION_FRAMES - 1) {
            assert_eq!(launch.tick(), None);
        }

        assert_eq!(launch.tick(), Some(LaunchStage::Ready(req)));
        assert_eq!(launch.stage(), LaunchStage::Ready(req));
    }

    #[test]
    fn cancel_resets_to_idle() {
        let mut launch = LaunchIntentController::new();
        let req = LaunchDescriptor {
            title: "NEON CIRCUIT",
            cartridge_id: "neon_circuit",
        };

        assert!(launch.request(req));
        assert!(launch.cancel());
        assert_eq!(launch.stage(), LaunchStage::Idle);
        assert!(!launch.is_active());
    }

    #[test]
    fn invalid_cartridge_id_is_rejected() {
        let bad = LaunchDescriptor {
            title: "Bad",
            cartridge_id: "Bad-ID",
        };
        assert_eq!(
            validate_launch_descriptor(bad),
            Err(LaunchValidationError::InvalidCartridgeId)
        );
    }

    #[test]
    fn empty_cartridge_id_is_rejected() {
        let bad = LaunchDescriptor {
            title: "Bad",
            cartridge_id: "",
        };
        assert_eq!(
            validate_launch_descriptor(bad),
            Err(LaunchValidationError::EmptyCartridgeId)
        );
    }
}
