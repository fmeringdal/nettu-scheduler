import fs from "fs";

const configFolder = __dirname + "/../../config";

export const readFile = async (path: string) => {
  return fs.readFileSync(path).toString();
};


export const readPublicKey = async () =>
  await readFile(`${configFolder}/test_public_rsa_key.crt`);

export const readPrivateKey = async (): Promise<string> =>
  await readFile(`${configFolder}/test_private_rsa_key.pem`)
