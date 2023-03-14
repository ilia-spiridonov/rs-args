use super::{PositionalArg, PositionalArgKind};

impl PositionalArg {
    pub fn named() -> Self {
        Self {
            kind: PositionalArgKind::Named,
        }
    }

    pub fn rest() -> Self {
        Self {
            kind: PositionalArgKind::Rest,
        }
    }
}
