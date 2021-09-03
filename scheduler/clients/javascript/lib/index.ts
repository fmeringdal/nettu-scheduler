import { NettuAccountClient } from "./accountClient";
import {
  AccountCreds,
  EmptyCreds,
  ICredentials,
  UserCreds,
} from "./baseClient";
import { NettuCalendarClient, NettuCalendarUserClient } from "./calendarClient";
import { NettuEventClient, NettuEventUserClient } from "./eventClient";
import { NettuHealthClient } from "./healthClient";
import { NettuScheduleUserClient, NettuScheduleClient } from "./scheduleClient";
import { NettuServiceUserClient, NettuServiceClient } from "./serviceClient";
import { NettuUserClient as _NettuUserClient, NettuUserUserClient } from "./userClient";

export * from "./domain";

type PartialCredentials = {
  apiKey?: string;
  nettuAccount?: string;
  token?: string;
};

export interface INettuUserClient {
  calendar: NettuCalendarUserClient;
  events: NettuEventUserClient;
  service: NettuServiceUserClient;
  schedule: NettuScheduleUserClient;
  user: NettuUserUserClient;
}

export interface INettuClient {
  account: NettuAccountClient;
  calendar: NettuCalendarClient;
  events: NettuEventClient;
  health: NettuHealthClient;
  service: NettuServiceClient;
  schedule: NettuScheduleClient;
  user: _NettuUserClient;
}

type ClientConfig = {
  baseUrl: string;
};

export const config: ClientConfig = {
  baseUrl: "http://localhost:5000/api/v1",
};

export const NettuUserClient = (
  partialCreds?: PartialCredentials
): INettuUserClient => {
  const creds = createCreds(partialCreds);

  return Object.freeze({
    calendar: new NettuCalendarUserClient(creds),
    events: new NettuEventUserClient(creds),
    service: new NettuServiceUserClient(creds),
    schedule: new NettuScheduleUserClient(creds),
    user: new NettuUserUserClient(creds)
  });
};

export const NettuClient = (
  partialCreds?: PartialCredentials
): INettuClient => {
  const creds = createCreds(partialCreds);

  return Object.freeze({
    account: new NettuAccountClient(creds),
    events: new NettuEventClient(creds),
    calendar: new NettuCalendarClient(creds),
    user: new _NettuUserClient(creds),
    service: new NettuServiceClient(creds),
    schedule: new NettuScheduleClient(creds),
    health: new NettuHealthClient(creds),
  });
};

const createCreds = (creds?: PartialCredentials): ICredentials => {
  creds = creds ? creds : {};
  if (creds.apiKey) {
    return new AccountCreds(creds.apiKey);
  } else if (creds.nettuAccount) {
    return new UserCreds(creds.nettuAccount, creds.token);
  } else {
    // throw new Error("No api key or nettu account provided to nettu client.");
    return new EmptyCreds();
  }
};
