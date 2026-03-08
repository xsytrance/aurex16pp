#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LaunchDescriptor {
    pub title: &'static str,
    pub cartridge_id: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LaunchValidationError {
    EmptyCartridgeId,
    InvalidCartridgeId,
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

#[cfg(test)]
mod validation_tests {
    use super::{LaunchDescriptor, LaunchValidationError, validate_launch_descriptor};

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
