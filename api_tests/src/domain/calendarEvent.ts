export enum Frequenzy {
  Daily = "daily",
  Weekly = "weekly",
  Monthly = "monthly",
  Yearly = "yearly",
}

export type RRuleOptions = {
  freq: Frequenzy;
  interval: number;
  count?: number;
  until?: number;
  tzid: string;
  wkst: number;
  bysetpos?: number[];
  byweekday?: number[];
  bynweekday?: number[][];
};

export type CalendarEvent = {
  id: string;
  startTs: number;
  duration: number;
  busy: boolean;
  endTs?: number;
  recurrence?: RRuleOptions;
  exdates: number[];
  calendarId: string;
  userId: string;
};

export type CalendarEventInstance = {
  startTs: number;
  endTs: number;
  busy: boolean;
};
