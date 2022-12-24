#[derive(Copy, Clone, Debug)]
pub enum Error {
    BadSchema,
    NoMedia,
    NotFound,
    InsufficentResource,
    EndOfFile,
    Generic,
}
