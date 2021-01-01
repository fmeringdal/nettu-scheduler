import fs from "fs";

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
