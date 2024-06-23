pub mod terrain;
pub mod weather;
use monsim_utils::NOTHING;
pub use terrain::*;
pub use weather::*;

use crate::{
    sim::event_dispatcher::{Event, EventContext, EventHandlerWithOwnerEmbedded, EventReturnable},
    ActivationOrder, Broadcaster, EventHandlerWithOwner,
};

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

    pub(crate) fn owned_event_handlers<C: EventContext + 'static, R: EventReturnable + 'static, B: Broadcaster + 'static>(
        &self,
        event: &impl Event<C, R, B>,
    ) -> Vec<Box<dyn EventHandlerWithOwnerEmbedded<C, R, B>>> {
        let mut output_owned_event_handlers = Vec::new();

        // From the weather
        if let Some(weather) = &self.weather {
            if let Some(event_handler) = event.get_event_handler_without_receiver(weather.event_listener()) {
                let owned_event_handler = Box::new(EventHandlerWithOwner {
                    event_handler,
                    receiver_id: NOTHING,
                    mechanic_id: NOTHING,
                    activation_order: ActivationOrder {
                        priority: 0,
                        speed: 0,
                        order: 0,
                    },
                }) as Box<dyn EventHandlerWithOwnerEmbedded<C, R, B>>;
                output_owned_event_handlers.extend([owned_event_handler]);
            }
        }

        // From the terrain
        if let Some(terrain) = &self.terrain {
            if let Some(event_handler) = event.get_event_handler_without_receiver(terrain.event_listener()) {
                let owned_event_handler = Box::new(EventHandlerWithOwner {
                    event_handler,
                    receiver_id: NOTHING,
                    mechanic_id: NOTHING,
                    activation_order: ActivationOrder {
                        priority: 0,
                        speed: 0,
                        order: 0,
                    },
                }) as Box<dyn EventHandlerWithOwnerEmbedded<C, R, B>>;
                output_owned_event_handlers.extend([owned_event_handler]);
            }
        }
        output_owned_event_handlers
    }
}
