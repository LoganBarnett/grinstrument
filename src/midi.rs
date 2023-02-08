use crate::{enclose, error::AppError};
use coremidi::{
    Client, Destination, Destinations, Endpoint, EventBuffer, EventList,
    InputPort, OutputPort, Protocol, Source, Sources,
};

pub fn endpoint_names<A: Endpoint>(iter: Iterator<A>) {
    iter.map(|(i, endpoint)| {
        (
            i,
            endpoint
                .display_name
                .unwrap_or("Could not get display name!"),
        )
    })
}

pub fn diagnose_endpoints<A: Object>(iter: Iterator<A>) {
    for (i, display_name) in endpoint_names(iter) {
        println!("[{}] {}", i, display_name);
    }
}

pub fn diagnose_midi_devices() {
    println!("Destinations:");
    diagnose_endpoints(Destinations.into_iter());
    println!("Sources:");
    diagnose_endpoints(Sources.into_iter());
}

const PREFERRED_CONTROLLER_NAME: &str = "APC mini mk2 Control";

pub fn connect_to_controller(
) -> Result<(Client, InputPort, OutputPort), AppError> {
    match endpoint_names(Sources).find(|name| name == PREFERRED_CONTROLLER_NAME)
    {
        Some(name) => {
            let client = Client::new("grinstrument-client")
                .map_err(AppError::MidiClientError)?;
            let output_port = client
                .output_port("grinstrument-output-port")
                .map_err(AppError::MidiPortError)?;
            let callback = enclose!((global_state) move |event_list: &EventList, context: &mut u32| {

            });
            let input_port = client.input_port_with_protocol(
                "grinstrument-input-port",
                Protocol::Midi10,
                callback,
            );
        }
        None => AppError::NoControllerFound,
    }
}
