pub mod weather;
use monsim_utils::NOTHING;
pub use weather::*;

use crate::{
    sim::event_dispatcher::{Event, EventContext, OwnedEventHandler, OwnedEventHandlerWithoutReceiver},
    ActivationOrder, Broadcaster,
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
        event: &impl Event<R, C, B>,
    ) -> Vec<Box<dyn OwnedEventHandler<R, C, B>>> {
        let mut output_owned_event_handlers = Vec::new();
        if let Some(weather) = &self.weather {
            if let Some(event_handler) = event.get_event_handler_without_receiver(weather.event_handlers()) {
                let owned_event_handler = Box::new(OwnedEventHandlerWithoutReceiver {
                    event_handler,
                    mechanic_id: NOTHING,
                    activation_order: ActivationOrder {
                        priority: 0,
                        speed: 0,
                        order: 0,
                    },
                }) as Box<dyn OwnedEventHandler<R, C, B>>;
                output_owned_event_handlers.extend([owned_event_handler]);
            }
        }
        output_owned_event_handlers
    }
}
