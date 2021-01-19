import { INettuClient, NettuClient, domain } from "@nettu/sdk-scheduler";
import { setupUserClient } from "./helpers/fixtures";

describe("Schedule API", () => {
  let client: INettuClient;
  const unauthClient = NettuClient();

  beforeAll(async () => {
    const data = await setupUserClient();
    client = data.userClient;
  });

  it("should not create schedule for unauthenticated user", async () => {
    const res = await unauthClient.schedule.insert({
      timezone: "Europe/Berlin",
    });
    expect(res.status).toBe(401);
  });

  it("should create schedule for authenticated user", async () => {
    const res = await client.schedule.insert({
      timezone: "Europe/Berlin",
    });
    expect(res.status).toBe(201);
    expect(res.data.id).toBeDefined();
  });

  it("should delete schedule for authenticated user and not for unauthenticated user", async () => {
    let { data: schedule } = await client.schedule.insert({
      timezone: "Europe/Berlin",
    });
    let res = await unauthClient.schedule.remove(schedule.id);
    expect(res.status).toBe(401);
    res = await client.schedule.remove(schedule.id);
    expect(res.status).toBe(200);
    res = await client.schedule.remove(schedule.id);
    expect(res.status).toBe(404);
  });

  it("should update schedule", async () => {
    const { data: schedule } = await client.schedule.insert({
      timezone: "Europe/Berlin",
    });
    const { data: updatedSchedule } = await client.schedule.update(
      schedule.id,
      {
        rules: [
          {
            variant: {
              type: domain.ScheduleRuleVariant.WDay,
              value: domain.Weekday.Mon,
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

    expect(updatedSchedule.id).toBe(schedule.id);
    expect(updatedSchedule.timezone).toBe("UTC");
    expect(updatedSchedule.rules.length).toBe(1);
  });
});
