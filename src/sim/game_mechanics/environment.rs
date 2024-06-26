pub mod entry_hazard;
pub mod terrain;
pub mod weather;

pub use entry_hazard::*;
pub use terrain::*;
pub use weather::*;

use crate::PerTeam;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Environment {
    pub weather: Option<Weather>,
    pub terrain: Option<Terrain>,
    pub entry_hazards: PerTeam<Option<EntryHazard>>,
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

    pub fn entry_hazards(&self) -> &PerTeam<Option<EntryHazard>> {
        &self.entry_hazards
    }

    pub fn entry_hazards_mut(&mut self) -> &mut PerTeam<Option<EntryHazard>> {
        &mut self.entry_hazards
    }
}
