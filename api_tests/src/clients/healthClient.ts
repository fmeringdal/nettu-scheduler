import { NettuBaseClient } from "./baseClient";

export class NettuHealthClient extends NettuBaseClient {
  public async checkStatus(): Promise<number> {
    const res = await this.get<void>("/", false);
    return res.status;
  }
}

export const nettuHealthClient = new NettuHealthClient();
