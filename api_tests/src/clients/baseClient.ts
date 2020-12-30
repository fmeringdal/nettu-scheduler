import axios, { AxiosResponse } from "axios";

// TODO: add generics and return APIResponse in the other http methods (remaining: PUT, POST, DELETE, done: GET)
export abstract class NettuBaseClient {
  private config = {
    baseUrl: "http://localhost:5000",
  };
  public static userId = "coooltestuser";
  public static token =
    "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VySWQiOiJjb29vbHRlc3R1c2VyIiwiaWF0IjoxNTE2MjM5MDIyLCJleHAiOjE5MTYyMzkwMjJ9.O1ENncekZdLZC8QkYa2Kz8ih6CDq0cCrx-nxiSgtrcFTr6fwxIBgEqg-eWnqKheFvgcF4wWoVwmp2so6bnsHDgeRowG7k8iRRP963zBiqZpXWRQNRJEuyjyRUeYknYsCuF6pWsCwANlULvM6axCGMv69C0urd9Qiv82Xw6Ox9-nLKmnzVcuXnPXFTG-GUPS4QiVsLA70pyq5tLiixHm9mtT2OcPX-n2kjXfQrHG9wzQL6peGIB7A6jNlP5wtDQJ0MQ4bnXJTMRTbiXLl86X_t2zxj_1FkY0z5VbDFtoaEKqaXj4pECUZZ6V__kqDBX9hYmW66OZdTugKLFxVRGHFnQ";
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
