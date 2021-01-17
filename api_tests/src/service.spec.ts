import { INettuClient } from "./clients";
import { setupAccount } from "./helpers/fixtures";

describe("Service API", () => {
  let client: INettuClient;
  let accountId: string;
  beforeAll(async () => {
    const data = await setupAccount();
    client = data.client;
    accountId = data.accountId;
  });

  it("should create service", async () => {
    const res = await client.service.insert();
    expect(res.status).toBe(201);

    const serviceRes = await client.service.find(res.data.serviceId);
    expect(serviceRes.data.id).toBe(res.data.serviceId);
    expect(serviceRes.data.accountId).toBe(accountId);
  });
});
