import axios, { AxiosResponse } from "axios";

// TODO: add generics and return APIResponse in the other http methods (remaining: PUT, POST, DELETE, done: GET)
export abstract class NettuBaseClient {
  private readonly config = {
    baseUrl: "http://localhost:5000",
  };
  private readonly credentials: Credentials;

  constructor(credentials: Credentials) {
    this.credentials = credentials;
  }

  private createAuthHeaders() {
    if (this.credentials.isAccountCredentials()) {
      return {
        "x-api-key": this.credentials.accountCreds!.apiKey,
      };
    } else if (this.credentials.isUserCredentials()) {
      const creds: any = {
        "nettu-account": this.credentials.userCreds!.nettuAccount,
      };
      if (this.credentials.userCreds!.token) {
        creds["authorization"] = `Bearer ${this.credentials.userCreds!.token}`;
      }

      return creds;
    } else {
      return {};
    }
  }

  private getAxiosConfig = () => ({
    validateStatus: () => true, // allow all status codes without throwing error
    headers: this.createAuthHeaders(),
  });

  protected async get<T>(path: string): Promise<APIResponse<T>> {
    const res = await axios.get(
      this.config.baseUrl + path,
      this.getAxiosConfig()
    );
    return new APIResponse(res);
  }

  protected async delete<T>(path: string): Promise<APIResponse<T>> {
    const res = await axios.delete(
      this.config.baseUrl + path,
      this.getAxiosConfig()
    );
    return new APIResponse(res);
  }

  protected async post<T>(path: string, data: any): Promise<APIResponse<T>> {
    const res = await axios.post(
      this.config.baseUrl + path,
      data,
      this.getAxiosConfig()
    );
    return new APIResponse(res);
  }

  protected async put<T>(path: string, data: any): Promise<APIResponse<T>> {
    const res = await axios.put(
      this.config.baseUrl + path,
      data,
      this.getAxiosConfig()
    );
    return new APIResponse(res);
  }
}

export class APIResponse<T> {
  readonly data: T;
  readonly status: number;
  readonly res: AxiosResponse;

  constructor(res: AxiosResponse) {
    this.res = res;
    this.data = res.data;
    this.status = res.status;
  }
}

export type UserCreds = {
  nettuAccount: string;
  token?: string;
};

export type AccountCreds = {
  apiKey: string;
};

export class Credentials {
  readonly userCreds?: UserCreds;
  readonly accountCreds?: AccountCreds;

  private constructor(userCreds?: UserCreds, accountCreds?: AccountCreds) {
    this.userCreds = userCreds;
    this.accountCreds = accountCreds;
  }

  public isAccountCredentials(): boolean {
    return this.accountCreds !== undefined;
  }

  public isUserCredentials(): boolean {
    // Account credentials take preference over user credentials (they should never both be set, butbut ...)
    return !this.isAccountCredentials() && this.userCreds !== undefined;
  }

  public static createFromSecretKey(creds: AccountCreds): Credentials {
    return new Credentials(undefined, creds);
  }

  public static createForUser(creds: UserCreds): Credentials {
    return new Credentials(creds, undefined);
  }

  public static createEmpty(): Credentials {
    return new Credentials(undefined, undefined);
  }
}
