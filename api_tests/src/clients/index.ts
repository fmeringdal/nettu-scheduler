import { NettuAccountClient } from "./accountClient";
import {
  AccountCreds,
  EmptyCreds,
  ICredentials,
  UserCreds,
} from "./baseClient";
import { NettuCalendarClient } from "./calendarClient";
import { NettuEventClient } from "./eventClient";
import { NettuHealthClient } from "./healthClient";
import { NettuServiceClient } from "./serviceClient";
import { NettuUserClient } from "./userClient";

type PartialCredentials = {
  apiKey?: string;
  nettuAccount?: string;
  token?: string;
};

export interface INettuClient {
  account: NettuAccountClient;
  calendar: NettuCalendarClient;
  events: NettuEventClient;
  user: NettuUserClient;
  service: NettuServiceClient;
  health: NettuHealthClient;
}

export const NettuClient = (
  partialCreds?: PartialCredentials
): INettuClient => {
  let creds = createCreds(partialCreds);

  return {
    account: new NettuAccountClient(creds),
    events: new NettuEventClient(creds),
    calendar: new NettuCalendarClient(creds),
    user: new NettuUserClient(creds),
    service: new NettuServiceClient(creds),
    health: new NettuHealthClient(creds),
  };
};

const createCreds = (creds?: PartialCredentials): ICredentials => {
  creds = creds ? creds : {};
  if (creds.apiKey) {
    return new AccountCreds(creds.apiKey);
  } else if (creds.nettuAccount) {
    return new UserCreds(creds.nettuAccount, creds.token);
  } else {
    return new EmptyCreds();
    // throw new Error("No api key or nettu account provided to nettu client.");
  }
};
