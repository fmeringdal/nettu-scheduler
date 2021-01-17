import { INettuClient, NettuClient } from "@nettu/sdk-scheduler";
import { Frequenzy } from "./domain/calendarEvent";
import { setupUserClient } from "./helpers/fixtures";

describe("User API", () => {
  let userId: string;
  let calendarId: string;
  let accountClient: INettuClient;
  let client: INettuClient;
  let unauthClient: INettuClient;
  beforeAll(async () => {
    const data = await setupUserClient();
    client = data.userClient;
    accountClient = data.accountClient;
    userId = data.userId;
    unauthClient = NettuClient({ nettuAccount: data.accountId });
    const calendarRes = await client.calendar.insert(undefined);
    calendarId = calendarRes.data.calendarId;
  });

  it("should create user", async () => {
    const userId = "345677";
    let res = await accountClient.user.create(userId);
    expect(res.status).toBe(201);
    expect(res.data.id).toBe(userId);

    res = await accountClient.user.find(userId);
    expect(res.status).toBe(200);
    expect(res.data.id).toBe(userId);

    // Not allow create user with same userid
    res = await accountClient.user.create(userId);
    expect(res.status).toBe(409);
  });

  it("should delete user", async () => {
    const userId = "345677";
    await accountClient.user.create(userId);
    await accountClient.user.remove(userId);
    const res = await accountClient.user.find(userId);
    expect(res.status).toBe(404);
  });

  it("should not show any freebusy with no events", async () => {
    const res = await client.user.freebusy(userId, {
      endTs: 1000 * 60 * 60 * 24 * 4,
      startTs: 10,
      calendarIds: [calendarId],
    });
    expect(res.status).toBe(200);
    expect(res.data.free.length).toBe(0);
  });

  it("should show correct freebusy with a single event in calendar", async () => {
    const event = await client.events.insert({
      calendarId,
      duration: 1000 * 60 * 60,
      startTs: 0,
      rruleOptions: {
        freq: Frequenzy.Daily,
        interval: 1,
        tzid: "UTC",
        wkst: 0,
        count: 100,
      },
    });

    const res = await unauthClient.user.freebusy(userId, {
      endTs: 1000 * 60 * 60 * 24 * 4,
      startTs: 10,
      calendarIds: [calendarId],
    });
    expect(res.data.free.length).toBe(3);

    await client.events.remove(event.data.eventId);
  });

  it("should show correct freebusy with multiple events in calendar", async () => {
    const event1 = await client.events.insert({
      calendarId,
      duration: 1000 * 60 * 60,
      startTs: 0,
      rruleOptions: {
        freq: Frequenzy.Daily,
        interval: 1,
        tzid: "UTC",
        wkst: 0,
        count: 100,
      },
    });
    const event2 = await client.events.insert({
      calendarId,
      duration: 1000 * 60 * 60,
      startTs: 1000 * 60 * 60 * 4,
      rruleOptions: {
        freq: Frequenzy.Daily,
        interval: 1,
        tzid: "UTC",
        wkst: 0,
        count: 100,
      },
    });
    const event3 = await client.events.insert({
      calendarId,
      duration: 1000 * 60 * 60,
      startTs: 0,
      busy: true,
      rruleOptions: {
        freq: Frequenzy.Daily,
        interval: 2,
        tzid: "UTC",
        wkst: 0,
        count: 100,
      },
    });

    const res = await unauthClient.user.freebusy(userId, {
      endTs: 1000 * 60 * 60 * 24 * 4,
      startTs: 10,
      calendarIds: [calendarId],
    });

    expect(res.data.free.length).toBe(6);

    let bookingRes = await unauthClient.user.bookingslots(userId, {
      date: "1970-1-1",
      duration: 1000 * 60 * 30,
      ianaTz: "UTC",
      interval: 1000 * 60 * 15,
      calendarIds: [calendarId],
    });
    expect(bookingRes.data.bookingSlots.length).toBe(3);

    bookingRes = await unauthClient.user.bookingslots(userId, {
      date: "1970-1-2",
      duration: 1000 * 60 * 30,
      ianaTz: "UTC",
      interval: 1000 * 60 * 15,
      calendarIds: [calendarId],
    });
    expect(bookingRes.data.bookingSlots.length).toBe(6);

    for (const e of [event1, event2, event3]) {
      await client.events.remove(e.data.eventId);
    }
  });
});
