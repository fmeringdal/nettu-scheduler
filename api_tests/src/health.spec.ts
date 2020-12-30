import { Client } from "./clients";

describe("Health API", () => {
  it("should report healthy status", async () => {
    const status = await Client.health.checkStatus();
    expect(status).toBe(200);
  });
});
