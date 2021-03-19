import { NettuClient } from "@nettu/sdk-scheduler";
import {
  setupAccount,
  setupUserClientForAccount,
  CREATE_ACCOUNT_CODE,
} from "./helpers/fixtures";
import { readPrivateKey, readPublicKey } from "./helpers/utils";

describe("Account API", () => {
  const client = NettuClient();

  it("should create account", async () => {
    const { status, data } = await client.account.create({
      code: CREATE_ACCOUNT_CODE,
    });
    expect(status).toBe(201);
    expect(data!).toBeDefined();
  });

  it("should find account", async () => {
    const { status, data } = await client.account.create({
      code: CREATE_ACCOUNT_CODE,
    });
    const accountClient = NettuClient({ apiKey: data!.secretApiKey });
    const res = await accountClient.account.me();
    expect(res.status).toBe(200);
    expect(res.data!.account.id).toBe(data!.account.id);
  });

  it("should not find account when not signed in", async () => {
    const res = await client.account.me();
    expect(res.status).toBe(401);
  });

  it("should upload account public key and be able to remove it", async () => {
    const { client } = await setupAccount();
    const publicKey = await readPublicKey();
    await client.account.setPublicSigningKey(publicKey);
    let res = await client.account.me();
    expect(res.data!.account.publicJwtKey!).toBe(publicKey);
    const userRes = await client.user.create();
    const user = userRes.data!.user;
    // validate that a user can now use token to interact with api
    const privateKey = await readPrivateKey();
    const { client: userClient } = setupUserClientForAccount(
      privateKey,
      user.id,
      res.data!.account.id
    );
    const { status } = await userClient.calendar.create({ timezone: "UTC" });
    expect(status).toBe(201);
    // now disable public key and dont allow jwt token anymore
    await client.account.removePublicSigningKey();
    res = await client.account.me();
    expect(res.data!.account.publicJwtKey).toBeNull();

    const { status: status2 } = await userClient.calendar.create({ timezone: "UTC" });
    expect(status2).toBe(401);
  });
});
