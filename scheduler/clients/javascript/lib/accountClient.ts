import { Account } from "./domain/account";
import { NettuBaseClient } from "./baseClient";

type AccountResponse = {
  account: Account;
};

type CreateAccountResponse = {
  account: Account;
  secretApiKey: string;
};

type CreateAccountRequest = {
  code: string;
};

type GoogleIntegration = {
  clientId: string;
  clientSecret: string;
  redirectUri: string;
};

export class NettuAccountClient extends NettuBaseClient {
  // data will be something in the future
  public create(data: CreateAccountRequest) {
    return this.post<CreateAccountResponse>("/account", data);
  }

  public setPublicSigningKey(publicSigningKey?: string) {
    return this.put<AccountResponse>("/account/pubkey", {
      publicJwtKey: publicSigningKey,
    });
  }

  public removePublicSigningKey() {
    return this.setPublicSigningKey();
  }

  public setWebhook(url: string) {
    return this.put<AccountResponse>(`/account/webhook`, {
      webhookUrl: url,
    });
  }

  public connectGoogle(data: GoogleIntegration) {
    return this.put<AccountResponse>(`/account/integration/google`, data);
  }

  public removeWebhook() {
    return this.delete<AccountResponse>(`/account/webhook`);
  }

  public me() {
    return this.get<AccountResponse>(`/account`);
  }
}
