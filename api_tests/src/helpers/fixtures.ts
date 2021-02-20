import { NettuClient } from "@nettu/sdk-scheduler";
import { readPrivateKey, readPublicKeyBase64, toBase64 } from "./utils";
import jwt from "jsonwebtoken";

export const CREATE_ACCOUNT_CODE = "FW4KbTC2loN1Ckr8KkIcwE3Av";

export const setupAccount = async () => {
  const client = NettuClient();
  const account = await client.account.insert({ code: CREATE_ACCOUNT_CODE });
  return {
    client: NettuClient({ apiKey: account.data!.secretApiKey }),
    accountId: account.data!.accountId,
  };
};

export const setupUserClient = async () => {
  const { client, accountId } = await setupAccount();
  const publicKeyB64 = await readPublicKeyBase64();
  const r = await client.account.setPublicSigningKey(publicKeyB64);
  const privateKey = await readPrivateKey();
  const { userId, token, client: userClient } = setupUserClientForAccount(
    privateKey,
    accountId
  );

  return {
    accountClient: client,
    userClient,
    userId,
    accountId,
  };
};

export const setupUserClientForAccount = (
  privateKey: string,
  accountId: string
) => {
  const userId = "123";
  const token = jwt.sign(
    {
      userId,
      schedulerPolicy: {
        allow: ["*"]
      }
    },
    privateKey as string,
    {
      algorithm: "RS256",
      expiresIn: "1h",
    }
  );
  return {
    token,
    userId,
    client: NettuClient({ token, nettuAccount: accountId }),
  };
};

export const createAccountAndUser = async () => {
  const data = await setupUserClient();
  const user = await data.accountClient.user.create("456");
  return {
    ...data,
    user,
  };
};
