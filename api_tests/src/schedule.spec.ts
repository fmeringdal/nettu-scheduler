import { INettuClient, INettuUserClient, NettuClient, ScheduleRuleVariant, Weekday } from "@nettu/sdk-scheduler";
import { setupUserClient } from "./helpers/fixtures";

describe("Schedule API", () => {
  let client: INettuUserClient;
  const unauthClient = NettuClient();
  let userId: string;

  beforeAll(async () => {
    const data = await setupUserClient();
    client = data.userClient;
    userId = data.userId;
  });

  it("should not create schedule for unauthenticated user", async () => {
    const res = await unauthClient.schedule.create(userId, {
      timezone: "Europe/Berlin",
    });
    expect(res.status).toBe(401);
  });

  it("should create schedule for authenticated user", async () => {
    const res = await client.schedule.create({
      timezone: "Europe/Berlin",
    });
    expect(res.status).toBe(201);
    expect(res.data!.schedule.id).toBeDefined();
  });

  it("should delete schedule for authenticated user and not for unauthenticated user", async () => {
    let { data } = await client.schedule.create({
      timezone: "Europe/Berlin",
    });
    const scheduleId = data!.schedule.id;

    let res = await unauthClient.schedule.remove(scheduleId);
    expect(res.status).toBe(401);
    res = await client.schedule.remove(scheduleId);
    expect(res.status).toBe(200);
    res = await client.schedule.remove(scheduleId);
    expect(res.status).toBe(404);
  });

  it("should update schedule", async () => {
    const { data } = await client.schedule.create({
      timezone: "Europe/Berlin",
    });
    const scheduleId = data!.schedule.id;
    const updatedScheduleRes = await client.schedule.update(
      scheduleId,
      {
        rules: [
          {
            variant: {
              type: ScheduleRuleVariant.WDay,
              value: Weekday.Mon,
            },
            intervals: [
              {
                start: {
                  hours: 10,
                  minutes: 0,
                },
                end: {
                  hours: 12,
                  minutes: 30,
                },
              },
            ],
          },
        ],
        timezone: "UTC",
      }
    );
    const updatedSchedule = updatedScheduleRes.data!.schedule;

    expect(updatedSchedule!.id).toBe(scheduleId);
    expect(updatedSchedule!.timezone).toBe("UTC");
    expect(updatedSchedule!.rules.length).toBe(1);
  });
});
