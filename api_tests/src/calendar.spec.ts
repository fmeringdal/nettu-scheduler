import { nettuClient } from "./client";

describe("Calendar API", () => {
  it("should not create calendar for unauthenticated user", async () => {
    const res = await nettuClient.createCalendar(undefined, false);
    expect(res.status).toBe(401);
  });

  it("should create calendar for authenticated user", async () => {
    const res = await nettuClient.createCalendar(undefined, true);
    expect(res.status).toBe(200);
    expect(res.data.calendarId).toBeDefined();
  });

  it("should delete calendar for authenticated user and not for unauthenticated user", async () => {
    let res = await nettuClient.createCalendar(undefined, true);
    const { calendarId } = res.data;
    res = await nettuClient.deleteCalendar(calendarId, false);
    expect(res.status).toBe(401);
    res = await nettuClient.deleteCalendar(calendarId, true);
    expect(res.status).toBe(200);
    res = await nettuClient.deleteCalendar(calendarId, true);
    expect(res.status).toBe(404);
  });
});
