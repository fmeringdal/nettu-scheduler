import { Metadata } from "./metadata";

export interface Calendar {
  id: string;
  userId: string;
  settings: {
    weekStart: string;
    timezone: string;
  };
  metadata: Metadata;
}

export enum GoogleCalendarAccessRole {
  Owner = "owner",
  Writer = "writer",
  Reader = "reader",
  FreeBusyReader = "freeBusyReader",
}

export interface GoogleCalendarListEntry {
  id: string;
  access_role: GoogleCalendarAccessRole;
  summary: string;
  summaryOverride?: string;
  description?: string;
  location?: string;
  timeZone?: string;
  colorId?: string;
  backgroundColor?: string;
  foregroundColor?: string;
  hidden?: boolean;
  selected?: boolean;
  primary?: boolean;
  deleted?: boolean;
}

export enum OutlookCalendarAccessRole {
  Writer = "writer",
  Reader = "reader",
}

export interface OutlookCalendar {
  id: string;
  name: string;
  color: string;
  changeKey: string;
  canShare: boolean;
  canViewPrivateItems: boolean;
  hexColor: string;
  canEdit: boolean;
}
