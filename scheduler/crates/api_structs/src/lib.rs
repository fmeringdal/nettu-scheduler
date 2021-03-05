mod account;
mod calendar;
mod event;
mod schedule;
mod service;
mod status;
mod user;

pub mod dtos {
    pub(crate) use crate::account::dtos::*;
    pub(crate) use crate::calendar::dtos::*;
    pub(crate) use crate::event::dtos::*;
    pub(crate) use crate::schedule::dtos::*;
    pub(crate) use crate::service::dtos::*;
    pub(crate) use crate::user::dtos::*;
}

pub use crate::account::api::*;
pub use crate::calendar::api::*;
pub use crate::event::api::*;
pub use crate::schedule::api::*;
pub use crate::service::api::*;
pub use crate::status::api::*;
pub use crate::user::api::*;
