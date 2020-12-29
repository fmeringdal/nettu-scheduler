import axios, { AxiosResponse } from "axios";

export class NettuClient {
  config = {
    baseUrl: "http://localhost:5000",
  };
  authHeaders = {
    authorization:
      "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VySWQiOiJjb29vbHVzZXIiLCJpYXQiOjE1MTYyMzkwMjIsImV4cCI6MjUxNjIzOTAyMn0.T4bxq5L6UGY7KYu9z83kIv98qaFnpAx43Eav3D3ztwE",
  };

  private getAxiosConfig = (auth: boolean) => ({
    validateStatus: () => true, // allow all status codes without throwing error
    headers: auth ? this.authHeaders : undefined,
  });

  private async get(path: string, auth: boolean): Promise<AxiosResponse> {
    return axios.get(this.config.baseUrl + path, this.getAxiosConfig(auth));
  }

  private async post(
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

  private async put(
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

  private async delete(path: string, auth: boolean): Promise<AxiosResponse> {
    return axios.delete(this.config.baseUrl + path, this.getAxiosConfig(auth));
  }

  public updateCalendar(data: any, auth: boolean) {
    return this.put("/calendar", data, auth);
  }

  public createCalendar(data: any, auth: boolean) {
    return this.post("/calendar", data, auth);
  }

  public getCalendar(calendarId: string, auth: boolean) {
    return this.get(`/calendar/${calendarId}`, auth);
  }

  public deleteCalendar(calendarId: string, auth: boolean) {
    return this.delete(`/calendar/${calendarId}`, auth);
  }

  public async checkStatus(): Promise<number> {
    const res = await this.get("/", false);
    return res.status;
  }
}

export const nettuClient = new NettuClient();
