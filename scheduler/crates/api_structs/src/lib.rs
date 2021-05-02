mod account;
mod calendar;
mod event;
mod schedule;
mod service;
mod status;
mod user;

pub mod dtos {
    pub use crate::account::dtos::*;
    pub use crate::calendar::dtos::*;
    pub use crate::event::dtos::*;
    pub use crate::schedule::dtos::*;
    pub use crate::service::dtos::*;
    pub use crate::user::dtos::*;
}

pub use crate::account::api::*;
pub use crate::calendar::api::*;
pub use crate::event::api::*;
pub use crate::schedule::api::*;
pub use crate::service::api::*;
pub use crate::status::api::*;
pub use crate::user::api::*;
