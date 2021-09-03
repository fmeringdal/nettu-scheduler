import {
  Calendar,
  GoogleCalendarAccessRole,
  GoogleCalendarListEntry,
  OutlookCalendar,
  OutlookCalendarAccessRole,
} from "./domain/calendar";
import { NettuBaseClient } from "./baseClient";
import { Metadata } from "./domain/metadata";
import {
  CalendarEvent,
  CalendarEventInstance,
  IntegrationProvider,
} from "./domain";
import { Timespan } from "./eventClient";

type CreateCalendarRequest = {
  timezone: string;
  weekStart?: number;
  metadata?: Metadata;
};

type UpdateCalendarRequest = CreateCalendarRequest;

type GetCalendarEventsResponse = {
  calendar: Calendar;
  events: {
    event: CalendarEvent;
    instances: CalendarEventInstance[];
  }[];
};

type CalendarResponse = {
  calendar: Calendar;
};

type SyncCalendarInput = {
  userId: string;
  calendarId: string;
  extCalendarId: string;
  provider: IntegrationProvider;
};

type StopCalendarSyncInput = {
  userId: string;
  calendarId: string;
  extCalendarId: string;
  provider: IntegrationProvider;
};

export class NettuCalendarClient extends NettuBaseClient {
  public create(userId: string, data: CreateCalendarRequest) {
    return this.post<CalendarResponse>(`/user/${userId}/calendar`, data);
  }

  public findById(calendarId: string) {
    return this.get<CalendarResponse>(`/user/calendar/${calendarId}`);
  }

  public findByMeta(
    meta: {
      key: string;
      value: string;
    },
    skip: number,
    limit: number
  ) {
    return this.get<{ calendars: Calendar[] }>(
      `/calendar/meta?skip=${skip}&limit=${limit}&key=${meta.key}&value=${meta.value}`
    );
  }

  async findGoogle(userId: string, minAccessRole: GoogleCalendarAccessRole) {
    return this.get<{ calendars: GoogleCalendarListEntry[] }>(
      `/user/${userId}/calendar/provider/google?minAccessRole=${minAccessRole}`
    );
  }

  async findOutlook(userId: string, minAccessRole: OutlookCalendarAccessRole) {
    return this.get<{ calendars: OutlookCalendar[] }>(
      `/user/${userId}/calendar/provider/outlook?minAccessRole=${minAccessRole}`
    );
  }

  public remove(calendarId: string) {
    return this.delete<CalendarResponse>(`/user/calendar/${calendarId}`);
  }

  public update(calendarId: string, data: UpdateCalendarRequest) {
    return this.put<CalendarResponse>(`/user/calendar/${calendarId}`, {
      settings: {
        timezone: data.timezone,
        weekStart: data.weekStart,
      },
      metadata: data.metadata,
    });
  }

  public getEvents(calendarId: string, startTS: number, endTS: number) {
    return this.get<GetCalendarEventsResponse>(
      `/user/calendar/${calendarId}/events?startTs=${startTS}&endTs=${endTS}`
    );
  }

  public syncCalendar(input: SyncCalendarInput) {
    const body = {
      calendarId: input.calendarId,
      extCalendarId: input.extCalendarId,
      provider: input.provider,
    };
    return this.put(`user/${input.userId}/calendar/sync`, body);
  }

  public stopCalendarSync(input: StopCalendarSyncInput) {
    const body = {
      calendarId: input.calendarId,
      extCalendarId: input.extCalendarId,
      provider: input.provider,
    };
    return this.deleteWithBody(`user/${input.userId}/calendar/sync`, body);
  }
}

export class NettuCalendarUserClient extends NettuBaseClient {
  public create(data: CreateCalendarRequest) {
    return this.post<CalendarResponse>("/calendar", data);
  }

  public findById(calendarId: string) {
    return this.get<CalendarResponse>(`/calendar/${calendarId}`);
  }

  async findGoogle(minAccessRole: GoogleCalendarAccessRole) {
    return this.get<{ calendars: GoogleCalendarListEntry[] }>(
      `/calendar/provider/google?minAccessRole=${minAccessRole}`
    );
  }

  async findOutlook(minAccessRole: OutlookCalendarAccessRole) {
    return this.get<{ calendars: OutlookCalendar[] }>(
      `/calendar/provider/outlook?minAccessRole=${minAccessRole}`
    );
  }

  public remove(calendarId: string) {
    return this.delete<CalendarResponse>(`/calendar/${calendarId}`);
  }

  public update(calendarId: string, data: UpdateCalendarRequest) {
    return this.put<CalendarResponse>(`/calendar/${calendarId}`, data);
  }

  public getEvents(calendarId: string, timespan: Timespan) {
    return this.get<GetCalendarEventsResponse>(
      `/user/calendar/${calendarId}/events?startTs=${timespan.startTs}&endTs=${timespan.endTs}`
    );
  }
}
