use phf::phf_map;

pub const NEVER_MISS: u8 = 0u8;
pub const INEFFECTIVE: f32 = 0.0f32;
pub const MIN_POSSIBLE_DAMAGE: u16 = 1u16;
pub const ACCURACY_STAGE_TO_MULTIPLIER: phf::Map<i8, f32> = phf_map!(
        -6i8 => 0.333f32,
        -5i8 => 0.375f32,
        -4i8 => 0.429f32,
        -3i8 => 0.500f32,
        -2i8 => 0.600f32,
        -1i8 => 0.750f32,
         0i8 => 1.000f32,
         1i8 => 1.333f32,
         2i8 => 1.667f32,
         3i8 => 2.000f32,
         4i8 => 2.333f32,
         5i8 => 2.667f32,
         6i8 => 3.000f32,
);