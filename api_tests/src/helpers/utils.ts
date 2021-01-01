import fs from "fs";

const configFolder = __dirname + "/../../config";

export const readFile = async (path: string) => {
  return new Promise((res, rej) => {
    fs.readFile(path, (err, data) => {
      if (err) {
        return rej(err);
      }
      res(data);
    });
  });
};

export const toBase64 = (data: string) => Buffer.from(data).toString("base64");

export const readPublicKey = async () =>
  await readFile(`${configFolder}/test_public_rsa_key.crt`);

export const readPrivateKey = async (): Promise<string> =>
  (await readFile(`${configFolder}/test_private_rsa_key.pem`)) as string;

export const readPublicKeyBase64 = async () => {
  const pubkey = await readPublicKey();
  return toBase64(pubkey as string);
};
