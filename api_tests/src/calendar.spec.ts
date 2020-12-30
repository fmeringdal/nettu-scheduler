import { Client } from "./clients";

describe("Calendar API", () => {
  it("should not create calendar for unauthenticated user", async () => {
    const res = await Client.calendar.insert(undefined, false);
    expect(res.status).toBe(401);
  });

  it("should create calendar for authenticated user", async () => {
    const res = await Client.calendar.insert(undefined, true);
    expect(res.status).toBe(201);
    expect(res.data.calendarId).toBeDefined();
  });

  it("should delete calendar for authenticated user and not for unauthenticated user", async () => {
    let res = await Client.calendar.insert(undefined, true);
    const { calendarId } = res.data;
    res = await Client.calendar.remove(calendarId, false);
    expect(res.status).toBe(401);
    res = await Client.calendar.remove(calendarId, true);
    expect(res.status).toBe(200);
    res = await Client.calendar.remove(calendarId, true);
    expect(res.status).toBe(404);
  });
});
