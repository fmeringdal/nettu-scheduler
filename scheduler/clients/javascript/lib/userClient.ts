import { CalendarEventInstance } from "./domain/calendarEvent";
import { NettuBaseClient } from "./baseClient";
import { Metadata } from "./domain/metadata";
import { User } from "./domain/user";
import { IntegrationProvider } from ".";

type GetUserFeebusyReq = {
  startTs: number;
  endTs: number;
  calendarIds?: string[];
};

type GetUserFeebusyResponse = {
  busy: CalendarEventInstance[];
};

type UpdateUserRequest = {
  metadata?: Metadata;
};

type CreateUserRequest = {
  metadata?: Metadata;
};

type UserResponse = {
  user: User;
};

export class NettuUserClient extends NettuBaseClient {
  public create(data?: CreateUserRequest) {
    data = data ? data : {};
    return this.post<UserResponse>(`/user`, data);
  }

  public find(userId: string) {
    return this.get<UserResponse>(`/user/${userId}`);
  }

  public update(userId: string, data: UpdateUserRequest) {
    return this.put<UserResponse>(`/user/${userId}`, data);
  }

  public findByMeta(
    meta: {
      key: string;
      value: string;
    },
    skip: number,
    limit: number
  ) {
    return this.get<User[]>(
      `/user/meta?skip=${skip}&limit=${limit}&key=${meta.key}&value=${meta.value}`
    );
  }

  public remove(userId: string) {
    return this.delete<UserResponse>(`/user/${userId}`);
  }

  public freebusy(userId: string, req: GetUserFeebusyReq) {
    let queryString = `startTs=${req.startTs}&endTs=${req.endTs}`;
    if (req.calendarIds && req.calendarIds.length > 0) {
      queryString += `&calendarIds=${req.calendarIds.join(",")}`;
    }
    return this.get<GetUserFeebusyResponse>(
      `/user/${userId}/freebusy?${queryString}`
    );
  }

  public oauth(userId: string, code: string, provider: IntegrationProvider) {
    const body = { code, provider };
    return this.post(`user/${userId}/oauth`, body);
  }

  public removeIntegration(userId: string, provider: IntegrationProvider) {
    return this.delete(`user/${userId}/oauth/${provider}`);
  }
}

export class NettuUserUserClient extends NettuBaseClient {
  public me() {
    return this.get<UserResponse>(`/me`);
  }
}
