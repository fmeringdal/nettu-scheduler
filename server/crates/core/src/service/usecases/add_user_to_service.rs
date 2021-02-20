use crate::context::Context;
use crate::{
    account::domain::Account,
    service::domain::{Service, ServiceResource},
    shared::usecase::UseCase,
};

struct AddUserToServiceUseCase {
    pub account: Account,
    pub service_id: String,
    pub user_id: String,
    calendar_ids: Option<Vec<String>>,
    schedule_ids: Option<Vec<String>>,
}

struct UseCaseRes {
    pub service: Service,
}

#[derive(Debug)]
enum UseCaseErrors {
    StorageError,
    ServiceNotFoundError,
    UserNotFoundError,
    UserAlreadyInService,
    CalendarNotOwnedByUser(String),
}

#[async_trait::async_trait(?Send)]
impl UseCase for AddUserToServiceUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let _user = match ctx.repos.user_repo.find(&self.user_id).await {
            Some(user) if user.account_id == self.account.id => user,
            _ => return Err(UseCaseErrors::UserNotFoundError),
        };

        let mut service = match ctx.repos.service_repo.find(&self.service_id).await {
            Some(service) if service.account_id == self.account.id => service,
            _ => return Err(UseCaseErrors::ServiceNotFoundError),
        };

        if let Some(_user_resource) = service.find_user(&self.user_id) {
            return Err(UseCaseErrors::UserAlreadyInService);
        }

        let calendar_ids = match &self.calendar_ids {
            Some(calendar_ids) => {
                let user_calendars = ctx
                    .repos
                    .calendar_repo
                    .find_by_user(&self.user_id)
                    .await
                    .into_iter()
                    .map(|cal| cal.id)
                    .collect::<Vec<_>>();
                for calendar_id in calendar_ids {
                    if !user_calendars.contains(calendar_id) {
                        return Err(UseCaseErrors::CalendarNotOwnedByUser(calendar_id.clone()));
                    }
                }
                Some(calendar_ids)
            }
            None => None,
        };

        let schedule_ids = match &self.schedule_ids {
            Some(schedule_ids) => {
                let user_schedules = ctx
                    .repos
                    .schedule_repo
                    .find_by_user(&self.user_id)
                    .await
                    .into_iter()
                    .map(|schedule| schedule.id)
                    .collect::<Vec<_>>();
                for schedule_id in schedule_ids {
                    if !user_schedules.contains(schedule_id) {
                        return Err(UseCaseErrors::CalendarNotOwnedByUser(schedule_id.clone()));
                    }
                }
                Some(schedule_ids)
            }
            None => None,
        };

        let user_resource = ServiceResource::new(
            &self.user_id,
            &calendar_ids.unwrap_or(&vec![]),
            &schedule_ids.unwrap_or(&vec![]),
        );
        service.add_user(user_resource);

        let res = ctx.repos.service_repo.save(&service).await;
        match res {
            Ok(_) => Ok(UseCaseRes { service }),
            Err(_) => Err(UseCaseErrors::StorageError),
        }
    }
}
