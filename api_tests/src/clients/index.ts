import { NettuAccountClient } from "./accountClient";
import { Credentials } from "./baseClient";
import { NettuCalendarClient } from "./calendarClient";
import { NettuEventClient } from "./eventClient";
import { NettuHealthClient } from "./healthClient";
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
    health: new NettuHealthClient(creds),
  };
};

const createCreds = (creds?: PartialCredentials): Credentials => {
  creds = creds ? creds : {};
  if (creds.apiKey) {
    return Credentials.createFromSecretKey({ apiKey: creds.apiKey });
  } else if (creds.nettuAccount) {
    return Credentials.createForUser({
      nettuAccount: creds.nettuAccount,
      token: creds.token,
    });
  } else {
    return Credentials.createEmpty();
  }
};
