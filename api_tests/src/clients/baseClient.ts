import axios, { AxiosResponse } from "axios";

// TODO: add generics and return APIResponse in the other http methods (remaining: PUT, POST, DELETE, done: GET)
export abstract class NettuBaseClient {
  private config = {
    baseUrl: "http://localhost:5000",
  };
  public static userId = "coooltestuser";
  public static token =
    "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VySWQiOiJjb29vbHRlc3R1c2VyIiwiaWF0IjoxNTE2MjM5MDIyLCJleHAiOjI1MTYyMzkwMjJ9.1qAop_g3Fb9NXa5-65Lk8xvHKvY6LRkRbFK47thRDwM";
  private authHeaders = {
    authorization: `Bearer ${NettuBaseClient.token}`,
  };

  private getAxiosConfig = (auth: boolean) => ({
    validateStatus: () => true, // allow all status codes without throwing error
    headers: auth ? this.authHeaders : undefined,
  });

  protected async get<T>(path: string, auth: boolean): Promise<APIResponse<T>> {
    const res = await axios.get(
      this.config.baseUrl + path,
      this.getAxiosConfig(auth)
    );
    return new APIResponse(res);
  }

  protected async delete(path: string, auth: boolean): Promise<AxiosResponse> {
    return axios.delete(this.config.baseUrl + path, this.getAxiosConfig(auth));
  }

  protected async post(
    path: string,
    data: any,
    auth: boolean
  ): Promise<AxiosResponse> {
    return axios.post(
      this.config.baseUrl + path,
      data,
      this.getAxiosConfig(auth)
    );
  }

  protected async put(
    path: string,
    data: any,
    auth: boolean
  ): Promise<AxiosResponse> {
    return axios.put(
      this.config.baseUrl + path,
      data,
      this.getAxiosConfig(auth)
    );
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
