use super::{Type, Percent};

pub const INEFFECTIVE: Percent = Percent(0);
pub const NOT_VERY_EFFECTIVE: Percent = Percent(50);
pub const EFFECTIVE: Percent = Percent(100);
pub const SUPER_EFFECTIVE: Percent = Percent(200);

pub const EMPTY_LINE: &str = "";