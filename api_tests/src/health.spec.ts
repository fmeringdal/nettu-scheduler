import { NettuClient } from "@nettu/sdk-scheduler";

describe("Health API", () => {
  const client = NettuClient();

  it("should report healthy status", async () => {
    const status = await client.health.checkStatus();
    expect(status).toBe(200);
  });
});
