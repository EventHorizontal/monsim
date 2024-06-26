pub mod terrain;
pub mod trap;
pub mod weather;

pub use terrain::*;
pub use trap::*;
pub use weather::*;

use crate::PerTeam;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Environment {
    pub weather: Option<Weather>,
    pub terrain: Option<Terrain>,
    pub traps: PerTeam<Option<Trap>>,
}

impl Environment {
    pub fn weather(&self) -> Option<&Weather> {
        self.weather.as_ref()
    }

    pub fn weather_mut(&mut self) -> Option<&mut Weather> {
        self.weather.as_mut()
    }

    pub fn terrain(&self) -> Option<&Terrain> {
        self.terrain.as_ref()
    }

    pub fn terrain_mut(&mut self) -> Option<&mut Terrain> {
        self.terrain.as_mut()
    }

    pub fn traps(&self) -> &PerTeam<Option<Trap>> {
        &self.traps
    }

    pub fn traps_mut(&mut self) -> &mut PerTeam<Option<Trap>> {
        &mut self.traps
    }
}
