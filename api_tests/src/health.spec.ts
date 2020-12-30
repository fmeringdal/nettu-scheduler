import { Client } from "./clients";

describe("Calendar API", () => {
  it("should give status 200", async () => {
    const status = await Client.health.checkStatus();
    expect(status).toBe(200);
  });
});
