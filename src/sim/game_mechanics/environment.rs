pub mod weather;
pub use weather::*;

use crate::{sim::event_dispatcher::EventContext, ActivationOrder, ActorID, Broadcaster, EventHandlerSelector, OwnedEventHandler};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Environment {
    pub weather: Option<Weather>,
}

impl Environment {
    pub fn weather(&self) -> Option<&Weather> {
        self.weather.as_ref()
    }

    pub fn weather_mut(&mut self) -> Option<&mut Weather> {
        self.weather.as_mut()
    }

    pub(crate) fn owned_event_handlers<R: Copy, C: EventContext + Copy, B: Broadcaster + Copy>(
        &self,
        event_handler_selector: EventHandlerSelector<R, C, B>,
    ) -> Vec<OwnedEventHandler<R, C, B>> {
        let mut output_owned_event_handlers = vec![];
        if let Some(weather) = &self.weather {
            event_handler_selector(weather.event_handlers())
                .into_iter()
                .flatten()
                .for_each(|event_handler| {
                    let owned_event_handler = OwnedEventHandler {
                        event_handler,
                        owner_id: ActorID::Environment,
                        activation_order: ActivationOrder {
                            priority: 0,
                            speed: 0,
                            order: 0,
                        },
                    };
                    output_owned_event_handlers.extend([owned_event_handler].into_iter());
                });
        }
        output_owned_event_handlers
    }
}
