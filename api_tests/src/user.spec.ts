import { Client } from "./clients";
import { NettuBaseClient } from "./clients/baseClient";

describe("User API", () => {
  let calendarId = "";
  const userId = NettuBaseClient.userId;

  beforeAll(async () => {
    let calendarRes = await Client.calendar.insert(undefined, true);
    calendarId = calendarRes.data.calendarId;
  });

  it("should not show any freebusy with no events", async () => {
    const res = await Client.user.freebusy(
      userId,
      {
        endTs: 1000 * 60 * 60 * 24 * 4,
        startTs: 10,
        calendarIds: [calendarId],
      },
      false
    );
    expect(res.status).toBe(200);
    expect(res.data.free.length).toBe(0);
  });

  it("should show correct freebusy with a single event in calendar", async () => {
    const event = await Client.events.insert(
      {
        calendarId,
        duration: 1000 * 60 * 60,
        startTs: 0,
        rruleOptions: {
          bynweekday: [],
          bysetpos: [],
          byweekday: [],
          freq: 4,
          interval: 1,
          tzid: "UTC",
          wkst: 0,
          count: 100,
        },
      },
      true
    );

    const res = await Client.user.freebusy(
      userId,
      {
        endTs: 1000 * 60 * 60 * 24 * 4,
        startTs: 10,
        calendarIds: [calendarId],
      },
      false
    );
    console.log(res.data.free);
    expect(res.data.free.length).toBe(3);

    await Client.events.remove(event.data.eventId, true);
  });

  it("should show correct freebusy with multiple events in calendar", async () => {
    const event1 = await Client.events.insert(
      {
        calendarId,
        duration: 1000 * 60 * 60,
        startTs: 0,
        rruleOptions: {
          bynweekday: [],
          bysetpos: [],
          byweekday: [],
          freq: 4,
          interval: 1,
          tzid: "UTC",
          wkst: 0,
          count: 100,
        },
      },
      true
    );
    const event2 = await Client.events.insert(
      {
        calendarId,
        duration: 1000 * 60 * 60,
        startTs: 1000 * 60 * 60 * 4,
        rruleOptions: {
          bynweekday: [],
          bysetpos: [],
          byweekday: [],
          freq: 4,
          interval: 1,
          tzid: "UTC",
          wkst: 0,
          count: 100,
        },
      },
      true
    );
    const event3 = await Client.events.insert(
      {
        calendarId,
        duration: 1000 * 60 * 60,
        startTs: 0,
        busy: true,
        rruleOptions: {
          bynweekday: [],
          bysetpos: [],
          byweekday: [],
          freq: 4,
          interval: 2,
          tzid: "UTC",
          wkst: 0,
          count: 100,
        },
      },
      true
    );

    const res = await Client.user.freebusy(
      userId,
      {
        endTs: 1000 * 60 * 60 * 24 * 4,
        startTs: 10,
        calendarIds: [calendarId],
      },
      false
    );

    expect(res.data.free.length).toBe(6);

    let bookingRes = await Client.user.bookingslots(
      userId,
      {
        date: "1970-1-1",
        duration: 1000 * 60 * 30,
        ianaTz: "UTC",
        calendarIds: [calendarId],
      },
      true
    );
    expect(bookingRes.data.bookingSlots.length).toBe(3);

    bookingRes = await Client.user.bookingslots(
      userId,
      {
        date: "1970-1-2",
        duration: 1000 * 60 * 30,
        ianaTz: "UTC",
        calendarIds: [calendarId],
      },
      true
    );
    expect(bookingRes.data.bookingSlots.length).toBe(6);

    for (const e of [event1, event2, event3]) {
      await Client.events.remove(e.data.eventId, true);
    }
  });
});
