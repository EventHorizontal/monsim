pub mod weather;
use monsim_utils::{Nothing, NOTHING};
pub use weather::*;

use crate::{
    sim::event_dispatcher::{EventContext, OwnedEventHandlerT},
    ActivationOrder, Broadcaster, EventHandlerSelector, OwnedEventHandler,
};

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

    pub(crate) fn owned_event_handlers<R: Copy + 'static, C: EventContext + Copy + 'static, B: Broadcaster + Copy + 'static>(
        &self,
        event_handler_selector: EventHandlerSelector<R, C, Nothing, B>,
    ) -> Vec<Box<dyn OwnedEventHandlerT<R, C, B>>> {
        let mut output_owned_event_handlers = Vec::new();
        if let Some(weather) = &self.weather {
            event_handler_selector(weather.event_handlers())
                .into_iter()
                .flatten()
                .for_each(|event_handler| {
                    let owned_event_handler = Box::new(OwnedEventHandler {
                        event_handler,
                        owner_id: NOTHING,
                        activation_order: ActivationOrder {
                            priority: 0,
                            speed: 0,
                            order: 0,
                        },
                    }) as Box<dyn OwnedEventHandlerT<R, C, B>>;
                    output_owned_event_handlers.extend([owned_event_handler].into_iter());
                });
        }
        output_owned_event_handlers
    }
}
