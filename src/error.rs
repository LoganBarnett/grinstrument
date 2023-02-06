
#[derive(Debug)]
pub enum AppError {
    DestinationDisplayNameError,
    SourceDisplayNameError,
    SourceNotFoundError,
    SourceListenError(i32),
    SourceUniqueIdError,
}
