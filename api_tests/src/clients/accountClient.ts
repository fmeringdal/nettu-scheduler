import { Account } from "../domain/account";
import { Calendar } from "../domain/calendar";
import { NettuBaseClient } from "./baseClient";

export type CreatedAccountResponse = {
  accountId: string;
  secretApiKey: string;
};

export type CreatedAccountRequest = {
  code: string;
};

export class NettuAccountClient extends NettuBaseClient {
  // data will be something in the future
  public insert(data: CreatedAccountRequest) {
    return this.post<CreatedAccountResponse>("/account", data);
  }

  public setPublicSigningKey(publicSigningKeyBase64?: string) {
    return this.put<void>("/account/pubkey", {
      publicKeyB64: publicSigningKeyBase64,
    });
  }

  public removePublicSigningKey() {
    return this.setPublicSigningKey();
  }

  public find() {
    return this.get<Account>(`/account`);
  }
}
