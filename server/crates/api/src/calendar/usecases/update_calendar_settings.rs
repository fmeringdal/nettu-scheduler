use crate::{
    calendar::dtos::CalendarDTO,
    shared::{
        auth::Permission,
        usecase::{execute_with_policy, PermissionBoundary, UseCase, UseCaseErrorContainer},
    },
};
use crate::{error::NettuError, shared::auth::protect_route};
use actix_web::{web, HttpResponse};
use nettu_scheduler_api_structs::api::update_calendar_settings::{PathParams, RequestBody};
use nettu_scheduler_core::Calendar;
use nettu_scheduler_infra::NettuContext;

pub async fn update_calendar_settings_controller(
    http_req: web::HttpRequest,
    ctx: web::Data<NettuContext>,
    path_params: web::Path<PathParams>,
    body: web::Json<RequestBody>,
) -> Result<HttpResponse, NettuError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;

    let usecase = UpdateCalendarSettingsUseCase {
        user_id: user.id,
        calendar_id: path_params.calendar_id.clone(),
        wkst: body.wkst.clone(),
        timezone: body.timezone.clone(),
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|calendar| HttpResponse::Ok().json(CalendarDTO::new(&calendar)))
        .map_err(|e| match e {
            UseCaseErrorContainer::Unauthorized(e) => NettuError::Unauthorized(e),
            UseCaseErrorContainer::UseCase(e) => match e {
                UseCaseErrors::StorageError => NettuError::InternalError,
                UseCaseErrors::CalendarNotFoundError => {
                    NettuError::NotFound("The calendar was not found.".into())
                }
                UseCaseErrors::InvalidSettings(err) => NettuError::BadClientData(format!(
                    "Bad calendar settings provided. Error message: {}",
                    err
                )),
            },
        })
}

#[derive(Debug)]
struct UpdateCalendarSettingsUseCase {
    pub user_id: String,
    pub calendar_id: String,
    pub wkst: Option<isize>,
    pub timezone: Option<String>,
}

#[derive(Debug)]
enum UseCaseErrors {
    CalendarNotFoundError,
    StorageError,
    InvalidSettings(String),
}

#[async_trait::async_trait(?Send)]
impl UseCase for UpdateCalendarSettingsUseCase {
    type Response = Calendar;

    type Errors = UseCaseErrors;

    type Context = NettuContext;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let mut calendar = match ctx.repos.calendar_repo.find(&self.calendar_id).await {
            Some(cal) if cal.user_id == self.user_id => cal,
            _ => return Err(UseCaseErrors::CalendarNotFoundError),
        };

        if let Some(wkst) = self.wkst {
            if !calendar.settings.set_wkst(wkst) {
                return Err(UseCaseErrors::InvalidSettings(format!(
                    "Invalid wkst property: {}, must be between 0 and 6",
                    wkst
                )));
            }
        }

        if let Some(timezone) = &self.timezone {
            if !calendar.settings.set_timezone(timezone) {
                return Err(UseCaseErrors::InvalidSettings(format!(
                    "Invalid timezone property: {}, must be a valid IANA Timezone string",
                    timezone
                )));
            }
        }

        let repo_res = ctx.repos.calendar_repo.save(&calendar).await;
        match repo_res {
            Ok(_) => Ok(calendar),
            Err(_) => Err(UseCaseErrors::StorageError),
        }
    }
}

impl PermissionBoundary for UpdateCalendarSettingsUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::UpdateCalendar]
    }
}

#[cfg(test)]
mod test {
    use nettu_scheduler_core::Calendar;
    use nettu_scheduler_infra::setup_context;

    use super::*;

    #[actix_web::main]
    #[test]
    async fn it_rejects_invalid_wkst() {
        let ctx = setup_context().await;
        let user_id = "1".to_string();
        let calendar = Calendar::new(&user_id);
        ctx.repos.calendar_repo.insert(&calendar).await.unwrap();

        let mut usecase = UpdateCalendarSettingsUseCase {
            calendar_id: calendar.id.into(),
            user_id: user_id.into(),
            wkst: Some(20),
            timezone: None,
        };
        let res = usecase.execute(&ctx).await;
        assert!(res.is_err());
    }

    #[actix_web::main]
    #[test]
    async fn it_update_settings_with_valid_wkst() {
        let ctx = setup_context().await;
        let user_id = "1".to_string();
        let calendar = Calendar::new(&user_id);
        ctx.repos.calendar_repo.insert(&calendar).await.unwrap();

        assert_eq!(calendar.settings.wkst, 0);
        let new_wkst = 3;
        let mut usecase = UpdateCalendarSettingsUseCase {
            calendar_id: calendar.id.clone(),
            user_id: user_id.into(),
            wkst: Some(new_wkst),
            timezone: None,
        };
        let res = usecase.execute(&ctx).await;
        assert!(res.is_ok());

        // Check that calendar settings have been updated
        let calendar = ctx.repos.calendar_repo.find(&calendar.id).await.unwrap();
        assert_eq!(calendar.settings.wkst, new_wkst);
    }
}
