pub mod terrain;
pub mod weather;
pub use terrain::*;
pub use weather::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Environment {
    pub weather: Option<Weather>,
    pub terrain: Option<Terrain>,
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
}
