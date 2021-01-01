import { INettuClient, NettuClient } from "./clients";
import { setupUserClient } from "./helpers/setup";

describe("Calendar API", () => {
  let client: INettuClient;
  const unauthClient = NettuClient();

  beforeAll(async () => {
    const data = await setupUserClient();
    client = data.userClient;
  });

  it("should not create calendar for unauthenticated user", async () => {
    const res = await unauthClient.calendar.insert(undefined);
    expect(res.status).toBe(401);
  });

  it("should create calendar for authenticated user", async () => {
    const res = await client.calendar.insert(undefined);
    // console.log(res);
    expect(res.status).toBe(201);
    expect(res.data.calendarId).toBeDefined();
  });

  it("should delete calendar for authenticated user and not for unauthenticated user", async () => {
    let res = await client.calendar.insert(undefined);
    const { calendarId } = res.data;
    res = await unauthClient.calendar.remove(calendarId);
    expect(res.status).toBe(401);
    res = await client.calendar.remove(calendarId);
    expect(res.status).toBe(200);
    res = await client.calendar.remove(calendarId);
    expect(res.status).toBe(404);
  });
});
