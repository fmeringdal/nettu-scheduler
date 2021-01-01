import { NettuClient } from "../clients";
import { readFile, toBase64 } from "./utils";
import jwt from "jsonwebtoken";

export const setupAccount = async () => {
  const client = NettuClient();
  const account = await client.account.insert(undefined);
  return {
    client: NettuClient({ apiKey: account.data.secretApiKey }),
    accountId: account.data.accountId,
  };
};

export const setupUserClient = async () => {
  const configFolder = __dirname + "/../../config";
  const { client, accountId } = await setupAccount();
  const publicKey = await readFile(`${configFolder}/test_public_rsa_key.crt`);
  const publicKeyB64 = toBase64(publicKey as string);
  const r = await client.account.setPublicSigningKey(publicKeyB64);
  const privateKey = await readFile(`${configFolder}/test_private_rsa_key.pem`);
  const userId = "123";
  const token = jwt.sign(
    {
      userId,
    },
    privateKey as string,
    {
      algorithm: "RS256",
      expiresIn: "1h",
    }
  );

  return {
    accountClient: client,
    userClient: NettuClient({ token, nettuAccount: accountId }),
    userId,
    accountId,
  };
};
