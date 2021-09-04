import {
  INettuClient,
  INettuUserClient,
  ScheduleRuleVariant,
  Weekday,
} from "../lib";
import { setupAccount, setupUserClient } from "./helpers/fixtures";

describe("Service API", () => {
  let client: INettuClient;
  let userClient: INettuUserClient;
  let accountId: string;
  let userId: string;
  beforeAll(async () => {
    const data = await setupUserClient();
    client = data.accountClient;
    accountId = data.accountId;
    userClient = data.userClient;
    userId = data.userId;
  });

  it("should create and find service", async () => {
    const res = await client.service.create();
    expect(res.status).toBe(201);

    const serviceRes = await client.service.find(res.data!.service.id);
    expect(serviceRes.data!.id).toBe(res.data!.service.id);
  });

  it("should add user to service", async () => {
    const serviceRes = await client.service.create();

    const userRes = await client.user.create();

    await client.service.addUser(serviceRes.data!.service.id, {
      userId: userRes.data!.user.id,
    });

    const service = await client.service.find(serviceRes.data!.service.id);
    expect(service.data!.users.length).toBe(1);
  });

  it("should remove user from service", async () => {
    const serviceRes = await client.service.create();

    const user = await client.user.create();

    await client.service.addUser(serviceRes.data!.service.id, {
      userId: user.data!.user.id,
    });
    await client.service.removeUser(
      serviceRes.data!.service.id,
      user.data!.user.id
    );

    const service = await client.service.find(serviceRes.data!.service.id);
    expect(service.data!.users.length).toBe(0);
  });

  it("should get service bookingslots with no users", async () => {
    const serviceRes = await client.service.create();

    const service = await client.service.getBookingslots(
      serviceRes.data!.service.id,
      {
        startDate: "2030-1-1",
        endDate: "2030-1-3",
        duration: 60 * 60 * 1000,
        ianaTz: "UTC",
        interval: 15 * 60 * 1000,
      }
    );

    expect(service.data!.dates.length).toBe(0);
  });

  it("should get service bookingslots with one user with a schedule", async () => {
    const serviceRes = await client.service.create();
    const serviceId = serviceRes.data!.service.id;

    // Available all the time schedule
    const scheduleRes = await userClient.schedule.create({
      timezone: "Europe/Berlin",
      rules: [
        Weekday.Mon,
        Weekday.Tue,
        Weekday.Wed,
        Weekday.Thu,
        Weekday.Fri,
        Weekday.Sat,
        Weekday.Sun,
      ].map((day) => ({
        variant: {
          type: ScheduleRuleVariant.WDay,
          value: day,
        },
        intervals: [
          {
            start: {
              hours: 0,
              minutes: 0,
            },
            end: {
              hours: 23,
              minutes: 59,
            },
          },
        ],
      })),
    });

    const scheduleId = scheduleRes.data!.schedule.id;
    const closestBookingTime = 60; // one hour in minutes
    await client.service.addUser(serviceId, {
      userId,
      availability: {
        variant: "Schedule",
        id: scheduleId,
      },
      closestBookingTime,
    });

    const now = new Date();
    const today = `${now.getFullYear()}-${now.getMonth() + 1}-${now.getDate()}`;

    const { data } = await client.service.getBookingslots(serviceId, {
      startDate: today,
      endDate: today,
      duration: 60 * 60 * 1000,
      ianaTz: "UTC",
      interval: 15 * 60 * 1000,
    });

    expect(data!.dates.length).toBe(1);
    let bookingSlots = data!.dates[0].slots;
    expect(bookingSlots[0].start).toBeGreaterThanOrEqual(
      now.valueOf() + closestBookingTime
    );

    const { data: dataFuture } = await client.service.getBookingslots(
      serviceId,
      {
        startDate: "2030-10-10",
        endDate: "2030-10-10",
        duration: 60 * 60 * 1000,
        ianaTz: "UTC",
        interval: 15 * 60 * 1000,
      }
    );

    expect(data!.dates.length).toBe(1);
    bookingSlots = dataFuture!.dates[0].slots;
    expect(bookingSlots.length).toBe(89);

    // Quqerying for bookingslots in the past should not yield and bookingslots
    const { data: data2 } = await client.service.getBookingslots(serviceId, {
      startDate: "1980-1-1",
      endDate: "1980-1-1",
      duration: 60 * 60 * 1000,
      ianaTz: "UTC",
      interval: 15 * 60 * 1000,
    });

    expect(data2!.dates.length).toBe(0);
  });
});
