import { NettuBaseClient } from "./baseClient";
import { Schedule, ScheduleRule } from "./domain/schedule";

interface UpdateScheduleRequest {
  rules?: ScheduleRule[];
  timezone?: string;
}

interface CreateScheduleRequest {
  timezone: string;
  rules?: ScheduleRule[];
}

type ScheduleResponse = {
  schedule: Schedule;
}

export class NettuScheduleClient extends NettuBaseClient {
  public async create(userId: string, req: CreateScheduleRequest) {
    return await this.post<ScheduleResponse>(`/user/${userId}/schedule`, req);
  }

  public async update(scheduleId: string, update: UpdateScheduleRequest) {
    return await this.put<ScheduleResponse>(`/user/schedule/${scheduleId}`, update);
  }

  public async remove(scheduleId: string) {
    return await this.delete<ScheduleResponse>(`/user/schedule/${scheduleId}`);
  }

  public async find(scheduleId: string) {
    return await this.get<ScheduleResponse>(`/user/schedule/${scheduleId}`);
  }
}

export class NettuScheduleUserClient extends NettuBaseClient {
  public async create(req: CreateScheduleRequest) {
    return await this.post<ScheduleResponse>(`/schedule`, req);
  }

  public async update(scheduleId: string, update: UpdateScheduleRequest) {
    return await this.put<ScheduleResponse>(`/schedule/${scheduleId}`, update);
  }

  public async remove(scheduleId: string) {
    return await this.delete<ScheduleResponse>(`/schedule/${scheduleId}`);
  }

  public async find(scheduleId: string) {
    return await this.get<ScheduleResponse>(`/schedule/${scheduleId}`);
  }
}