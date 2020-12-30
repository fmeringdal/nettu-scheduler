import { NettuBaseClient } from "./baseClient";
import { RRuleOptions } from "../domain/calendarEvent";

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

export type Timespan = {
  startTs: number;
  endTs: number;
};

export class NettuEventClient extends NettuBaseClient {
  public update(eventId: string, data: UpdateCalendarEventReq, auth: boolean) {
    return this.put(`/events/${eventId}`, data, auth);
  }

  public insert(data: CreateCalendarEventReq, auth: boolean) {
    return this.post("/events", data, auth);
  }

  public findById(eventId: string, auth: boolean) {
    return this.get(`/events/${eventId}`, auth);
  }

  public remove(eventId: string, auth: boolean) {
    return this.delete(`/events/${eventId}`, auth);
  }

  public getInstances(eventId: string, timespan: Timespan, auth: boolean) {
    return this.get(
      `/events/${eventId}/instances?startTs=${timespan.startTs}&endTs=${timespan.endTs}`,
      auth
    );
  }
}

export const nettuEventClient = new NettuEventClient();