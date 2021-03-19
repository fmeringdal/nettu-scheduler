import { config, NettuClient } from "@nettu/sdk-scheduler";

describe("Health API", () => {
  const client = NettuClient();

  it("should report healthy status", async () => {
    config.baseUrl = "http://localhost:5000/api/v1";
    const status = await client.health.checkStatus();
    expect(status).toBe(200);
  });
});
