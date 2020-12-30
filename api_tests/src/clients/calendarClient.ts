import { Calendar } from "../domain/calendar";
import { NettuBaseClient } from "./baseClient";

export class NettuCalendarClient extends NettuBaseClient {
  // data will probably be something in the future
  public insert(data: undefined, auth: boolean) {
    return this.post("/calendar", data, auth);
  }

  public findById(calendarId: string, auth: boolean) {
    return this.get<Calendar>(`/calendar/${calendarId}`, auth);
  }

  public remove(calendarId: string, auth: boolean) {
    return this.delete(`/calendar/${calendarId}`, auth);
  }
}

export const nettuCalendarClient = new NettuCalendarClient();
