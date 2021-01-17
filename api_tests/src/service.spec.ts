import { INettuClient } from "@nettu/sdk-scheduler";
import { setupAccount } from "./helpers/fixtures";

describe("Service API", () => {
  let client: INettuClient;
  let accountId: string;
  beforeAll(async () => {
    const data = await setupAccount();
    client = data.client;
    accountId = data.accountId;
  });

  it("should create and find service", async () => {
    const res = await client.service.insert();
    expect(res.status).toBe(201);

    const serviceRes = await client.service.find(res.data.serviceId);
    expect(serviceRes.data.id).toBe(res.data.serviceId);
    expect(serviceRes.data.accountId).toBe(accountId);
  });

  it("should add user to service", async () => {
    const {
      data: { serviceId },
    } = await client.service.insert();

    const user1 = await client.user.create("1");

    await client.service.addUserToService(serviceId, {
      calendarIds: [],
      userId: user1.data.id,
    });

    const service = await client.service.find(serviceId);
    expect(service.data.users.length).toBe(1);
  });

  it("should remove user from service", async () => {
    const {
      data: { serviceId },
    } = await client.service.insert();

    const user1 = await client.user.create("1");

    await client.service.addUserToService(serviceId, {
      calendarIds: [],
      userId: user1.data.id,
    });
    await client.service.removeUserFromService(serviceId, user1.data.id);

    const service = await client.service.find(serviceId);
    expect(service.data.users.length).toBe(0);
  });

  it("should get service bookingslots with no users", async () => {
    const {
      data: { serviceId },
    } = await client.service.insert();

    const service = await client.service.getBookingslots(serviceId, {
      date: "1980-1-1",
      duration: 60 * 60 * 1000,
      ianaTz: "UTC",
      interval: 15 * 60 * 1000,
    });

    expect(service.data.bookingSlots.length).toBe(0);
  });
});
