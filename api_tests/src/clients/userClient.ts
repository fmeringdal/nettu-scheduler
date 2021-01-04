import { CalendarEvent, CalendarEventInstance } from "../domain/calendarEvent";
import { NettuBaseClient } from "./baseClient";

export type GetUserFeebusyReq = {
  startTs: number;
  endTs: number;
  calendarIds?: string[];
};

export type GetUserBookingslotsReq = {
  ianaTz: string;
  duration: number;
  interval: number;
  date: string;
  calendarIds?: string[];
};

export type GetUserFeebusyResponse = {
  free: CalendarEventInstance[];
};

export type BookingSlot = {
  start: number;
  duration: number;
  available_until: number;
};

export type GetUserBookingslotsResponse = {
  bookingSlots: BookingSlot[];
};

export class NettuUserClient extends NettuBaseClient {
  public freebusy(userId: string, req: GetUserFeebusyReq) {
    let queryString = `startTs=${req.startTs}&endTs=${req.endTs}`;
    if (req.calendarIds && req.calendarIds.length > 0) {
      queryString += `&calendarIds=${req.calendarIds.join(",")}`;
    }
    return this.get<GetUserFeebusyResponse>(
      `/user/${userId}/freebusy?${queryString}`
    );
  }

  public bookingslots(userId: string, req: GetUserBookingslotsReq) {
    let queryString = `date=${req.date}&ianaTz=${req.ianaTz}&duration=${req.duration}&interval=${req.interval}`;
    if (req.calendarIds && req.calendarIds.length > 0) {
      queryString += `&calendarIds=${req.calendarIds.join(",")}`;
    }
    return this.get<GetUserBookingslotsResponse>(
      `/user/${userId}/booking?${queryString}`
    );
  }
}
