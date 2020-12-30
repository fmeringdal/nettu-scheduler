import { nettuCalendarClient } from "./calendarClient";
import { nettuEventClient } from "./eventClient";
import { nettuHealthClient } from "./healthClient";
import { nettuUserClient } from "./userClient";

export const Client = {
  events: nettuEventClient,
  calendar: nettuCalendarClient,
  user: nettuUserClient,
  health: nettuHealthClient,
};
