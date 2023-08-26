use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("iRacing is down for maintenance")]
    DownForMaintenance,
    #[error("got invalid api response data")]
    UnknownApiResponseError,
}
