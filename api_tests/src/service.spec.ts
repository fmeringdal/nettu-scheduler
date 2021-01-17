import { INettuClient } from "./clients";
import { setupAccount } from "./helpers/fixtures";

describe("Service API", () => {
  let client: INettuClient;
  beforeAll(async () => {
    const data = await setupAccount();
    client = data.client;
  });

  it("should create service", async () => {
    const res = await client.service.insert();
    expect(res.status).toBe(201);
  });
});
