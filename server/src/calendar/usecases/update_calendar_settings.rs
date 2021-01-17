use crate::shared::usecase::{execute, Usecase};
use crate::{
    api::{Context, NettuError},
    shared::auth::protect_route,
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct UpdateCalendarSettigsPathParams {
    calendar_id: String,
}

#[derive(Deserialize)]
pub struct UpdateCalendarSettingsBody {
    wkst: isize,
}

pub async fn update_calendar_settings_controller(
    http_req: web::HttpRequest,
    ctx: web::Data<Context>,
    path_params: web::Path<UpdateCalendarSettigsPathParams>,
    body: web::Json<UpdateCalendarSettingsBody>,
) -> Result<HttpResponse, NettuError> {
    let user = protect_route(&http_req, &ctx).await?;

    let usecase = UpdateCalendarSettingsUseCase {
        user_id: user.id,
        calendar_id: path_params.calendar_id.clone(),
        wkst: body.wkst,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| HttpResponse::Created().json(usecase_res))
        .map_err(|e| match e {
            UseCaseErrors::StorageError => NettuError::InternalError,
            UseCaseErrors::CalendarNotFoundError => {
                NettuError::NotFound("The calendar was not found.".into())
            }
            UseCaseErrors::InvalidSettings(err) => NettuError::BadClientData(format!(
                "Bad calendar settings provided. Error message: {}",
                err
            )),
        })
}

struct UpdateCalendarSettingsUseCase {
    pub user_id: String,
    pub calendar_id: String,
    pub wkst: isize,
}

#[derive(Debug)]
enum UseCaseErrors {
    CalendarNotFoundError,
    StorageError,
    InvalidSettings(String),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UseCaseRes {}

#[async_trait::async_trait(?Send)]
impl Usecase for UpdateCalendarSettingsUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let mut calendar = match ctx.repos.calendar_repo.find(&self.calendar_id).await {
            Some(cal) if cal.user_id == self.user_id => cal,
            _ => return Err(UseCaseErrors::CalendarNotFoundError),
        };

        if !calendar.settings.set_wkst(self.wkst) {
            return Err(UseCaseErrors::InvalidSettings(format!(
                "Invalid wkst property: {}, must be between 0 and 6",
                self.wkst
            )));
        }

        let repo_res = ctx
            .repos
            .event_repo
            .update_calendar_wkst(&calendar.id, self.wkst as i32)
            .await;
        if repo_res.is_err() {
            return Err(UseCaseErrors::StorageError);
        }

        let repo_res = ctx.repos.calendar_repo.save(&calendar).await;
        match repo_res {
            Ok(_) => Ok(UseCaseRes {}),
            Err(_) => Err(UseCaseErrors::StorageError),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        calendar::domain::Calendar,
        event::domain::event::{CalendarEvent, RRuleOptions},
    };

    use super::*;

    #[actix_web::main]
    #[test]
    async fn it_rejects_invalid_wkst() {
        let ctx = Context::create_inmemory();
        let user_id = "1".to_string();
        let calendar = Calendar::new(&user_id);
        ctx.repos.calendar_repo.insert(&calendar).await.unwrap();

        let mut usecase = UpdateCalendarSettingsUseCase {
            calendar_id: calendar.id.into(),
            user_id: user_id.into(),
            wkst: 20,
        };
        let res = usecase.execute(&ctx).await;
        assert!(res.is_err());
    }

    #[actix_web::main]
    #[test]
    async fn it_update_settings_with_valid_wkst() {
        let ctx = Context::create_inmemory();
        let user_id = "1".to_string();
        let calendar = Calendar::new(&user_id);
        ctx.repos.calendar_repo.insert(&calendar).await.unwrap();

        let recurring_event = CalendarEvent {
            account_id: "0".into(),
            busy: false,
            calendar_id: calendar.id.clone(),
            duration: 60,
            end_ts: 100,
            exdates: vec![],
            id: "1".into(),
            recurrence: Some(Default::default()),
            reminder: None,
            start_ts: 10,
            user_id: user_id.clone(),
        };
        let recurring_event_in_other_calendar = CalendarEvent {
            account_id: "0".into(),
            busy: false,
            calendar_id: calendar.id.clone() + "9",
            duration: 60,
            end_ts: 100,
            exdates: vec![],
            id: "3".into(),
            recurrence: Some(Default::default()),
            reminder: None,
            start_ts: 10,
            user_id: user_id.clone(),
        };
        let non_recurring_event = CalendarEvent {
            account_id: "0".into(),
            busy: false,
            calendar_id: calendar.id.clone(),
            duration: 60,
            end_ts: 100,
            exdates: vec![],
            id: "2".into(),
            recurrence: None,
            reminder: None,
            start_ts: 10,
            user_id: user_id.clone(),
        };
        ctx.repos.event_repo.insert(&recurring_event).await.unwrap();
        ctx.repos
            .event_repo
            .insert(&recurring_event_in_other_calendar)
            .await
            .unwrap();
        ctx.repos
            .event_repo
            .insert(&non_recurring_event)
            .await
            .unwrap();

        assert_eq!(calendar.settings.wkst, 0);
        let new_wkst = 3;
        let mut usecase = UpdateCalendarSettingsUseCase {
            calendar_id: calendar.id.clone(),
            user_id: user_id.into(),
            wkst: new_wkst,
        };
        let res = usecase.execute(&ctx).await;
        assert!(res.is_ok());

        // Check that calendar settings have been updated
        let calendar = ctx.repos.calendar_repo.find(&calendar.id).await.unwrap();
        assert_eq!(calendar.settings.wkst, new_wkst);

        // Check that rrule options have been updated for the calendar event
        let e = ctx
            .repos
            .event_repo
            .find(&recurring_event.id)
            .await
            .unwrap();
        assert_eq!(e.recurrence.unwrap().wkst, new_wkst);

        // Check that rrule options have not been updated for event in other calendar
        let e = ctx
            .repos
            .event_repo
            .find(&recurring_event_in_other_calendar.id)
            .await
            .unwrap();
        assert_eq!(e.recurrence.unwrap().wkst, 0);

        // Check that non recurring event has not been updated
        let e = ctx
            .repos
            .event_repo
            .find(&non_recurring_event.id)
            .await
            .unwrap();
        assert!(e.recurrence.is_none());
    }
}
