import { NettuClient, NettuUserClient } from "@nettu/sdk-scheduler";
import { readPrivateKey, readPublicKey } from "./utils";
import jwt from "jsonwebtoken";

export const CREATE_ACCOUNT_CODE = "h0E4VC3d1bW1GLeW";

export const setupAccount = async () => {
  const client = NettuClient();
  const account = await client.account.create({ code: CREATE_ACCOUNT_CODE });
  return {
    client: NettuClient({ apiKey: account.data!.secretApiKey }),
    accountId: account.data!.account.id,
  };
};

export const setupUserClient = async () => {
  const { client, accountId } = await setupAccount();
  const publicKey = await readPublicKey();
  await client.account.setPublicSigningKey(publicKey);
  const privateKey = await readPrivateKey();
  const userRes = await client.user.create();
  const user = userRes.data!.user;
  const { client: userClient } = setupUserClientForAccount(
    privateKey,
    user.id,
    accountId
  );

  return {
    accountClient: client,
    userClient,
    userId: user.id,
    accountId,
  };
};

export const setupUserClientForAccount = (
  privateKey: string,
  userId: string,
  accountId: string
) => {
  const token = jwt.sign(
    {
      nettuSchedulerUserId: userId,
      schedulerPolicy: {
        allow: ["*"]
      }
    },
    privateKey,
    {
      algorithm: "RS256",
      expiresIn: "1h",
    }
  );
  return {
    token,
    client: NettuUserClient({ token, nettuAccount: accountId }),
  };
};

export const createAccountAndUser = async () => {
  const data = await setupUserClient();
  const user = await data.accountClient.user.create();
  return {
    ...data,
    user,
  };
};
