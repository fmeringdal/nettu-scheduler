import { nettuClient } from "./client";

describe("Calendar API", () => {
  it("should give status 200", async () => {
    const status = await nettuClient.checkStatus();
    expect(status).toBe(200);
  });
});
