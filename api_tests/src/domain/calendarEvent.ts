export type RRuleOptions = {
  freq: number;
  interval: number;
  count?: number;
  until?: number;
  tzid: string;
  wkst: number;
  bysetpos: number[];
  byweekday: number[];
  bynweekday: number[][];
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