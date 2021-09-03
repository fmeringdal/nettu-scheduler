import { NettuBaseClient } from "./baseClient";
import {
  CalendarEvent,
  CalendarEventInstance,
  RRuleOptions,
} from "./domain/calendarEvent";
import { Metadata } from "./domain/metadata";

interface EventReminder {
  delta: number;
  identifier: string;
}

type CreateCalendarEventReq = {
  calendarId: string;
  startTs: number;
  duration: number;
  busy?: boolean;
  recurrence?: RRuleOptions;
  serviceId?: boolean;
  reminders?: EventReminder[];
  metadata?: Metadata;
};

type UpdateCalendarEventReq = {
  startTs?: number;
  duration?: number;
  busy?: boolean;
  recurrence?: RRuleOptions;
  serviceId?: boolean;
  exdates?: number[];
  reminders?: EventReminder[];
  metadata?: Metadata;
};

export type Timespan = {
  startTs: number;
  endTs: number;
};

type GetEventInstancesResponse = {
  instances: CalendarEventInstance[];
};

type EventReponse = {
  event: CalendarEvent;
};

export class NettuEventClient extends NettuBaseClient {
  public update(eventId: string, data: UpdateCalendarEventReq) {
    return this.put<EventReponse>(`/user/events/${eventId}`, data);
  }

  public create(userId: string, data: CreateCalendarEventReq) {
    return this.post<EventReponse>(`/user/${userId}/events`, data);
  }

  public findById(eventId: string) {
    return this.get<EventReponse>(`/user/events/${eventId}`);
  }

  public findByMeta(
    meta: {
      key: string;
      value: string;
    },
    skip: number,
    limit: number
  ) {
    return this.get<{ events: CalendarEvent[] }>(
      `/events/meta?skip=${skip}&limit=${limit}&key=${meta.key}&value=${meta.value}`
    );
  }

  public remove(eventId: string) {
    return this.delete<EventReponse>(`/user/events/${eventId}`);
  }

  public getInstances(eventId: string, timespan: Timespan) {
    return this.get<GetEventInstancesResponse>(
      `/user/events/${eventId}/instances?startTs=${timespan.startTs}&endTs=${timespan.endTs}`
    );
  }
}

export class NettuEventUserClient extends NettuBaseClient {
  public update(eventId: string, data: UpdateCalendarEventReq) {
    return this.put<EventReponse>(`/events/${eventId}`, data);
  }

  public create(data: CreateCalendarEventReq) {
    return this.post<EventReponse>("/events", data);
  }

  public findById(eventId: string) {
    return this.get<EventReponse>(`/events/${eventId}`);
  }

  public remove(eventId: string) {
    return this.delete<EventReponse>(`/events/${eventId}`);
  }

  public getInstances(eventId: string, timespan: Timespan) {
    return this.get<GetEventInstancesResponse>(
      `/events/${eventId}/instances?startTs=${timespan.startTs}&endTs=${timespan.endTs}`
    );
  }
}
