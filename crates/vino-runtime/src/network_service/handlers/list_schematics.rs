use crate::dev::prelude::*;

type Result<T> = std::result::Result<T, NetworkError>;

#[derive(Message)]
#[rtype(result = "Result<ProviderSignature>")]
pub(crate) struct GetSignature {}
