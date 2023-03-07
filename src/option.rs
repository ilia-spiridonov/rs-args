use super::{OptionalArg, OptionalArgKind};

impl OptionalArg {
    pub fn new(kind: OptionalArgKind, multiple: bool) -> Self {
        Self { kind, multiple }
    }
}
