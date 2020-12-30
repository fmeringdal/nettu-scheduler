import { Client } from "./clients";

describe("CalendarEvent API", () => {
  let calendarId = "";

  beforeAll(async () => {
    const calendarRes = await Client.calendar.insert(undefined, true);
    calendarId = calendarRes.data.calendarId;
  });

  it("should not let unauthenticated user create event", async () => {
    const res = await Client.events.insert(
      {
        calendarId,
        duration: 1000,
        startTs: 1000,
      },
      false
    );

    expect(res.status).toBe(401);
  });

  it("should let authenticated user create event", async () => {
    const res = await Client.events.insert(
      {
        calendarId,
        duration: 1000,
        startTs: 1000,
      },
      true
    );

    expect(res.status).toBe(201);
  });

  it("should create daily event and retrieve instances", async () => {
    const count = 10;
    let res = await Client.events.insert(
      {
        calendarId,
        duration: 1000,
        startTs: 1000,
        rruleOptions: {
          bynweekday: [[]],
          bysetpos: [],
          byweekday: [],
          freq: 4,
          interval: 1,
          tzid: "UTC",
          wkst: 0,
          count,
        },
      },
      true
    );
    const { eventId } = res.data;
    expect(res.status).toBe(201);
    res = await Client.events.getInstances(
      eventId,
      { startTs: 20, endTs: 1000 * 60 * 60 * 24 * (count + 1) },
      true
    );
    let instances = res.data.instances;
    expect(instances.length).toBe(count);

    // Query after instances are finished
    res = await Client.events.getInstances(
      eventId,
      {
        startTs: 1000 * 60 * 60 * 24 * (count + 1),
        endTs: 1000 * 60 * 60 * 24 * (count + 30),
      },
      true
    );
    instances = res.data.instances;
    expect(instances.length).toBe(0);
  });
});
