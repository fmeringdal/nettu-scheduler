import axios, { AxiosResponse } from "axios";

// TODO: add generics and return APIResponse in the other http methods (remaining: PUT, POST, DELETE, done: GET)
export abstract class NettuBaseClient {
  private readonly config = {
    baseUrl: "http://localhost:5000",
  };
  private readonly credentials: ICredentials;

  constructor(credentials: ICredentials) {
    this.credentials = credentials;
  }

  private getAxiosConfig = () => ({
    validateStatus: () => true, // allow all status codes without throwing error
    headers: this.credentials.createAuthHeaders(),
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

export class UserCreds implements ICredentials {
  private readonly nettuAccount: string;
  private readonly token?: string;

  constructor(nettuAccount: string, token?: string) {
    this.nettuAccount = nettuAccount;
    this.token = token;
  }

  createAuthHeaders() {
    const creds: any = {
      "nettu-account": this.nettuAccount,
    };
    if (this.token) {
      creds["authorization"] = `Bearer ${this.token}`;
    }

    return Object.freeze(creds);
  }
}

export class AccountCreds implements ICredentials {
  private readonly apiKey: string;

  constructor(apiKey: string) {
    this.apiKey = apiKey;
  }

  createAuthHeaders() {
    return Object.freeze({
      "x-api-key": this.apiKey,
    });
  }
}

export interface ICredentials {
  createAuthHeaders(): object;
}

export class EmptyCreds implements ICredentials {
  createAuthHeaders() {
    return Object.freeze({});
  }
}

export interface ICredentials {
  createAuthHeaders(): object;
}
