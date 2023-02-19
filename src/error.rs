#[derive(Debug)]
pub enum AppError {
    DestinationNotFoundError,
    DisplayNameError,
    MidiClientError(i32),
    MidiPortError(i32),
    NoControllerFound,
    SourceNotFoundError,
    SourceListenError(i32),
    SourceUniqueIdError,
}
