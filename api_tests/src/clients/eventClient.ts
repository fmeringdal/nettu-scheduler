import { NettuBaseClient } from "./baseClient";
import {
  CalendarEvent,
  CalendarEventInstance,
  RRuleOptions,
} from "../domain/calendarEvent";

export type CreateCalendarEventReq = {
  calendarId: string;
  startTs: number;
  duration: number;
  busy?: boolean;
  rruleOptions?: RRuleOptions;
};

export type UpdateCalendarEventReq = {
  startTs?: number;
  duration?: number;
  busy?: boolean;
  rruleOptions?: RRuleOptions;
};

export type CreateCalendarEventExceptionReq = {
  exceptionTs: number;
};

export type Timespan = {
  startTs: number;
  endTs: number;
};

export class NettuEventClient extends NettuBaseClient {
  public update(eventId: string, data: UpdateCalendarEventReq) {
    return this.put<any>(`/events/${eventId}`, data);
  }

  public insert(data: CreateCalendarEventReq) {
    return this.post<any>("/events", data);
  }

  public createException(
    eventId: string,
    data: CreateCalendarEventExceptionReq
  ) {
    return this.post<any>(`/events/${eventId}/exception`, data);
  }

  public findById(eventId: string) {
    return this.get<CalendarEvent>(`/events/${eventId}`);
  }

  public remove(eventId: string) {
    return this.delete<any>(`/events/${eventId}`);
  }

  public getInstances(eventId: string, timespan: Timespan) {
    return this.get<{ instances: CalendarEventInstance[] }>(
      `/events/${eventId}/instances?startTs=${timespan.startTs}&endTs=${timespan.endTs}`
    );
  }
}
