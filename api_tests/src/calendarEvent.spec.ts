import { INettuClient, NettuClient } from "./clients";
import { setupUserClient } from "./helpers/setup";

describe("CalendarEvent API", () => {
  let calendarId: string;
  let client: INettuClient;
  let unauthClient: INettuClient;
  beforeEach(async () => {
    const data = await setupUserClient();
    client = data.userClient;
    unauthClient = NettuClient({ nettuAccount: data.accountId });
    const calendarRes = await client.calendar.insert(undefined);
    calendarId = calendarRes.data.calendarId;
  });

  it("should not let unauthenticated user create event", async () => {
    const res = await unauthClient.events.insert({
      calendarId,
      duration: 1000,
      startTs: 1000,
    });

    expect(res.status).toBe(401);
  });

  it("should let authenticated user create event", async () => {
    const res = await client.events.insert({
      calendarId,
      duration: 1000,
      startTs: 1000,
    });
    expect(res.status).toBe(201);
  });

  it("should create daily event and retrieve instances", async () => {
    const count = 10;
    let res = await client.events.insert({
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
    });
    const { eventId } = res.data;
    expect(res.status).toBe(201);
    let res2 = await client.events.getInstances(eventId, {
      startTs: 20,
      endTs: 1000 * 60 * 60 * 24 * (count + 1),
    });
    let instances = res2.data.instances;
    expect(instances.length).toBe(count);

    // Query after instances are finished
    let res3 = await client.events.getInstances(eventId, {
      startTs: 1000 * 60 * 60 * 24 * (count + 1),
      endTs: 1000 * 60 * 60 * 24 * (count + 30),
    });
    instances = res3.data.instances;
    expect(instances.length).toBe(0);
  });

  it("should create exception for calendar event", async () => {
    const count = 10;
    let res = await client.events.insert({
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
    });

    const { eventId } = res.data;
    const getInstances = async () => {
      const res = await client.events.getInstances(eventId, {
        startTs: 20,
        endTs: 1000 * 60 * 60 * 24 * (count + 1),
      });
      return res.data.instances;
    };
    const instancesBeforeException = await getInstances();

    // do create exception
    const res2 = await client.events.createException(eventId, {
      exceptionTs: 1000 + 24 * 60 * 60 * 1000,
    });
    expect(res2.status).toBe(200);

    const instancesAfterException = await getInstances();
    expect(instancesAfterException.length).toBe(
      instancesBeforeException.length - 1
    );
  });

  it("updating calendar event start time removes exception", async () => {
    const count = 10;
    let res = await client.events.insert({
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
    });

    const { eventId } = res.data;
    const getInstances = async () => {
      const res = await client.events.getInstances(eventId, {
        startTs: 20,
        endTs: 1000 * 60 * 60 * 24 * (count + 1),
      });
      return res.data.instances;
    };
    const instancesBeforeException = await getInstances();
    // do create exception
    const res2 = await client.events.createException(eventId, {
      exceptionTs: 1000 + 24 * 60 * 60 * 1000,
    });
    expect(res2.status).toBe(200);

    const instancesAfterException = await getInstances();
    expect(instancesAfterException.length).toBe(
      instancesBeforeException.length - 1
    );
    await client.events.update(eventId, {
      startTs: 1000 + 24 * 60 * 60 * 1000,
    });
    const instancesAfterExceptionDeleted = await getInstances();
    expect(instancesAfterExceptionDeleted.length).toBe(
      instancesBeforeException.length
    );
  });
});
