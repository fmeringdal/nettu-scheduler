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
  public freebusy(userId: string, req: GetUserFeebusyReq, auth: boolean) {
    let queryString = `startTs=${req.startTs}&endTs=${req.endTs}`;
    if (req.calendarIds && req.calendarIds.length > 0) {
      queryString += `&calendarIds=${req.calendarIds.join(",")}`;
    }
    return this.get<GetUserFeebusyResponse>(
      `/user/${userId}/freebusy?${queryString}`,
      auth
    );
  }

  public bookingslots(
    userId: string,
    req: GetUserBookingslotsReq,
    auth: boolean
  ) {
    let queryString = `date=${req.date}&ianaTz=${req.ianaTz}&duration=${req.duration}`;
    if (req.calendarIds && req.calendarIds.length > 0) {
      queryString += `&calendarIds=${req.calendarIds.join(",")}`;
    }
    return this.get<GetUserBookingslotsResponse>(
      `/user/${userId}/booking?${queryString}`,
      auth
    );
  }
}

export const nettuUserClient = new NettuUserClient();
