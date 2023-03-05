use crate::error::AppError;
use coremidi::{
    Client, Destination, Destinations, EventList, InputPortWithContext,
    OutputPort, Protocol, Source, Sources,
};

macro_rules! endpoint_names {
    ( $iter:expr ) => {
        $iter.into_iter().enumerate().map(|(i, obj)| {
            (
                i,
                obj.display_name()
                    .unwrap_or("Could not get display name!".to_string()),
            )
        })
    };
}

macro_rules! diagnose_endpoints {
    ( $iter:expr ) => {
        for (i, display_name) in endpoint_names!($iter) {
            println!("[{}] {}", i, display_name);
        }
    };
}

pub fn diagnose_midi_devices() {
    println!("Destinations:");
    diagnose_endpoints!(Destinations);
    println!("Sources:");
    diagnose_endpoints!(Sources);
}

const PREFERRED_CONTROLLER_NAME: &str = "APC mini mk2 Control";

pub fn connect_to_controller<
    F: FnMut(&EventList, &mut u32) + Send + 'static,
>(
    callback: F,
) -> Result<(Client, InputPortWithContext<u32>, OutputPort), AppError> {
    match endpoint_names!(Sources)
        .find(|(_, name)| name == PREFERRED_CONTROLLER_NAME)
    {
        Some(name) => {
            let client = Client::new("grinstrument-client")
                .map_err(AppError::MidiClientError)?;
            let output_port = client
                .output_port("grinstrument-output-port")
                .map_err(AppError::MidiPortError)?;
            let input_port = client
                .input_port_with_protocol(
                    "grinstrument-input-port",
                    Protocol::Midi10,
                    callback,
                )
                .map_err(AppError::MidiPortError)?;
            Ok((client, input_port, output_port))
        }
        None => Err(AppError::NoControllerFound),
    }
}

pub fn get_destination(name: &str) -> Option<Destination> {
    Destinations
        .into_iter()
        .find(|dest| match dest.display_name() {
            Some(display_name) => display_name == name,
            None => false,
        })
}

pub fn get_source(name: &str) -> Option<Source> {
    Sources.into_iter().find(|x| match x.display_name() {
        Some(display_name) => display_name == name,
        None => false,
    })
}
