import { NettuClient } from "@nettu/sdk-scheduler";
import {
  setupAccount,
  setupUserClientForAccount,
  CREATE_ACCOUNT_CODE,
} from "./helpers/fixtures";
import { readPrivateKey, readPublicKeyBase64 } from "./helpers/utils";

describe("Account API", () => {
  const client = NettuClient();

  it("should create account", async () => {
    const { status, data } = await client.account.insert({
      code: CREATE_ACCOUNT_CODE,
    });
    expect(status).toBe(201);
    expect(data.secretApiKey).toBeDefined();
  });

  it("should find account", async () => {
    const { status, data } = await client.account.insert({
      code: CREATE_ACCOUNT_CODE,
    });
    const accountClient = NettuClient({ apiKey: data.secretApiKey });
    const res = await accountClient.account.find();
    expect(res.status).toBe(200);
    expect(res.data.id).toBe(data.accountId);
  });

  it("should not find account when not signed in", async () => {
    await client.account.insert({ code: CREATE_ACCOUNT_CODE });
    const res = await client.account.find();
    expect(res.status).toBe(401);
    expect(res.data.id).toBeUndefined();
  });

  it("should upload account public key and be able to remove it", async () => {
    const { client } = await setupAccount();
    const publicKeyB64 = await readPublicKeyBase64();
    await client.account.setPublicSigningKey(publicKeyB64);
    let res = await client.account.find();
    expect(res.data.public_key_b64).toBe(publicKeyB64);
    // validate that a user can now use token to interact with api
    const privateKey = await readPrivateKey();
    const { client: userClient } = setupUserClientForAccount(
      privateKey,
      res.data.id
    );
    const { status } = await userClient.calendar.insert(undefined);
    expect(status).toBe(201);
    // now disable public key and dont allow jwt token anymore
    await client.account.removePublicSigningKey();
    res = await client.account.find();
    expect(res.data.public_key_b64).toBeNull();

    const { status: status2 } = await userClient.calendar.insert(undefined);
    expect(status2).toBe(401);
  });
});
